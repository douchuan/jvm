use super::Interp;
use crate::runtime::exception;
use classfile::consts as cls_const;
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Sub};

impl<'a> Interp<'a> {
    #[inline]
    pub fn iadd(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        stack.push_int(v1.wrapping_add(v2));
    }
    #[inline]
    pub fn ladd(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        stack.push_long(v1.wrapping_add(v2));
    }
    #[inline]
    pub fn fadd(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_float();
        let v1 = stack.pop_float();
        stack.push_float(v1.add(v2));
    }
    #[inline]
    pub fn dadd(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_double();
        let v1 = stack.pop_double();
        stack.push_double(v1.add(v2));
    }

    #[inline]
    pub fn isub(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        stack.push_int(v1.wrapping_sub(v2));
    }
    #[inline]
    pub fn lsub(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        stack.push_long(v1.wrapping_sub(v2));
    }
    #[inline]
    pub fn fsub(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_float();
        let v1 = stack.pop_float();
        stack.push_float(v1.sub(v2));
    }
    #[inline]
    pub fn dsub(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_double();
        let v1 = stack.pop_double();
        stack.push_double(v1.sub(v2));
    }

    #[inline]
    pub fn imul(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        stack.push_int(v1.wrapping_mul(v2));
    }
    #[inline]
    pub fn lmul(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        stack.push_long(v1.wrapping_mul(v2));
    }
    #[inline]
    pub fn fmul(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_float();
        let v1 = stack.pop_float();
        stack.push_float(v1.mul(v2));
    }
    #[inline]
    pub fn dmul(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_double();
        let v1 = stack.pop_double();
        stack.push_double(v1.mul(v2));
    }

    #[inline]
    pub fn idiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_int(v1 / v2);
        }
    }
    #[inline]
    pub fn ldiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_long(v1 / v2);
        }
    }
    #[inline]
    pub fn fdiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_float();
        let v1 = stack.pop_float();
        if v2 == 0.0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_float(v1 / v2);
        }
    }
    #[inline]
    pub fn ddiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_double();
        let v1 = stack.pop_double();
        if v2 == 0.0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_double(v1 / v2);
        }
    }

    #[inline]
    pub fn irem(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_int(v1 - (v1 / v2) * v2);
        }
    }
    #[inline]
    pub fn lrem(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_long(v1 - (v1 / v2) * v2);
        }
    }
    #[inline]
    pub fn frem(&self) {
        panic!("Use of deprecated instruction frem, please check your Java compiler");
    }
    #[inline]
    pub fn drem(&self) {
        panic!("Use of deprecated instruction drem, please check your Java compiler");
    }

    #[inline]
    pub fn ineg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        stack.push_int(-v);
    }
    #[inline]
    pub fn lneg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        stack.push_long(-v);
    }
    #[inline]
    pub fn fneg(&self) {
        panic!("Use of deprecated instruction fneg, please check your Java compiler");
    }
    #[inline]
    pub fn dneg(&self) {
        panic!("Use of deprecated instruction dneg, please check your Java compiler");
    }

    #[inline]
    pub fn ishl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        let s = v2 & 0x1F;
        stack.push_int(v1 << s);
    }
    #[inline]
    pub fn lshl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        stack.push_long(v1 << s);
    }
    #[inline]
    pub fn ishr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        let s = v2 & 0x1F;
        stack.push_int(v1 >> s);
    }
    #[inline]
    pub fn lshr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        stack.push_long(v1 >> s);
    }
    #[inline]
    pub fn iushr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int() as u32;
        let s = (v2 & 0x1F) as u32;
        stack.push_int((v1 >> s) as i32);
    }
    #[inline]
    pub fn lushr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_long() as u64;
        let s = (v2 & 0x3F) as u64;
        stack.push_long((v1 >> s) as i64);
    }

    #[inline]
    pub fn iand(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        stack.push_int(v1.bitand(v2));
    }
    #[inline]
    pub fn land(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        stack.push_long(v1.bitand(v2));
    }
    #[inline]
    pub fn ior(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        stack.push_int(v1.bitor(v2));
    }
    #[inline]
    pub fn lor(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        stack.push_long(v1.bitor(v2));
    }
    #[inline]
    pub fn ixor(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        stack.push_int(v1.bitxor(v2));
    }
    #[inline]
    pub fn lxor(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        stack.push_long(v1.bitxor(v2));
    }

    pub fn iinc(&mut self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let op_widen = self.op_widen;
        let pos;
        let factor;
        if op_widen {
            self.op_widen = false;
            pos = super::read::read_u2(pc, codes);
            factor = (super::read::read_u2(pc, codes) as i16) as i32
        } else {
            pos = super::read::read_u1(pc, codes);
            factor = (super::read::read_byte(pc, codes) as i8) as i32
        };
        let v = self.local.borrow().get_int(pos);
        let v = v.wrapping_add(factor);
        self.local.borrow_mut().set_int(pos, v);
    }
}
