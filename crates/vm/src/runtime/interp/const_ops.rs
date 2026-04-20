use super::Interp;
use crate::oop::{self, Oop};
use crate::runtime;
use crate::runtime::stack::Stack;
use crate::util;
use classfile::constant_pool::get_utf8 as get_cp_utf8;
use classfile::ConstantPoolType;
use std::sync::atomic::Ordering;
use std::sync::RwLockReadGuard;

impl<'a> Interp<'a> {
    pub fn load_constant(&self, pos: usize) {
        match &self.cp[pos] {
            ConstantPoolType::Integer { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_int2(v)
            }
            ConstantPoolType::Float { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_float2(v)
            }
            ConstantPoolType::Long { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_long2(v)
            }
            ConstantPoolType::Double { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_double2(v)
            }
            ConstantPoolType::String { string_index } => {
                let s = get_cp_utf8(&self.cp, *string_index as usize);
                let s = util::oop::new_java_lang_string3(s.as_slice());
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_ref(s, false);
            }
            ConstantPoolType::Class { name_index } => {
                let name = get_cp_utf8(&self.cp, *name_index as usize);
                let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
                let cl = { self.frame.class.get_class().class_loader };
                let class = runtime::require_class3(cl, name.as_bytes()).unwrap();
                oop::class::init_class(&class);
                oop::class::init_class_fully(&class);
                let mirror = { class.get_class().get_mirror() };
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_ref(mirror, false);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn bipush(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let v = (super::read::read_byte(pc, codes) as i8) as i32;
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.push_int(v);
    }

    #[inline]
    pub fn sipush(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let v = super::read::read_i2(pc, codes);
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.push_int(v);
    }

    #[inline]
    pub fn ldc(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let pos = super::read::read_u1(pc, codes);
        self.load_constant(pos);
    }

    #[inline]
    pub fn ldc_w(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let pos = super::read::read_u2(pc, codes);
        self.load_constant(pos);
    }

    #[inline]
    pub fn ldc2_w(&self) {
        self.ldc_w();
    }
}
