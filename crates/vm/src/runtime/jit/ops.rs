// ============================================================
// JIT runtime callout functions
//
// 本模块包含 JIT 编译的代码回调的 extern "C" 函数。
// 这些函数处理需要访问 JVM 运行时状态的操作（字段访问、对象分配、
// 类型检查、数组操作等）。
//
// ## 设计原则
//
// 1. 函数签名使用原始 i32/u32 类型，匹配 LLVM IR 中的 i32 表示
// 2. 通过 TLS `JitInvokeCtx` 获取方法所属类，用于常量池解析
// 3. 异常通过 `exception::meet_ex` 设置，JIT 代码在返回后检查
// 4. 返回值通过修改栈槽传递（引用返回 slot_id，int 直接写入）
// ============================================================

use crate::oop::{self, Oop};
use crate::runtime::cmp;
use crate::runtime::exception;
use crate::runtime::jit::runtime::{get_invoke_ctx, restore_invoke_ctx};
use crate::runtime::{require_class3, ClassLoader};
use crate::util;
use classfile::constant_pool;
use classfile::consts as cls_const;
use tracing::warn;

/// Resolve a class from the constant pool using the method class.
fn resolve_cp_class(
    cp_idx: u16,
    method_cls: &crate::types::ClassRef,
) -> Option<crate::types::ClassRef> {
    let cp = match method_cls.get_constant_pool() {
        Some(cp) => cp,
        None => return None,
    };
    let class_name = constant_pool::get_class_name(&cp, cp_idx as usize);
    require_class3(Some(ClassLoader::Bootstrap), class_name.as_slice())
}

/// Resolve a class directly from a name slice.
fn resolve_cp_class_from_name(name: &[u8]) -> Option<crate::types::ClassRef> {
    require_class3(Some(ClassLoader::Bootstrap), name)
}

/// JIT `new` 指令：创建对象实例。
/// 返回 slot_id（引用），0 表示 null（异常情况下）。
#[no_mangle]
pub extern "C" fn jit_new_inst(cp_idx: u16) -> u32 {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return 0,
    };
    let cls = ctx.method_class.clone();

    // 解析类
    let class = match resolve_cp_class(cp_idx, &cls) {
        Some(c) => c,
        None => {
            exception::meet_ex(cls_const::J_CLASS_NOT_FOUND, None);
            restore_invoke_ctx(Some(ctx));
            return 0;
        }
    };

    // 初始化类
    oop::class::init_class(&class);
    oop::class::init_class_fully(&class);

    // 创建实例
    let oop = Oop::new_inst(class);
    restore_invoke_ctx(Some(ctx));

    match oop {
        Oop::Ref(slot_id) => slot_id,
        Oop::Null => 0,
        _ => 0,
    }
}

/// JIT `newarray` 指令：创建基本类型数组。
/// `ary_type` 是 newarray 的类型字节（T_INT, T_BYTE 等）。
#[no_mangle]
pub extern "C" fn jit_new_array(ary_type: u8, length: i32) -> u32 {
    if length < 0 {
        exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
        return 0;
    }
    let oop = Oop::new_type_ary(ary_type, length as usize);
    match oop {
        Oop::Ref(slot_id) => slot_id,
        Oop::Null => 0,
        _ => 0,
    }
}

/// JIT `anewarray` 指令：创建引用类型数组。
#[no_mangle]
pub extern "C" fn jit_anewarray(cp_idx: u16, length: i32) -> u32 {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return 0,
    };
    let cls = ctx.method_class.clone();

    if length < 0 {
        exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
        restore_invoke_ctx(Some(ctx));
        return 0;
    }

    let class = match resolve_cp_class(cp_idx, &cls) {
        Some(c) => c,
        None => {
            exception::meet_ex(cls_const::J_CLASS_NOT_FOUND, None);
            restore_invoke_ctx(Some(ctx));
            return 0;
        }
    };

    oop::class::init_class(&class);
    oop::class::init_class_fully(&class);

    // 构建数组类型描述符
    use crate::oop::class::ClassKindType;
    let (name, cl) = {
        let class = class.get_class();
        let t = class.get_class_kind_type();
        let name = match t {
            ClassKindType::Instance | ClassKindType::ObjectAry => {
                let mut v = Vec::with_capacity(class.name.len() + 3);
                v.push(b'[');
                v.push(b'L');
                v.extend_from_slice(class.name.as_slice());
                v.push(b';');
                v
            }
            ClassKindType::TypAry => {
                let mut v = Vec::with_capacity(class.name.len() + 1);
                v.push(b'[');
                v.extend_from_slice(class.name.as_slice());
                v
            }
        };
        (std::sync::Arc::new(name), class.class_loader)
    };

    let result = match require_class3(cl, &name) {
        Some(ary_cls_obj) => {
            oop::class::init_class(&ary_cls_obj);
            oop::class::init_class_fully(&ary_cls_obj);
            let ary = Oop::new_ref_ary(ary_cls_obj, length as usize);
            match ary {
                Oop::Ref(slot_id) => slot_id,
                _ => 0,
            }
        }
        None => {
            restore_invoke_ctx(Some(ctx));
            return 0;
        }
    };

    restore_invoke_ctx(Some(ctx));
    result
}

