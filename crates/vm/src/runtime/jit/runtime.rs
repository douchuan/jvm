// ============================================================
// JIT invoke 运行时函数
//
// JIT 编译的代码通过 LLVM 外部函数声明调用本模块中的函数。
// ============================================================

use crate::oop::{self, Oop};
use crate::runtime::exception;
use crate::runtime::thread;
use crate::runtime::{self, frame::Frame, JavaCall};
use crate::types::ClassRef;
use classfile::consts as cls_const;
use std::cell::RefCell;
use std::sync::Arc;
use tracing::warn;

/// JIT invoke 的调用方上下文。
#[derive(Clone)]
pub struct JitInvokeCtx {
    /// 当前 JIT 编译方法所属的类（用于常量池方法解析）。
    pub method_class: ClassRef,
}

thread_local! {
    static JIT_INVOKE_CTX: RefCell<Option<JitInvokeCtx>> = const { RefCell::new(None) };
}

/// 设置当前 JIT invoke 上下文。
pub fn set_invoke_ctx(ctx: Option<JitInvokeCtx>) {
    JIT_INVOKE_CTX.with(|cell| {
        *cell.borrow_mut() = ctx;
    });
}

/// 获取当前 JIT invoke 上下文。
pub fn get_invoke_ctx() -> Option<JitInvokeCtx> {
    JIT_INVOKE_CTX.with(|cell| cell.borrow_mut().take())
}

/// 恢复当前 JIT invoke 上下文。
pub fn restore_invoke_ctx(ctx: Option<JitInvokeCtx>) {
    JIT_INVOKE_CTX.with(|cell| {
        *cell.borrow_mut() = ctx;
    });
}

/// 执行 invokevirtual。
pub extern "C" fn jit_invoke_virtual(
    cp_idx: u16,
    _locals: *mut i32,
    stack: *mut i32,
    stack_top: u32,
) {
    invoke_from_jit_stack(cp_idx, stack, stack_top, false, false, false);
}

/// 执行 invokespecial。
pub extern "C" fn jit_invoke_special(
    cp_idx: u16,
    _locals: *mut i32,
    stack: *mut i32,
    stack_top: u32,
) {
    invoke_from_jit_stack(cp_idx, stack, stack_top, false, true, false);
}

/// 执行 invokestatic。
pub extern "C" fn jit_invoke_static(
    cp_idx: u16,
    _locals: *mut i32,
    stack: *mut i32,
    stack_top: u32,
) {
    invoke_from_jit_stack(cp_idx, stack, stack_top, true, true, false);
}

/// 执行 invokeinterface。
pub extern "C" fn jit_invoke_interface(
    cp_idx: u16,
    _locals: *mut i32,
    stack: *mut i32,
    stack_top: u32,
) {
    invoke_from_jit_stack(cp_idx, stack, stack_top, false, false, true);
}

