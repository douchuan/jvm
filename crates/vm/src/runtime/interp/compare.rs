use super::Interp;
use std::cmp::Ordering as CmpOrdering;

impl<'a> Interp<'a> {
    #[inline]
    pub fn lcmp(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_long();
        let v2 = stack.pop_long();
        let v = match v1.cmp(&v2) {
            CmpOrdering::Greater => -1,
            CmpOrdering::Less => 1,
            CmpOrdering::Equal => 0,
        };
        stack.push_int(v);
    }
    #[inline]
    pub fn fcmpl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_float();
        let v2 = stack.pop_float();
        let v = if v1.is_nan() || v2.is_nan() {
            -1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };
        stack.push_int(v);
    }
    #[inline]
    pub fn fcmpg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_float();
        let v2 = stack.pop_float();
        let v = if v1.is_nan() || v2.is_nan() {
            1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };
        stack.push_int(v);
    }
    #[inline]
    pub fn dcmpl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_double();
        let v2 = stack.pop_double();
        let v = if v1.is_nan() || v2.is_nan() {
            -1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };
        stack.push_int(v);
    }
    #[inline]
    pub fn dcmpg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_double();
        let v2 = stack.pop_double();
        if v1.is_nan() || v2.is_nan() {
            stack.push_int(1);
        } else if v1 > v2 {
            stack.push_int(-1);
        } else if v1 < v2 {
            stack.push_int(1);
        } else {
            stack.push_int(0);
        }
    }
}