/// JIT `arraylength` 指令：获取数组长度。
#[no_mangle]
pub extern "C" fn jit_array_length(obj_slot: u32) -> i32 {
    if obj_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return 0;
    }
    oop::with_heap(|heap| {
        let desc = heap.get(obj_slot);
        let guard = desc.read().unwrap();
        match &guard.v {
            oop::RefKind::Array(ary) => ary.elements.len() as i32,
            oop::RefKind::TypeArray(tary) => tary.len() as i32,
            _ => {
                exception::meet_ex(cls_const::J_NPE, None);
                0
            }
        }
    })
}

/// JIT `checkcast` 指令：类型检查。不匹配时抛出 ClassCastException。
#[no_mangle]
pub extern "C" fn jit_checkcast(cp_idx: u16, obj_slot: u32) {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return,
    };
    let cls = ctx.method_class.clone();

    if obj_slot == 0 {
        // null 对象通过 checkcast
        restore_invoke_ctx(Some(ctx));
        return;
    }

    let target_cls = match resolve_cp_class(cp_idx, &cls) {
        Some(c) => c,
        None => {
            exception::meet_ex(cls_const::J_CLASS_NOT_FOUND, None);
            restore_invoke_ctx(Some(ctx));
            return;
        }
    };

    let result = oop::with_heap(|heap| {
        let desc = heap.get(obj_slot);
        let guard = desc.read().unwrap();
        match &guard.v {
            oop::RefKind::Inst(inst) => {
                let obj_cls = inst.class.clone();
                cmp::instance_of(obj_cls.clone(), target_cls.clone())
            }
            oop::RefKind::Array(ary) => {
                let obj_cls = ary.class.clone();
                cmp::instance_of(obj_cls.clone(), target_cls.clone())
            }
            oop::RefKind::Mirror(mirror) => {
                let obj_cls = mirror.target.clone().unwrap();
                let target_name = target_cls.get_class().name.as_slice();
                target_name == b"java/lang/Class"
                    || cmp::instance_of(obj_cls.clone(), target_cls.clone())
            }
            oop::RefKind::TypeArray(tary) => {
                let class_name = tary.class_name();
                match require_class3(None, &class_name) {
                    Some(ary_cls) => cmp::instance_of(ary_cls.clone(), target_cls.clone()),
                    None => false,
                }
            }
            _ => unreachable!(),
        }
    });

    if !result {
        let obj_name = oop::with_heap(|heap| {
            let desc = heap.get(obj_slot);
            let guard = desc.read().unwrap();
            match &guard.v {
                oop::RefKind::Inst(inst) => inst.class.get_class().name.clone(),
                oop::RefKind::Array(ary) => ary.class.get_class().name.clone(),
                _ => std::sync::Arc::new(b"<unknown>".to_vec()),
            }
        });
        let obj_name_str = String::from_utf8_lossy(&obj_name).replace("/", ".");
        let target_name_str =
            String::from_utf8_lossy(&target_cls.get_class().name).replace("/", ".");
        let msg = format!("{} cannot be cast to {}", obj_name_str, target_name_str);
        exception::meet_ex(cls_const::J_CCE, Some(msg));
    }

    restore_invoke_ctx(Some(ctx));
}

