use super::Interp;
use crate::oop::class::ClassKindType;
use crate::oop::{self, Oop};
use crate::runtime;
use crate::runtime::exception;
use classfile::consts as cls_const;
use std::sync::atomic::Ordering;

impl<'a> Interp<'a> {
    pub fn invoke_helper(&self, is_static: bool, idx: usize, force_no_resolve: bool, is_interface: bool) {
        use crate::runtime;
        let cls = self.frame.class.get_class();
        let mir = cls.get_cp_method(idx).unwrap();
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
        warn!("invokedynamic not supported");
        exception::meet_ex(b"java/lang/UnsupportedOperationException", Some("invokedynamic not supported".to_string()));
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
    let down_type = {
        let cls = cls.get_class();
        cls.get_array_down_type().unwrap()
    };
    if idx < lens.len() - 1 {
        let mut elms = Vec::with_capacity(length);
        for i in 0..length {
            let e = new_multi_object_array_helper(down_type.clone(), lens, idx + 1);
            elms.push(e);
        }
        Oop::new_ref_ary2(cls, elms)
    } else {
        Oop::new_ref_ary(cls, length)
    }
}
