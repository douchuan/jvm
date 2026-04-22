use super::Interp;
use crate::oop::{self, Oop, TypeArrayDesc};
use crate::runtime::exception;
use crate::runtime::thread;
use classfile::consts as cls_const;

impl<'a> Interp<'a> {
    #[inline]
    pub fn iaload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let (ary, len) = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_type_array().extract_ints();
                    (ary.clone(), ary.len())
                });
                if pos < 0 || pos as usize >= len {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length is {}, but index is {}", len, pos)),
                    );
                } else {
                    stack.push_int(ary[pos as usize]);
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn laload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let (ary, len) = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_type_array().extract_longs();
                    (ary.clone(), ary.len())
                });
                if pos < 0 || pos as usize >= len {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length is {}, but index is {}", len, pos)),
                    );
                } else {
                    stack.push_long(ary[pos as usize]);
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn faload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let (ary, len) = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_type_array().extract_floats();
                    (ary.clone(), ary.len())
                });
                if pos < 0 || pos as usize >= len {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length is {}, but index is {}", len, pos)),
                    );
                } else {
                    stack.push_float(ary[pos as usize]);
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn daload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let (ary, len) = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_type_array().extract_doubles();
                    (ary.clone(), ary.len())
                });
                if pos < 0 || pos as usize >= len {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length is {}, but index is {}", len, pos)),
                    );
                } else {
                    stack.push_double(ary[pos as usize]);
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn aaload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let (elements, len) = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_array();
                    (ary.elements.clone(), ary.elements.len())
                });
                if pos < 0 || pos as usize >= len {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length is {}, but index is {}", len, pos)),
                    );
                } else {
                    stack.push_ref(elements[pos as usize].clone(), false);
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn baload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let result = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_type_array();
                    let len = ary.len();
                    if pos < 0 || pos as usize >= len {
                        Err(())
                    } else {
                        match ary {
                            TypeArrayDesc::Byte(ary) => Ok(ary[pos as usize] as i32),
                            TypeArrayDesc::Bool(ary) => Ok(ary[pos as usize] as i32),
                            t => unreachable!("t = {:?}", t),
                        }
                    }
                });
                if let Err(()) = result {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length exceeded, index={}", pos)),
                    );
                } else if let Ok(v) = result {
                    stack.push_int(v);
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn caload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let (ary, len) = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_type_array().extract_chars();
                    (ary.clone(), ary.len())
                });
                if pos < 0 || pos as usize >= len {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length is {}, but index is {}", len, pos)),
                    );
                } else {
                    stack.push_int(ary[pos as usize] as i32);
                }
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn saload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let (ary, len) = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    let ary = guard.v.extract_type_array().extract_shorts();
                    (ary.clone(), ary.len())
                });
                if pos < 0 || pos as usize >= len {
                    exception::meet_ex(
                        cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                        Some(format!("length is {}, but index is {}", len, pos)),
                    );
                } else {
                    stack.push_int(ary[pos as usize] as i32);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn bastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_type_array().len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                let v = v as u8;
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    match guard.v.extract_mut_type_array() {
                        TypeArrayDesc::Byte(ary) => ary[pos as usize] = v,
                        TypeArrayDesc::Bool(ary) => ary[pos as usize] = v,
                        _ => unreachable!(),
                    }
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn castore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_type_array().len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                let v = v as u16;
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    let ary = guard.v.extract_mut_type_array().extract_mut_chars();
                    ary[pos as usize] = v;
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn sastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_type_array().len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                let v = v as i16;
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    let ary = guard.v.extract_mut_type_array().extract_mut_shorts();
                    ary[pos as usize] = v;
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn iastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_type_array().len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    let ary = guard.v.extract_mut_type_array().extract_mut_ints();
                    ary[pos as usize] = v;
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn lastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_type_array().len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    let ary = guard.v.extract_mut_type_array().extract_mut_longs();
                    ary[pos as usize] = v;
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn fastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_type_array().len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    let ary = guard.v.extract_mut_type_array().extract_mut_floats();
                    ary[pos as usize] = v;
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn dastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_type_array().len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    let ary = guard.v.extract_mut_type_array().extract_mut_doubles();
                    ary[pos as usize] = v;
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn aastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        let pos = stack.pop_int();
        let ary_rf = stack.pop_ref();
        drop(stack);
        match ary_rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                let len = oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.v.extract_array().elements.len()
                });
                check_bounds(len, pos);
                if thread::is_meet_ex() {
                    return;
                }
                oop::with_heap_mut(|heap| {
                    let desc = heap.get(slot_id);
                    let mut guard = desc.write().unwrap();
                    let ary = guard.v.extract_mut_array();
                    ary.elements[pos as usize] = v;
                });
            }
            _ => unreachable!(),
        }
    }
}

fn check_bounds(len: usize, pos: i32) {
    if pos < 0 || pos as usize >= len {
        exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, None);
    }
}