/// JIT `instanceof` 指令：类型判断。
/// 返回 1 如果匹配，0 否则。
#[no_mangle]
pub extern "C" fn jit_instanceof(cp_idx: u16, obj_slot: u32) -> i32 {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return 0,
    };
    let cls = ctx.method_class.clone();

    if obj_slot == 0 {
        // null instanceof 任何类型都返回 0
        restore_invoke_ctx(Some(ctx));
        return 0;
    }

    let target_cls = match resolve_cp_class(cp_idx, &cls) {
        Some(c) => c,
        None => {
            restore_invoke_ctx(Some(ctx));
            return 0;
        }
    };

    let result = oop::with_heap(|heap| {
        let desc = heap.get(obj_slot);
        let guard = desc.read().unwrap();
        match &guard.v {
            oop::RefKind::Inst(inst) => {
                let obj_cls = inst.class.clone();
                cmp::instance_of(obj_cls.clone(), target_cls.clone())
            }
            oop::RefKind::Array(ary) => {
                let obj_cls = ary.class.clone();
                cmp::instance_of(obj_cls.clone(), target_cls.clone())
            }
            oop::RefKind::Mirror(mirror) => {
                let obj_cls = mirror.target.clone().unwrap();
                let target_name = target_cls.get_class().name.as_slice();
                target_name == b"java/lang/Class"
                    || cmp::instance_of(obj_cls.clone(), target_cls.clone())
            }
            oop::RefKind::TypeArray(tary) => {
                let class_name = tary.class_name();
                match require_class3(None, &class_name) {
                    Some(ary_cls) => cmp::instance_of(ary_cls.clone(), target_cls.clone()),
                    None => false,
                }
            }
            _ => unreachable!(),
        }
    });

    restore_invoke_ctx(Some(ctx));
    if result {
        1
    } else {
        0
    }
}

// ============================================================
// Field access runtime callouts
// ============================================================

/// JIT `getfield` 指令：读取实例字段。
/// 栈顶是 object slot_id，读取后推入字段值到 stack。
#[no_mangle]
pub extern "C" fn jit_getfield(cp_idx: u16, stack: *mut i32, stack_top: u32) {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return,
    };
    let cls = ctx.method_class.clone();

    // Pop object ref
    let obj_pos = (stack_top - 1) as usize;
    let obj_slot = unsafe { *stack.add(obj_pos) } as u32;

    if obj_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        restore_invoke_ctx(Some(ctx));
        return;
    }

    let fir = match cls.get_cp_field(cp_idx as usize, false) {
        Some(f) => f,
        None => {
            exception::meet_ex(cls_const::J_NSME, None);
            restore_invoke_ctx(Some(ctx));
            return;
        }
    };

    let value_type = fir.field.value_type;
    let v = oop::Class::get_field_value2(obj_slot, fir.offset);

    restore_invoke_ctx(Some(ctx));

    // Push result
    let new_top = stack_top - 1; // pop obj, push value
    unsafe { *stack.add(new_top as usize) = v.extract_int() };
}

/// JIT `putfield` 指令：写入实例字段。
#[no_mangle]
pub extern "C" fn jit_putfield(cp_idx: u16, stack: *mut i32, stack_top: u32) {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return,
    };
    let cls = ctx.method_class.clone();

    let fir = match cls.get_cp_field(cp_idx as usize, false) {
        Some(f) => f,
        None => {
            exception::meet_ex(cls_const::J_NSME, None);
            restore_invoke_ctx(Some(ctx));
            return;
        }
    };

    let value_type = fir.field.value_type;
    let val_pos = (stack_top - 2) as usize;
    let obj_pos = (stack_top - 1) as usize;

    let obj_slot = unsafe { *stack.add(obj_pos) } as u32;

    if obj_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        restore_invoke_ctx(Some(ctx));
        return;
    }

    let v = match value_type {
        crate::oop::ValueType::LONG => {
            let lo = unsafe { *stack.add(val_pos) } as i64;
            let hi = unsafe { *stack.add(val_pos + 1) } as i64;
            Oop::Long(lo | (hi << 32))
        }
        crate::oop::ValueType::DOUBLE => {
            let lo = unsafe { *stack.add(val_pos) } as u64;
            let hi = unsafe { *stack.add(val_pos + 1) } as u64;
            Oop::Double(f64::from_bits(lo | (hi << 32)))
        }
        crate::oop::ValueType::FLOAT => {
            let bits = unsafe { *stack.add(val_pos) } as u32;
            Oop::Float(f32::from_bits(bits))
        }
        crate::oop::ValueType::INT
        | crate::oop::ValueType::BYTE
        | crate::oop::ValueType::SHORT
        | crate::oop::ValueType::CHAR
        | crate::oop::ValueType::BOOLEAN => {
            let val = unsafe { *stack.add(val_pos) };
            Oop::Int(val)
        }
        crate::oop::ValueType::ARRAY | crate::oop::ValueType::OBJECT => {
            let slot = unsafe { *stack.add(val_pos) } as u32;
            Oop::Ref(slot)
        }
        _ => Oop::Int(0),
    };

    oop::Class::put_field_value2(obj_slot, fir.offset, v);
    restore_invoke_ctx(Some(ctx));
}

