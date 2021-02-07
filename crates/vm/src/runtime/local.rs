use crate::oop::{Oop, OopPtr};
use crate::runtime::Slot;
use crate::util;

pub struct Local {
    locals: Vec<Slot>,
}

impl Local {
    pub fn new(size: usize) -> Self {
        let size = size + 1;
        let locals = vec![Slot::Nop; size];
        Self { locals }
    }

    #[inline]
    pub fn set_int(&mut self, pos: usize, v: i32) {
        self.locals[pos] = Slot::I32(v);
    }

    #[inline]
    pub fn set_long(&mut self, pos: usize, v: i64) {
        self.locals[pos] = Slot::I64(v);
    }

    #[inline]
    pub fn set_float(&mut self, pos: usize, v: f32) {
        self.locals[pos] = Slot::F32(v);
    }

    #[inline]
    pub fn set_double(&mut self, pos: usize, v: f64) {
        self.locals[pos] = Slot::F64(v);
    }

    #[inline]
    pub fn set_ref(&mut self, pos: usize, v: Oop) {
        self.locals[pos] = Slot::Ref(v);
    }

    #[inline]
    pub fn get_int(&self, pos: usize) -> i32 {
        match self.locals.get(pos).unwrap() {
            Slot::I32(v) => *v,
            Slot::Ref(v) => OopPtr::java_lang_integer_value(v.extract_ref()),
            t => panic!("Illegal type {:?}", t),
        }
    }

    #[inline]
    pub fn get_long(&self, pos: usize) -> i64 {
        if let Slot::I64(v) = self.locals.get(pos).unwrap() {
            *v
        } else {
            panic!("Illegal type");
        }
    }

    #[inline]
    pub fn get_float(&self, pos: usize) -> f32 {
        if let Slot::F32(v) = self.locals.get(pos).unwrap() {
            *v
        } else {
            panic!("Illegal type");
        }
    }

    #[inline]
    pub fn get_double(&self, pos: usize) -> f64 {
        if let Slot::F64(v) = self.locals.get(pos).unwrap() {
            *v
        } else {
            panic!("Illegal type");
        }
    }

    #[inline]
    pub fn get_ref(&self, pos: usize) -> Oop {
        match self.locals.get(pos) {
            Some(Slot::Ref(v)) => v.clone(),
            t => panic!("Illegal type = {:?}", t),
        }
    }
}
