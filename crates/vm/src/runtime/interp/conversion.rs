use super::Interp;

impl<'a> Interp<'a> {
    #[inline]
    pub fn i2l(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.frame.area.stack.borrow_mut().push_long(v as i64);
    }
    #[inline]
    pub fn i2f(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.frame.area.stack.borrow_mut().push_float(v as f32);
    }
    #[inline]
    pub fn i2d(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.frame.area.stack.borrow_mut().push_double(v as f64);
    }
    #[inline]
    pub fn l2i(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.frame.area.stack.borrow_mut().push_int(v as i32);
    }
    #[inline]
    pub fn l2f(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.frame.area.stack.borrow_mut().push_float(v as f32);
    }
    #[inline]
    pub fn l2d(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.frame.area.stack.borrow_mut().push_double(v as f64);
    }
    #[inline]
    pub fn f2i(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        if v.is_nan() {
            stack.push_int(0);
        } else if v.is_infinite() {
            stack.push_int(if v.is_sign_positive() {
                i32::MAX
            } else {
                i32::MIN
            });
        } else {
            stack.push_int(v as i32);
        }
    }
    #[inline]
    pub fn f2l(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        if v.is_nan() {
            stack.push_long(0);
        } else if v.is_infinite() {
            stack.push_long(if v.is_sign_positive() {
                i64::MAX
            } else {
                i64::MIN
            });
        } else {
            stack.push_long(v as i64);
        }
    }
    #[inline]
    pub fn f2d(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_float();
        self.frame.area.stack.borrow_mut().push_double(v as f64);
    }
    #[inline]
    pub fn d2i(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        if v.is_nan() {
            stack.push_int(0);
        } else if v.is_infinite() {
            stack.push_int(if v.is_sign_positive() {
                i32::MAX
            } else {
                i32::MIN
            });
        } else {
            stack.push_int(v as i32);
        }
    }
    #[inline]
    pub fn d2l(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        if v.is_nan() {
            stack.push_long(0);
        } else if v.is_infinite() {
            stack.push_long(if v.is_sign_positive() {
                i64::MAX
            } else {
                i64::MIN
            });
        } else {
            stack.push_long(v as i64);
        }
    }
    #[inline]
    pub fn d2f(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_double();
        self.frame.area.stack.borrow_mut().push_float(v as f32);
    }
    #[inline]
    pub fn i2b(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.frame.area.stack.borrow_mut().push_int(v as i8 as i32);
    }
    #[inline]
    pub fn i2c(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.frame
            .area
            .stack
            .borrow_mut()
            .push_int((v as u16) as i32);
    }
    #[inline]
    pub fn i2s(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.frame.area.stack.borrow_mut().push_int(v as i16 as i32);
    }
}
