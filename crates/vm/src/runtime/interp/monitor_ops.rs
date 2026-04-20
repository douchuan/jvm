use super::Interp;
use crate::oop::{self, Oop};
use crate::runtime::exception;
use crate::types::JavaThreadRef;
use classfile::consts as cls_const;

impl<'a> Interp<'a> {
    pub fn set_return(&self, v: Option<Oop>) {
        let mut return_v = self.frame.area.return_v.borrow_mut();
        *return_v = v;
    }

    #[inline]
    pub fn ireturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = Oop::new_int(stack.pop_int());
        drop(stack);
        self.set_return(Some(v));
    }
    #[inline]
    pub fn lreturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = Oop::new_long(stack.pop_long());
        drop(stack);
        self.set_return(Some(v));
    }
    #[inline]
    pub fn freturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = Oop::new_float(stack.pop_float());
        drop(stack);
        self.set_return(Some(v));
    }
    #[inline]
    pub fn dreturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = Oop::new_double(stack.pop_double());
        drop(stack);
        self.set_return(Some(v));
    }
    #[inline]
    pub fn areturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        drop(stack);
        self.set_return(Some(v));
    }
    #[inline]
    pub fn return_void(&self) {
        self.set_return(None);
    }

    #[inline]
    pub fn wide(&mut self) {
        self.op_widen = true;
    }

    #[inline]
    pub fn monitor_enter(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        drop(stack);
        match v {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.monitor_enter();
                });
            }
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn monitor_exit(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        drop(stack);
        match v {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(slot_id) => {
                oop::with_heap(|heap| {
                    let desc = heap.get(slot_id);
                    let guard = desc.read().unwrap();
                    guard.monitor_exit();
                });
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn athrow(&self, jt: JavaThreadRef) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let ex = stack.pop_ref();
        drop(stack);
        jt.write().unwrap().set_ex(ex);
    }
}
