use super::Interp;
use crate::oop::class::ClassKindType;
use crate::oop::{self, Oop};
use crate::runtime;
use crate::runtime::exception;
use classfile::consts as cls_const;
use std::sync::atomic::Ordering;
use tracing::{trace, warn};

impl<'a> Interp<'a> {
    pub fn invoke_helper(
        &self,
        is_static: bool,
        idx: usize,
        force_no_resolve: bool,
        is_interface: bool,
    ) {
        use crate::runtime;
        let cls = self.frame.class.get_class();
        let mir = match cls.get_cp_method(idx) {
            Some(m) => m,
            None => {
                warn!("Method resolution failed at constant pool index {}", idx);
                exception::meet_ex(cls_const::J_NSME, None);
                return;
            }
        };
        let caller = match &mir.method.signature.retype {
            classfile::SignatureType::Void => None,
            _ => Some(&self.frame.area),
        };
        debug_assert_eq!(mir.method.is_static(), is_static);
        if let Ok(mut jc) = runtime::invoke::JavaCall::new(&self.frame.area, mir) {
            jc.is_interface = is_interface;
            jc.invoke(caller, force_no_resolve);
        }
    }

    #[inline]
    pub fn invoke_virtual(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = super::read::read_u2(pc, codes);
        self.invoke_helper(false, idx, false, false);
    }
    #[inline]
    pub fn invoke_special(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = super::read::read_u2(pc, codes);
        self.invoke_helper(false, idx, true, false);
    }
    #[inline]
    pub fn invoke_static(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = super::read::read_u2(pc, codes);
        self.invoke_helper(true, idx, true, false);
    }
    #[inline]
    pub fn invoke_interface(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = super::read::read_u2(pc, codes);
        let _count = super::read::read_u1(pc, codes);
        let zero = super::read::read_u1(pc, codes);
        if zero != 0 {
            warn!("interpreter: invalid invokeinterface: the value of the fourth operand byte must always be zero.");
        }
        self.invoke_helper(false, cp_idx, false, true);
    }
    #[inline]
    pub fn invoke_dynamic(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = super::read::read_u2(pc, codes);

        let cls_name = String::from_utf8_lossy(self.frame.class.get_class().name.as_slice());
        trace!("invokedynamic in class: {}", cls_name);

        // Step 1: Resolve CONSTANT_InvokeDynamic_info
        let (bootstrap_idx, name, desc) = {
            let cp_item = self.cp.get(cp_idx);
            match cp_item {
                Some(classfile::ConstantPoolType::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                }) => {
                    let (name, desc) = classfile::constant_pool::get_name_and_type(
                        &self.cp,
                        *name_and_type_index as usize,
                    );
                    (*bootstrap_method_attr_index, name.clone(), desc.clone())
                }
                _ => unreachable!("Expected InvokeDynamic at cp index {}", cp_idx),
            }
        };

        // Step 2: Get BootstrapMethod
        let class = self.frame.class.get_class();
        let bootstrap_methods = match class.get_bootstrap_methods() {
            Some(bsm) => bsm,
            None => {
                exception::meet_ex(
                    b"java/lang/BootstrapMethodError",
                    Some("No BootstrapMethods attribute".to_string()),
                );
                return;
            }
        };

        let bsm = match bootstrap_methods.get(bootstrap_idx as usize) {
            Some(b) => b,
            None => {
                exception::meet_ex(
                    b"java/lang/BootstrapMethodError",
                    Some(format!(
                        "BootstrapMethod index {} out of range (count={})",
                        bootstrap_idx,
                        bootstrap_methods.len()
                    )),
                );
                return;
            }
        };

        // Step 3: Resolve bootstrap method identity
        let (ref_kind, ref_index) =
            classfile::constant_pool::get_method_handle_ref(&self.cp, bsm.method_ref as usize);
        let (target_class, target_name, _target_desc) =
            classfile::constant_pool::get_method_handle_target(&self.cp, ref_index);

        trace!(
            "invokedynamic: bootstrap={} method={}:{} ref_kind={}",
            bootstrap_idx,
            String::from_utf8_lossy(target_class),
            String::from_utf8_lossy(target_name),
            ref_kind,
        );