/// JIT `getstatic` 指令：读取静态字段。
#[no_mangle]
pub extern "C" fn jit_getstatic(cp_idx: u16, stack: *mut i32, stack_top: u32) {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return,
    };
    let cls = ctx.method_class.clone();

    let fir = match cls.get_cp_field(cp_idx as usize, true) {
        Some(f) => f,
        None => {
            exception::meet_ex(cls_const::J_NSME, None);
            restore_invoke_ctx(Some(ctx));
            return;
        }
    };

    let v = fir
        .field
        .class
        .get_class()
        .get_static_field_value(fir.clone());
    restore_invoke_ctx(Some(ctx));

    unsafe { *stack.add(stack_top as usize) = v.extract_int() };
}

/// JIT `putstatic` 指令：写入静态字段。
#[no_mangle]
pub extern "C" fn jit_putstatic(cp_idx: u16, stack: *mut i32, _stack_top: u32) {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return,
    };
    let cls = ctx.method_class.clone();

    let fir = match cls.get_cp_field(cp_idx as usize, true) {
        Some(f) => f,
        None => {
            exception::meet_ex(cls_const::J_NSME, None);
            restore_invoke_ctx(Some(ctx));
            return;
        }
    };

    let value_type = fir.field.value_type;
    let v = match value_type {
        crate::oop::ValueType::LONG => {
            let lo = unsafe { *stack.add(0) } as i64;
            let hi = unsafe { *stack.add(1) } as i64;
            Oop::Long(lo | (hi << 32))
        }
        crate::oop::ValueType::DOUBLE => {
            let lo = unsafe { *stack.add(0) } as u64;
            let hi = unsafe { *stack.add(1) } as u64;
            Oop::Double(f64::from_bits(lo | (hi << 32)))
        }
        crate::oop::ValueType::FLOAT => {
            let bits = unsafe { *stack.add(0) } as u32;
            Oop::Float(f32::from_bits(bits))
        }
        crate::oop::ValueType::INT
        | crate::oop::ValueType::BYTE
        | crate::oop::ValueType::SHORT
        | crate::oop::ValueType::CHAR
        | crate::oop::ValueType::BOOLEAN => {
            let val = unsafe { *stack.add(0) };
            Oop::Int(val)
        }
        crate::oop::ValueType::ARRAY | crate::oop::ValueType::OBJECT => {
            let slot = unsafe { *stack.add(0) } as u32;
            Oop::Ref(slot)
        }
        _ => Oop::Int(0),
    };

    fir.field
        .class
        .get_class()
        .put_static_field_value(fir.clone(), v);
    restore_invoke_ctx(Some(ctx));
}

// ============================================================
// Array load/store runtime callouts
// ============================================================

fn array_load_helper<T>(
    array_slot: u32,
    index: i32,
    extractor: fn(&oop::TypeArrayDesc) -> &[T],
    stack: *mut i32,
    stack_top: u32,
    converter: fn(T) -> i32,
) where
    T: Copy,
{
    if array_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    let result = oop::with_heap(|heap| {
        let desc = heap.get(array_slot);
        let guard = desc.read().unwrap();
        let ary = guard.v.extract_type_array();
        let len = ary.len();
        if index < 0 || index as usize >= len {
            Err(())
        } else {
            let slice = extractor(&ary);
            Ok(slice[index as usize])
        }
    });
    match result {
        Ok(v) => unsafe {
            *stack.add((stack_top - 2) as usize) = converter(v);
        },
        Err(()) => exception::meet_ex(
            cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
            Some(format!("length exceeded, index={}", index)),
        ),
    }
}