/// 从 JIT 栈构建参数并执行方法调用。
fn invoke_from_jit_stack(
    cp_idx: u16,
    stack: *mut i32,
    stack_top: u32,
    _is_static: bool,
    force_no_resolve: bool,
    is_interface: bool,
) {
    if stack.is_null() {
        warn!("jit invoke: null stack pointer");
        return;
    }

    // 获取当前上下文（消费它，防止递归调用时丢失）
    let ctx = match get_invoke_ctx() {
        Some(c) => c,
        None => {
            warn!("jit invoke: no invoke ctx");
            return;
        }
    };

    let cls = ctx.method_class.clone();

    // 从常量池解析方法
    let mir = match cls.get_cp_method(cp_idx as usize) {
        Some(m) => m,
        None => {
            warn!(
                "jit invoke: method resolution failed at cp index {}",
                cp_idx
            );
            exception::meet_ex(cls_const::J_NSME, None);
            return;
        }
    };

    let sig = &mir.method.signature;

    // 从 JIT 栈构建 Oop 参数（逆序弹出）
    let mut args = Vec::with_capacity(sig.args.len() + 1);
    let mut pos = stack_top as usize;

    for arg_type in sig.args.iter().rev() {
        let (v, slots) = read_jit_stack_value(stack, pos, arg_type);
        if pos < slots {
            warn!("jit invoke: stack underflow");
            return;
        }
        pos -= slots;
        args.push(v);
    }
    args.reverse();

    // 非静态方法需要 this
    let has_this = !mir.method.is_static();
    if has_this {
        if pos == 0 {
            warn!("jit invoke: missing 'this' on stack");
            exception::meet_ex(cls_const::J_NPE, None);
            return;
        }
        let slot_id = unsafe { *stack.add(pos - 1) } as u32;
        let this = Oop::Ref(slot_id);

        if let Oop::Null = this {
            exception::meet_ex(cls_const::J_NPE, None);
            return;
        }

        args.insert(0, this);
    }

    // 虚方法解析
    let target_mir = if !force_no_resolve && has_this && !mir.method.is_final() {
        if let Oop::Ref(slot_id) = &args[0] {
            let sid = *slot_id;
            let mut resolved = None;
            oop::with_heap(|heap| {
                let desc = heap.get(sid);
                let guard = desc.read().unwrap();
                if let oop::RefKind::Inst(inst) = &guard.v {
                    let name = mir.method.name.clone();
                    let desc_bytes = mir.method.desc.clone();
                    let actual_cls = inst.class.get_class();
                    let found = if is_interface {
                        actual_cls.get_interface_method(&name, &desc_bytes).ok()
                    } else {
                        actual_cls.get_virtual_method(&name, &desc_bytes).ok()
                    };
                    resolved = found;
                }
            });
            resolved.unwrap_or(mir)
        } else {
            mir
        }
    } else {
        mir
    };

    // 创建 JavaCall
    let mut jc = JavaCall::new_with_args(target_mir, args);
    jc.is_interface = is_interface;

    // 同步
    sync_enter(&jc.mir, &jc.args);

    let jt = runtime::thread::current_java_thread();

    // 尝试 JIT 路径
    if jc.try_jit_invoke(None) {
        sync_exit(&jc.mir, &jc.args);
        let _ = jt.write().unwrap().frames.pop();
        return;
    }

    // 回退到解释器
    let frame_len = jt.read().unwrap().frames.len();
    if frame_len >= runtime::consts::THREAD_MAX_STACK_FRAMES {
        exception::meet_ex(cls_const::J_SOE, None);
        sync_exit(&jc.mir, &jc.args);
        let _ = jt.write().unwrap().frames.pop();
        return;
    }

    let frame_id = frame_len + 1;
    let frame = Frame::new(jc.mir.clone(), frame_id);
    let frame_ref = Arc::new(std::sync::RwLock::new(Box::new(frame)));

    jt.write().unwrap().frames.push(frame_ref.clone());

    // Build locals
    let max_locals = jc.mir.method.get_max_locals();
    let mut local = runtime::local::Local::new(max_locals);
    let mut slot_pos: usize = 0;
    for v in jc.args.iter() {
        let step = match v {
            Oop::Int(v) => {
                local.set_int(slot_pos, *v);
                1
            }
            Oop::Float(v) => {
                local.set_float(slot_pos, *v);
                1
            }
            Oop::Double(v) => {
                local.set_double(slot_pos, *v);
                2
            }
            Oop::Long(v) => {
                local.set_long(slot_pos, *v);
                2
            }
            _ => {
                local.set_ref(slot_pos, v.clone());
                1
            }
        };
        slot_pos += step;
    }

    let frame_h = frame_ref.try_read().unwrap();
    let mut interp = runtime::Interp::new(frame_h, local);
    interp.run();

    // 写入返回值到调用方的栈
    if !jc.is_return_void && !thread::is_meet_ex() {
        let return_v = {
            let f = frame_ref.try_read().unwrap();
            let borrowed = f.area.return_v.borrow();
            let result = borrowed.clone();
            drop(borrowed);
            drop(f);
            result
        };
        if let Some(rv) = return_v {
            // 通过当前线程栈顶之前的 frame（即调用方）写入
            if let Some(top_frame) = {
                let frames = &jt.read().unwrap().frames;
                if frames.len() >= 2 {
                    Some(frames[frames.len() - 2].clone())
                } else {
                    None
                }
            } {
                let stk = &top_frame.try_read().unwrap().area.stack;
                match &jc.mir.method.signature.retype {
                    classfile::SignatureType::Byte
                    | classfile::SignatureType::Boolean
                    | classfile::SignatureType::Int
                    | classfile::SignatureType::Char
                    | classfile::SignatureType::Short => {
                        stk.borrow_mut().push_int(rv.extract_int())
                    }
                    classfile::SignatureType::Long => stk.borrow_mut().push_long(rv.extract_long()),
                    classfile::SignatureType::Float => {
                        stk.borrow_mut().push_float(rv.extract_float())
                    }
                    classfile::SignatureType::Double => {
                        stk.borrow_mut().push_double(rv.extract_double())
                    }
                    _ => stk.borrow_mut().push_ref(rv, false),
                }
            }
        }
    }

    let _ = jt.write().unwrap().frames.pop();
    sync_exit(&jc.mir, &jc.args);
}