        // Step 4: Dispatch based on bootstrap method identity
        if target_class.as_slice() == b"java/lang/invoke/StringConcatFactory"
            && target_name.as_slice() == b"makeConcatWithConstants"
        {
            handle_string_concat_factory(&self.frame.area, &self.cp, bsm, &name, &desc);
        } else if target_class.as_slice() == b"java/lang/invoke/LambdaMetafactory"
            && target_name.as_slice() == b"metafactory"
        {
            handle_lambda_metafactory(&self.frame.area, &self.cp, bsm, &name, &desc);
        } else {
            warn!(
                "invokedynamic: unsupported bootstrap method {}::{}",
                String::from_utf8_lossy(target_class),
                String::from_utf8_lossy(target_name),
            );
            exception::meet_ex(
                b"java/lang/UnsupportedOperationException",
                Some(format!(
                    "invokedynamic: unsupported bootstrap {}::{}",
                    String::from_utf8_lossy(target_class),
                    String::from_utf8_lossy(target_name),
                )),
            );
        }
    }

    #[inline]
    pub fn new_(&self) {
        use crate::runtime;
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = super::read::read_u2(pc, codes);
        let class = match runtime::require_class2(idx as u16, &self.cp) {
            Some(class) => {
                oop::class::init_class(&class);
                oop::class::init_class_fully(&class);
                class
            }
            None => unreachable!("Cannot get class info from constant pool"),
        };
        let v = Oop::new_inst(class);
        self.frame.area.stack.borrow_mut().push_ref(v, false);
    }

    #[inline]
    pub fn new_array(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let ary_type = super::read::read_byte(pc, codes);
        let mut stack = self.frame.area.stack.borrow_mut();
        let len = stack.pop_int();
        if len < 0 {
            drop(stack);
            exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
        } else {
            let ary = Oop::new_type_ary(ary_type, len as usize);
            stack.push_ref(ary, false);
        }
    }

    #[inline]
    pub fn anew_array(&self) {
        use crate::runtime;
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = super::read::read_i2(pc, codes);
        let mut stack = self.frame.area.stack.borrow_mut();
        let length = stack.pop_int();
        drop(stack);
        if length < 0 {
            exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
        } else {
            let class = match runtime::require_class2(cp_idx as u16, &self.cp) {
                Some(class) => class,
                None => panic!("Cannot get class info from constant pool"),
            };
            oop::class::init_class(&class);
            oop::class::init_class_fully(&class);
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
            match runtime::require_class(cl, &name) {
                Some(ary_cls_obj) => {
                    oop::class::init_class(&ary_cls_obj);
                    oop::class::init_class_fully(&ary_cls_obj);
                    let ary = Oop::new_ref_ary(ary_cls_obj, length as usize);
                    self.frame.area.stack.borrow_mut().push_ref(ary, false);
                }
                None => unreachable!(),
            }
        }
    }

    #[inline]
    pub fn array_length(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        drop(stack);
        match v {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    match &guard.v {
                        oop::RefKind::Array(ary) => ary.elements.len() as i32,
                        oop::RefKind::TypeArray(ary) => ary.len() as i32,
                        _ => unreachable!(),
                    }
                });
                self.frame.area.stack.borrow_mut().push_int(len);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn multi_anew_array(&self) {
        use crate::runtime::require_class2;
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = super::read::read_u2(pc, codes);
        let dimension = super::read::read_u1(pc, codes);
        let mut lens = Vec::new();
        let mut stack = self.frame.area.stack.borrow_mut();
        for _ in 0..dimension {
            lens.push(stack.pop_int());
        }
        drop(stack);
        let cls = require_class2(cp_idx as u16, &self.cp).unwrap();
        let ary = new_multi_object_array_helper(cls, &lens, 0);
        self.frame.area.stack.borrow_mut().push_ref(ary, false);
    }
}