#[no_mangle]
pub extern "C" fn jit_iaload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    array_load_helper(
        array_slot,
        index,
        |a| a.extract_ints(),
        stack,
        stack_top,
        |v| v,
    );
}

#[no_mangle]
pub extern "C" fn jit_laload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    if array_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    let result = oop::with_heap(|heap| {
        let desc = heap.get(array_slot);
        let guard = desc.read().unwrap();
        let ary = guard.v.extract_type_array();
        let len = ary.len();
        if index < 0 || index as usize >= len {
            Err(())
        } else {
            Ok(*ary.extract_longs().get(index as usize).unwrap())
        }
    });
    match result {
        Ok(v) => {
            let pos = (stack_top - 2) as usize;
            unsafe {
                *stack.add(pos) = (v & 0xFFFFFFFF) as i32;
                *stack.add(pos + 1) = ((v >> 32) & 0xFFFFFFFF) as i32;
            }
        }
        Err(()) => exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, None),
    }
}

#[no_mangle]
pub extern "C" fn jit_faload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    array_load_helper(
        array_slot,
        index,
        |a| a.extract_floats(),
        stack,
        stack_top,
        |v: f32| v.to_bits() as i32,
    );
}

#[no_mangle]
pub extern "C" fn jit_daload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    if array_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    let result = oop::with_heap(|heap| {
        let desc = heap.get(array_slot);
        let guard = desc.read().unwrap();
        let ary = guard.v.extract_type_array();
        let len = ary.len();
        if index < 0 || index as usize >= len {
            Err(())
        } else {
            Ok(*ary.extract_doubles().get(index as usize).unwrap())
        }
    });
    match result {
        Ok(v) => {
            let bits = v.to_bits();
            let pos = (stack_top - 2) as usize;
            unsafe {
                *stack.add(pos) = (bits & 0xFFFFFFFF) as i32;
                *stack.add(pos + 1) = ((bits >> 32) & 0xFFFFFFFF) as i32;
            }
        }
        Err(()) => exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, None),
    }
}

#[no_mangle]
pub extern "C" fn jit_aaload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    if array_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    let result = oop::with_heap(|heap| {
        let desc = heap.get(array_slot);
        let guard = desc.read().unwrap();
        let ary = guard.v.extract_array();
        let len = ary.elements.len();
        if index < 0 || index as usize >= len {
            None
        } else {
            Some(ary.elements[index as usize].clone())
        }
    });
    match result {
        Some(Oop::Ref(slot)) => unsafe {
            *stack.add((stack_top - 2) as usize) = slot as i32;
        },
        Some(Oop::Null) => unsafe {
            *stack.add((stack_top - 2) as usize) = 0;
        },
        Some(_) => unreachable!(),
        None => exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, None),
    }
}

#[no_mangle]
pub extern "C" fn jit_baload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    if array_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    let result = oop::with_heap(|heap| {
        let desc = heap.get(array_slot);
        let guard = desc.read().unwrap();
        let ary = guard.v.extract_type_array();
        let len = ary.len();
        if index < 0 || index as usize >= len {
            Err(())
        } else {
            let v = match ary {
                oop::TypeArrayDesc::Byte(b) => b[index as usize] as i32,
                oop::TypeArrayDesc::Bool(b) => b[index as usize] as i32,
                _ => unreachable!(),
            };
            Ok(v)
        }
    });
    if let Ok(v) = result {
        unsafe {
            *stack.add((stack_top - 2) as usize) = v;
        }
    } else {
        exception::meet_ex(
            cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
            Some(format!("length exceeded, index={}", index)),
        );
    }
}

#[no_mangle]
pub extern "C" fn jit_caload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    array_load_helper(
        array_slot,
        index,
        |a| a.extract_chars(),
        stack,
        stack_top,
        |v: u16| v as i32,
    );
}

#[no_mangle]
pub extern "C" fn jit_saload(array_slot: u32, index: i32, stack: *mut i32, stack_top: u32) {
    array_load_helper(
        array_slot,
        index,
        |a| a.extract_shorts(),
        stack,
        stack_top,
        |v: i16| v as i32,
    );
}