/// 从 JIT i32 栈读取一个值。
fn read_jit_stack_value(
    stack: *mut i32,
    pos: usize,
    sig_type: &classfile::SignatureType,
) -> (Oop, usize) {
    match sig_type {
        classfile::SignatureType::Byte
        | classfile::SignatureType::Boolean
        | classfile::SignatureType::Int
        | classfile::SignatureType::Char
        | classfile::SignatureType::Short => {
            let val = unsafe { *stack.add(pos - 1) };
            (Oop::Int(val), 1)
        }
        classfile::SignatureType::Long => {
            let lo = unsafe { *stack.add(pos - 2) } as i64;
            let hi = unsafe { *stack.add(pos - 1) } as i64;
            let combined = lo | (hi << 32);
            (Oop::Long(combined), 2)
        }
        classfile::SignatureType::Float => {
            let val = unsafe { *stack.add(pos - 1) };
            (Oop::Float(f32::from_bits(val as u32)), 1)
        }
        classfile::SignatureType::Double => {
            let lo = unsafe { *stack.add(pos - 2) } as u64;
            let hi = unsafe { *stack.add(pos - 1) } as u64;
            let bits = lo | (hi << 32);
            (Oop::Double(f64::from_bits(bits)), 2)
        }
        classfile::SignatureType::Object(_, _, _) | classfile::SignatureType::Array(_) => {
            let slot_id = unsafe { *stack.add(pos - 1) } as u32;
            (Oop::Ref(slot_id), 1)
        }
        t => {
            warn!("jit invoke: unsupported arg type {:?}", t);
            (Oop::Int(0), 1)
        }
    }
}

fn sync_enter(mir: &crate::types::MethodIdRef, args: &[Oop]) {
    if mir.method.is_synchronized() {
        if mir.method.is_static() {
            let class = mir.method.class.get_class();
            class.monitor_enter();
        } else if let Some(Oop::Ref(slot_id)) = args.first() {
            let sid = *slot_id;
            oop::with_heap(|heap| {
                let desc = heap.get(sid);
                let guard = desc.read().unwrap();
                guard.monitor_enter();
            });
        }
    }
}

fn sync_exit(mir: &crate::types::MethodIdRef, args: &[Oop]) {
    if mir.method.is_synchronized() {
        if mir.method.is_static() {
            let class = mir.method.class.get_class();
            class.monitor_exit();
        } else if let Some(Oop::Ref(slot_id)) = args.first() {
            let sid = *slot_id;
            oop::with_heap(|heap| {
                let desc = heap.get(sid);
                let guard = desc.read().unwrap();
                guard.monitor_exit();
            });
        }
    }
}