fn new_multi_object_array_helper(cls: crate::types::ClassRef, lens: &[i32], idx: usize) -> Oop {
    let length = lens[idx] as usize;
    let cls_obj = cls.get_class();
    match cls_obj.get_array_down_type() {
        Some(down_type) => {
            // Has a down-type: either ObjectArray or multi-dim TypeArray
            if idx < lens.len() - 1 {
                let mut elms = Vec::with_capacity(length);
                for _ in 0..length {
                    let e = new_multi_object_array_helper(down_type.clone(), lens, idx + 1);
                    elms.push(e);
                }
                Oop::new_ref_ary2(cls, elms)
            } else {
                // Innermost: down_type is the element type
                let down_cls = down_type.get_class();
                if down_cls.is_array() {
                    // Multi-dim: e.g., innermost of int[][][] is [I
                    Oop::new_ref_ary(down_type, length)
                } else {
                    // Instance class: e.g., String[]
                    Oop::new_ref_ary(cls, length)
                }
            }
        }
        None => {
            // No down-type: single-dim primitive array like [I
            // Create the actual primitive type array
            let value_type = cls_obj.get_array_value_type().unwrap();
            // Map ValueType descriptor byte to TypeArrayEnum internal code
            let type_byte: &[u8] = value_type.into();
            let type_code = match type_byte[0] {
                b'B' => 8,
                b'Z' => 4,
                b'C' => 5,
                b'S' => 9,
                b'I' => 10,
                b'J' => 11,
                b'F' => 6,
                b'D' => 7,
                _ => unreachable!("unknown primitive type: {:?}", type_byte),
            };
            Oop::new_type_ary(type_code, length)
        }
    }
}

fn handle_string_concat_factory(
    caller: &runtime::DataArea,
    cp: &classfile::ConstantPool,
    bsm: &classfile::attributes::BootstrapMethod,
    _method_name: &[u8],
    method_desc: &[u8],
) {
    // Step 1: Parse argument types from method descriptor
    let arg_types = parse_method_type_args(method_desc);

    // Step 2: Pop arguments from operand stack (reverse order)
    let mut args: Vec<Oop> = Vec::with_capacity(arg_types.len());
    for ty in arg_types.iter().rev() {
        let v = match ty {
            classfile::SignatureType::Int
            | classfile::SignatureType::Byte
            | classfile::SignatureType::Boolean
            | classfile::SignatureType::Char
            | classfile::SignatureType::Short => Oop::Int(caller.stack.borrow_mut().pop_int()),
            classfile::SignatureType::Long => Oop::Long(caller.stack.borrow_mut().pop_long()),
            classfile::SignatureType::Float => Oop::Float(caller.stack.borrow_mut().pop_float()),
            classfile::SignatureType::Double => Oop::Double(caller.stack.borrow_mut().pop_double()),
            classfile::SignatureType::Object(_, _, _) | classfile::SignatureType::Array(_) => {
                caller.stack.borrow_mut().pop_ref()
            }
            _ => unreachable!("Unsupported invokedynamic arg type: {:?}", ty),
        };
        args.push(v);
    }
    args.reverse();

    // Step 3: Extract format string from bootstrap args
    // For javac-compiled StringConcatFactory, the first arg is the format string constant.
    let format_string = if !bsm.args.is_empty() {
        classfile::constant_pool::get_string(cp, bsm.args[0] as usize)
    } else {
        String::new()
    };

    // Step 4: Build result string by replacing \u0001 placeholders
    let result = build_concat_string(&format_string, &args);

    // Step 5: Create java.lang.String and push result
    let string_oop = crate::util::oop::new_java_lang_string_direct(&result);
    caller.stack.borrow_mut().push_ref(string_oop, false);
}

fn parse_method_type_args(desc: &[u8]) -> Vec<classfile::SignatureType> {
    use classfile::SignatureType;
    use std::sync::Arc;

    let mut args = Vec::new();
    let mut i = 1; // skip '('
    while i < desc.len() && desc[i] != b')' {
        let (ty, consumed) = parse_one_type(desc, i);
        args.push(ty);
        i += consumed;
    }
    args
}