fn array_store_helper(
    array_slot: u32,
    index: i32,
    mut setter: impl FnMut(&mut oop::TypeArrayDesc) -> Result<(), ()>,
) {
    if array_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    let result = oop::with_heap_mut(|heap| {
        let desc = heap.get(array_slot);
        let mut guard = desc.write().unwrap();
        setter(&mut guard.v.extract_mut_type_array())
    });
    if let Err(()) = result {
        exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, None);
    }
}

#[no_mangle]
pub extern "C" fn jit_iastore(array_slot: u32, index: i32, value: i32) {
    array_store_helper(array_slot, index, |ary| {
        let ints = ary.extract_mut_ints();
        if index < 0 || index as usize >= ints.len() {
            Err(())
        } else {
            ints[index as usize] = value;
            Ok(())
        }
    });
}

#[no_mangle]
pub extern "C" fn jit_lastore(array_slot: u32, index: i32, lo: i32, hi: i32) {
    array_store_helper(array_slot, index, |ary| {
        let longs = ary.extract_mut_longs();
        if index < 0 || index as usize >= longs.len() {
            Err(())
        } else {
            let v = (lo as i64 & 0xFFFFFFFF) | ((hi as i64 & 0xFFFFFFFF) << 32);
            longs[index as usize] = v;
            Ok(())
        }
    });
}

#[no_mangle]
pub extern "C" fn jit_fastore(array_slot: u32, index: i32, bits: i32) {
    array_store_helper(array_slot, index, |ary| {
        let floats = ary.extract_mut_floats();
        if index < 0 || index as usize >= floats.len() {
            Err(())
        } else {
            floats[index as usize] = f32::from_bits(bits as u32);
            Ok(())
        }
    });
}

#[no_mangle]
pub extern "C" fn jit_dastore(array_slot: u32, index: i32, lo: i32, hi: i32) {
    array_store_helper(array_slot, index, |ary| {
        let doubles = ary.extract_mut_doubles();
        if index < 0 || index as usize >= doubles.len() {
            Err(())
        } else {
            let bits = (lo as u64 & 0xFFFFFFFF) | ((hi as u64 & 0xFFFFFFFF) << 32);
            doubles[index as usize] = f64::from_bits(bits);
            Ok(())
        }
    });
}

#[no_mangle]
pub extern "C" fn jit_bastore(array_slot: u32, index: i32, value: i32) {
    array_store_helper(array_slot, index, |ary| match ary {
        oop::TypeArrayDesc::Byte(b) => {
            if index < 0 || index as usize >= b.len() {
                Err(())
            } else {
                b[index as usize] = value as u8;
                Ok(())
            }
        }
        oop::TypeArrayDesc::Bool(b) => {
            if index < 0 || index as usize >= b.len() {
                Err(())
            } else {
                b[index as usize] = value as u8;
                Ok(())
            }
        }
        _ => unreachable!(),
    });
}

#[no_mangle]
pub extern "C" fn jit_castore(array_slot: u32, index: i32, value: i32) {
    array_store_helper(array_slot, index, |ary| {
        let chars = ary.extract_mut_chars();
        if index < 0 || index as usize >= chars.len() {
            Err(())
        } else {
            chars[index as usize] = value as u16;
            Ok(())
        }
    });
}

#[no_mangle]
pub extern "C" fn jit_sastore(array_slot: u32, index: i32, value: i32) {
    array_store_helper(array_slot, index, |ary| {
        let shorts = ary.extract_mut_shorts();
        if index < 0 || index as usize >= shorts.len() {
            Err(())
        } else {
            shorts[index as usize] = value as i16;
            Ok(())
        }
    });
}

#[no_mangle]
pub extern "C" fn jit_aastore(array_slot: u32, index: i32, value_slot: u32) {
    if array_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    let result = oop::with_heap_mut(|heap| {
        let desc = heap.get(array_slot);
        let mut guard = desc.write().unwrap();
        let ary = guard.v.extract_mut_array();
        if index < 0 || index as usize >= ary.elements.len() {
            Err(())
        } else {
            let v = if value_slot == 0 {
                Oop::Null
            } else {
                Oop::Ref(value_slot)
            };
            ary.elements[index as usize] = v;
            Ok(())
        }
    });
    if let Err(()) = result {
        exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, None);
    }
}

