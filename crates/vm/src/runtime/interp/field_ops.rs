use super::Interp;
use crate::oop::{self, Class, Oop, ValueType};
use crate::runtime::cmp;
use crate::runtime::exception;
use crate::types::ClassRef;
use classfile::consts as cls_const;
use std::sync::atomic::Ordering;

impl<'a> Interp<'a> {
    pub fn pop_value(&self, vt: ValueType) -> Oop {
        let mut stack = self.frame.area.stack.borrow_mut();
        match vt {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => Oop::new_int(stack.pop_int()),
            ValueType::FLOAT => Oop::new_float(stack.pop_float()),
            ValueType::DOUBLE => Oop::new_double(stack.pop_double()),
            ValueType::LONG => Oop::new_long(stack.pop_long()),
            ValueType::ARRAY | ValueType::OBJECT => stack.pop_ref(),
            _ => unreachable!(),
        }
    }

    pub fn get_field_helper(&self, receiver: Oop, idx: usize, is_static: bool) {
        let cls = self.frame.class.get_class();
        let fir = cls.get_cp_field(idx, is_static).unwrap();
        debug_assert_eq!(fir.field.is_static(), is_static);
        let value_type = fir.field.value_type;
        let v = if is_static {
            let class = fir.field.class.get_class();
            class.get_static_field_value(fir.clone())
        } else {
            let rf = receiver.extract_ref();
            crate::oop::Class::get_field_value2(rf, fir.offset)
        };
        let with_nop = matches!(value_type, ValueType::DOUBLE | ValueType::LONG);
        self.frame.area.stack.borrow_mut().push_ref(v, with_nop);
    }

    pub fn put_field_helper(&self, idx: usize, is_static: bool) {
        let cls = self.frame.class.get_class();
        let fir = cls.get_cp_field(idx, is_static).unwrap();
        debug_assert_eq!(fir.field.is_static(), is_static);
        let value_type = fir.field.value_type;
        let v = self.pop_value(value_type);
        if is_static {
            let mut class = fir.field.class.get_mut_class();
            class.put_static_field_value(fir.clone(), v);
        } else {
            let receiver = { self.frame.area.stack.borrow_mut().pop_ref() };
            match receiver {
                Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
                _ => crate::oop::Class::put_field_value2(receiver.extract_ref(), fir.offset, v),
            }
        }
    }

    #[inline]
    pub fn get_static(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = super::read::read_u2(pc, codes);
        self.get_field_helper(Oop::Null, cp_idx, true);
    }
    #[inline]
    pub fn put_static(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = super::read::read_u2(pc, codes);
        self.put_field_helper(cp_idx, true);
    }
    #[inline]
    pub fn get_field(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = super::read::read_u2(pc, codes);
        let rf = self.frame.area.stack.borrow_mut().pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            _ => self.get_field_helper(rf, idx, false),
        }
    }
    #[inline]
    pub fn put_field(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = super::read::read_u2(pc, codes);
        self.put_field_helper(idx, false);
    }

    pub fn check_cast_helper(&self, is_cast: bool) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = super::read::read_i2(pc, codes);
        let target_cls = super::require_class2(cp_idx as u16, &self.cp).unwrap();
        let obj_rf = self.pop_value(ValueType::OBJECT);
        let obj_rf_clone = obj_rf.clone();
        match obj_rf {
            Oop::Null => {
                let mut stack = self.frame.area.stack.borrow_mut();
                if is_cast {
                    stack.push_ref(obj_rf, false);
                } else {
                    stack.push_const0(false);
                }
            }
            Oop::Ref(slot_id) => {
                oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    match &guard.v {
                        oop::RefKind::Inst(inst) => {
                            let obj_cls = inst.class.clone();
                            let r = cmp::instance_of(obj_cls.clone(), target_cls.clone());
                            if is_cast {
                                self.op_check_cast(r, obj_cls, target_cls);
                            } else {
                                self.op_instance_of(r);
                            }
                        }
                        oop::RefKind::Array(ary) => {
                            let obj_cls = ary.class.clone();
                            let r = cmp::instance_of(obj_cls.clone(), target_cls.clone());
                            if is_cast {
                                self.op_check_cast(r, obj_cls, target_cls);
                            } else {
                                self.op_instance_of(r);
                            }
                        }
                        oop::RefKind::Mirror(mirror) => {
                            let obj_cls = mirror.target.clone().unwrap();
                            let target_name = target_cls.get_class().name.as_slice();
                            let r = target_name == b"java/lang/Class"
                                || cmp::instance_of(obj_cls.clone(), target_cls.clone());
                            if is_cast {
                                self.op_check_cast(r, obj_cls, target_cls);
                            } else {
                                self.op_instance_of(r);
                            }
                        }
                        _ => unreachable!(),
                    }
                });
            }
            _ => unreachable!(),
        }
    }

    pub fn op_check_cast(&self, r: bool, obj_cls: ClassRef, target_cls: ClassRef) {
        if r {
            // object already on stack, nothing to do
        } else {
            let obj_name = { obj_cls.get_class().name.clone() };
            let target_name = { target_cls.get_class().name.clone() };
            let obj_name = String::from_utf8_lossy(obj_name.as_slice()).replace("/", ".");
            let target_name = String::from_utf8_lossy(target_name.as_slice()).replace("/", ".");
            let msg = format!("{} cannot be cast to {}", obj_name, target_name);
            exception::meet_ex(cls_const::J_CCE, Some(msg));
        }
    }

    pub fn op_instance_of(&self, r: bool) {
        let mut stack = self.frame.area.stack.borrow_mut();
        if r {
            stack.push_const1(false);
        } else {
            stack.push_const0(false);
        }
    }

    #[inline]
    pub fn check_cast(&self) {
        self.check_cast_helper(true);
    }
    #[inline]
    pub fn instance_of(&self) {
        self.check_cast_helper(false);
    }
}