fn parse_one_type(desc: &[u8], i: usize) -> (classfile::SignatureType, usize) {
    use classfile::SignatureType;
    use std::sync::Arc;

    match desc[i] {
        b'B' => (SignatureType::Byte, 1),
        b'C' => (SignatureType::Char, 1),
        b'D' => (SignatureType::Double, 1),
        b'F' => (SignatureType::Float, 1),
        b'I' => (SignatureType::Int, 1),
        b'J' => (SignatureType::Long, 1),
        b'S' => (SignatureType::Short, 1),
        b'Z' => (SignatureType::Boolean, 1),
        b'[' => {
            let mut end = i + 1;
            if end < desc.len() && desc[end] == b'L' {
                while end < desc.len() && desc[end] != b';' {
                    end += 1;
                }
                end += 1;
            } else {
                end += 1;
            }
            (
                SignatureType::Array(Arc::new(desc[i..end].to_vec())),
                end - i,
            )
        }
        b'L' => {
            let mut end = i + 1;
            while end < desc.len() && desc[end] != b';' {
                end += 1;
            }
            (
                SignatureType::Object(Arc::new(desc[i..=end].to_vec()), None, None),
                end - i + 1,
            )
        }
        _ => unreachable!("Invalid type descriptor at {}: {:?}", i, &desc[i..]),
    }
}

fn build_concat_string(format: &str, args: &[Oop]) -> String {
    let mut result = String::new();
    let mut arg_idx = 0;
    let placeholder = '\u{0001}';

    for ch in format.chars() {
        if ch == placeholder {
            if arg_idx < args.len() {
                result.push_str(&oop_to_string(&args[arg_idx]));
                arg_idx += 1;
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn oop_to_string(v: &Oop) -> String {
    match v {
        Oop::Int(val) => val.to_string(),
        Oop::Long(val) => val.to_string(),
        Oop::Float(val) => {
            if val.is_infinite() {
                if *val > 0.0 {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else if val.is_nan() {
                "NaN".to_string()
            } else {
                val.to_string()
            }
        }
        Oop::Double(val) => {
            if val.is_infinite() {
                if *val > 0.0 {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else if val.is_nan() {
                "NaN".to_string()
            } else {
                val.to_string()
            }
        }
        Oop::Ref(slot_id) => {
            if Oop::is_java_lang_string(*slot_id) {
                Oop::java_lang_string(*slot_id)
            } else {
                format!("<object:{}>", slot_id)
            }
        }
        Oop::Null => "null".to_string(),
        Oop::ConstUtf8(bytes) => String::from_utf8_lossy(bytes).to_string(),
    }
}

/// Handle LambdaMetafactory.metafactory invokedynamic calls.
/// Creates a minimal lambda proxy object that implements the SAM (Single Abstract Method)
/// functional interface. The proxy returns default values (null/0/false) when invoked.
fn handle_lambda_metafactory(
    caller: &runtime::DataArea,
    cp: &classfile::ConstantPool,
    bsm: &classfile::attributes::BootstrapMethod,
    _method_name: &[u8],
    method_desc: &[u8],
) {
    // LambdaMetafactory.metafactory bootstrap args:
    //   args[0]: MethodType (samMethodType - the functional interface method signature)
    //   args[1]: MethodHandle (implMethod - the actual implementation method)
    //   args[2]: MethodType (instantiatedMethodType)
    //
    // The invokedynamic descriptor tells us the return type = the functional interface type.
    // E.g.: (Ljava/util/function/Supplier;)Ljava/util/function/Supplier;
    //       -> return type is Ljava/util/function/Supplier;

    // Extract the SAM interface type from the method descriptor return type
    let sam_interface_desc = {
        let mut i = 0;
        while i < method_desc.len() && method_desc[i] != b')' {
            i += 1;
        }
        // skip ')'
        i += 1;
        if i < method_desc.len() {
            &method_desc[i..]
        } else {
            b"Ljava/lang/Object;"
        }
    };

    // Build the internal class name from the descriptor
    let class_name = if sam_interface_desc.starts_with(b"L") {
        // Ljava/util/function/Supplier; -> java/util/function/Supplier
        &sam_interface_desc[1..sam_interface_desc.len() - 1]
    } else {
        b"java/lang/Object"
    };

    // Try to load the SAM interface class
    let proxy = match runtime::require_class3(None, class_name) {
        Some(cls_ref) => {
            oop::class::init_class(&cls_ref);
            oop::class::init_class_fully(&cls_ref);
            Oop::new_inst(cls_ref)
        }
        None => {
            warn!(
                "LambdaMetafactory: cannot load SAM interface {:?}",
                String::from_utf8_lossy(class_name)
            );
            Oop::Null
        }
    };

    caller.stack.borrow_mut().push_ref(proxy, false);
}