// ============================================================
// Monitor runtime callouts
// ============================================================

#[no_mangle]
pub extern "C" fn jit_monitorenter(obj_slot: u32) {
    if obj_slot == 0 {
        exception::meet_ex(cls_const::J_NPE, None);
        return;
    }
    oop::with_heap(|heap| {
        let desc = heap.get(obj_slot);
        let guard = desc.read().unwrap();
        guard.monitor_enter();
    });
}

#[no_mangle]
pub extern "C" fn jit_monitorexit(obj_slot: u32) {
    if obj_slot == 0 {
        return;
    }
    oop::with_heap(|heap| {
        let desc = heap.get(obj_slot);
        let guard = desc.read().unwrap();
        guard.monitor_exit();
    });
}

// ============================================================
// ldc runtime callout
// ============================================================

/// JIT `ldc` / `ldc_w` 指令：从常量池加载常量。
/// 支持：String、Integer、Float、Long、Double、Class
#[no_mangle]
pub extern "C" fn jit_ldc(cp_idx: u16, stack: *mut i32, stack_top: u32) {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return,
    };
    let cls = ctx.method_class.clone();

    let cp = match cls.get_constant_pool() {
        Some(cp) => cp,
        None => {
            restore_invoke_ctx(Some(ctx));
            return;
        }
    };
    use classfile::ConstantPoolType;
    let entry = cp.get(cp_idx as usize);

    let result = match entry {
        Some(ConstantPoolType::String { string_index }) => {
            let s = classfile::constant_pool::get_utf8(&cp, *string_index as usize);
            let oop = util::oop::new_java_lang_string3(s.as_slice());
            match oop {
                Oop::Ref(slot_id) => slot_id as i32,
                Oop::Null => 0,
                _ => 0,
            }
        }
        Some(ConstantPoolType::Integer { v }) => i32::from_be_bytes(v.clone()),
        Some(ConstantPoolType::Float { v }) => i32::from_be_bytes(v.clone()),
        Some(ConstantPoolType::Class { name_index }) => {
            let class_name = classfile::constant_pool::get_utf8(&cp, *name_index as usize);
            let class = resolve_cp_class_from_name(class_name.as_slice());
            match class {
                Some(cls) => {
                    let mirror = cls.get_mirror();
                    match mirror {
                        Oop::Ref(slot_id) => slot_id as i32,
                        _ => 0,
                    }
                }
                None => 0,
            }
        }
        _ => {
            warn!("JIT: ldc unsupported type {:?}", entry);
            0
        }
    };

    restore_invoke_ctx(Some(ctx));
    unsafe { *stack.add(stack_top as usize) = result };
}

/// JIT `ldc2_w` 指令：从常量池加载 long/double 常量。
#[no_mangle]
pub extern "C" fn jit_ldc2_w(cp_idx: u16, stack: *mut i32, stack_top: u32) {
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => return,
    };
    let cls = ctx.method_class.clone();

    let cp = match cls.get_constant_pool() {
        Some(cp) => cp,
        None => {
            restore_invoke_ctx(Some(ctx));
            return;
        }
    };
    use classfile::ConstantPoolType;
    let entry = cp.get(cp_idx as usize);

    match entry {
        Some(ConstantPoolType::Long { v }) => {
            let lo = i32::from_be_bytes([v[4], v[5], v[6], v[7]]);
            let hi = i32::from_be_bytes([v[0], v[1], v[2], v[3]]);
            unsafe {
                *stack.add(stack_top as usize) = lo;
                *stack.add((stack_top + 1) as usize) = hi;
            }
        }
        Some(ConstantPoolType::Double { v }) => {
            let lo = i32::from_be_bytes([v[4], v[5], v[6], v[7]]);
            let hi = i32::from_be_bytes([v[0], v[1], v[2], v[3]]);
            unsafe {
                *stack.add(stack_top as usize) = lo;
                *stack.add((stack_top + 1) as usize) = hi;
            }
        }
        _ => {
            warn!("JIT: ldc2_w unsupported type {:?}", entry);
        }
    }

    restore_invoke_ctx(Some(ctx));
}
