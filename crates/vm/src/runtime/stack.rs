use crate::oop::{consts, Oop};
use crate::runtime::Slot;

#[derive(Debug)]
pub struct Stack {
    inner: Vec<Slot>,
}

impl Stack {
    pub fn new(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }

    #[inline]
    pub fn push_int(&mut self, v: i32) {
        self.inner.push(Slot::I32(v));
    }

    #[inline]
    pub fn push_int2(&mut self, v: &[u8; 4]) {
        let v = i32::from_be_bytes(*v);
        self.inner.push(Slot::I32(v));
    }

    #[inline]
    pub fn push_float(&mut self, v: f32) {
        self.inner.push(Slot::F32(v));
    }

    #[inline]
    pub fn push_float2(&mut self, v: &[u8; 4]) {
        let v = u32::from_be_bytes(*v);
        let v = f32::from_bits(v);
        self.inner.push(Slot::F32(v));
    }

    #[inline]
    pub fn push_double(&mut self, v: f64) {
        self.push_nop();
        self.inner.push(Slot::F64(v));
    }

    #[inline]
    pub fn push_double2(&mut self, v: &[u8; 8]) {
        let v = u64::from_be_bytes(*v);
        let v = f64::from_bits(v);
        self.push_double(v);
    }

    #[inline]
    pub fn push_long(&mut self, v: i64) {
        self.push_nop();
        self.inner.push(Slot::I64(v));
    }

    #[inline]
    pub fn push_long2(&mut self, v: &[u8; 8]) {
        let v = i64::from_be_bytes(*v);
        self.push_long(v);
    }

    #[inline]
    pub fn push_null(&mut self) {
        self.inner.push(Slot::Ref(consts::get_null()));
    }

    #[inline]
    pub fn push_const_m1(&mut self) {
        self.inner.push(Slot::ConstM1);
    }

    /*
    double & long, with_nop = true
    */
    #[inline]
    pub fn push_const0(&mut self, with_nop: bool) {
        if with_nop {
            self.push_nop();
        }
        self.inner.push(Slot::Const0);
    }

    /*
       double & long, with_nop = true
    */
    #[inline]
    pub fn push_const1(&mut self, with_nop: bool) {
        if with_nop {
            self.push_nop();
        }
        self.inner.push(Slot::Const1);
    }

    #[inline]
    pub fn push_const2(&mut self) {
        self.inner.push(Slot::Const2);
    }

    #[inline]
    pub fn push_const3(&mut self) {
        self.inner.push(Slot::Const3);
    }

    #[inline]
    pub fn push_const4(&mut self) {
        self.inner.push(Slot::Const4);
    }

    #[inline]
    pub fn push_const5(&mut self) {
        self.inner.push(Slot::Const5);
    }

    #[inline]
    pub fn push_ref(&mut self, v: Oop, with_nop: bool) {
        if with_nop {
            self.push_nop();
        }
        self.inner.push(Slot::Ref(v));
    }

    #[inline]
    pub fn pop_int(&mut self) -> i32 {
        match self.inner.pop().unwrap() {
            Slot::ConstM1 => -1,
            Slot::Const0 => 0,
            Slot::Const1 => 1,
            Slot::Const2 => 2,
            Slot::Const3 => 3,
            Slot::Const4 => 4,
            Slot::Const5 => 5,
            Slot::I32(v) => v,
            Slot::Ref(v) => v.extract_int(),
            _ => panic!("Illegal type"),
        }
    }

    #[inline]
    pub fn pop_float(&mut self) -> f32 {
        match self.inner.pop().unwrap() {
            Slot::Const0 => 0.0,
            Slot::Const1 => 1.0,
            Slot::Const2 => 2.0,
            Slot::F32(v) => v,
            Slot::Ref(v) => v.extract_float(),
            _ => panic!("Illegal type"),
        }
    }

    #[inline]
    pub fn pop_double(&mut self) -> f64 {
        match self.inner.pop() {
            Some(v) => {
                self.pop_nop();
                match v {
                    Slot::Const0 => 0.0,
                    Slot::Const1 => 1.0,
                    Slot::F64(v) => v,
                    Slot::Ref(v) => v.extract_double(),
                    _ => panic!("Illegal type"),
                }
            }
            None => panic!("Empty Stack!"),
        }
    }

    #[inline]
    pub fn pop_long(&mut self) -> i64 {
        match self.inner.pop() {
            Some(v) => {
                self.pop_nop();
                match v {
                    Slot::Const0 => 0,
                    Slot::Const1 => 1,
                    Slot::I64(v) => v,
                    Slot::Ref(v) => v.extract_long(),
                    _ => panic!("Illegal type"),
                }
            }
            _ => panic!("Empty Stack!"),
        }
    }

    #[inline]
    pub fn pop_ref(&mut self) -> Oop {
        match self.inner.pop() {
            Some(Slot::Ref(v)) => v,
            t => panic!("Illegal type = {:?}", t),
        }
    }

    pub fn drop_top(&mut self) {
        let _ = self.inner.pop();
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn dup(&mut self) {
        let v = self.inner.pop().unwrap();
        self.inner.push(v.clone());
        self.inner.push(v);
    }

    #[inline]
    pub fn dup_x1(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        self.inner.push(v1.clone());
        self.inner.push(v2);
        self.inner.push(v1);
    }

    #[inline]
    pub fn dup_x2(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        let v3 = self.inner.pop().unwrap();
        self.inner.push(v1.clone());
        self.inner.push(v3);
        self.inner.push(v2);
        self.inner.push(v1);
    }

    #[inline]
    pub fn dup2(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        self.inner.push(v2.clone());
        self.inner.push(v1.clone());
        self.inner.push(v2);
        self.inner.push(v1);
    }

    #[inline]
    pub fn dup2_x1(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        let v3 = self.inner.pop().unwrap();
        self.inner.push(v2.clone());
        self.inner.push(v1.clone());
        self.inner.push(v3);
        self.inner.push(v2);
        self.inner.push(v1);
    }

    #[inline]
    pub fn dup2_x2(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        let v3 = self.inner.pop().unwrap();
        let v4 = self.inner.pop().unwrap();
        self.inner.push(v2.clone());
        self.inner.push(v1.clone());
        self.inner.push(v4);
        self.inner.push(v3);
        self.inner.push(v2);
        self.inner.push(v1);
    }

    #[inline]
    pub fn swap(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        self.inner.push(v1);
        self.inner.push(v2);
    }
}

impl Stack {
    fn push_nop(&mut self) {
        self.inner.push(Slot::Nop);
    }

    fn pop_nop(&mut self) {
        match self.inner.pop() {
            Some(Slot::Nop) => (),
            _ => panic!("Should be Nop!"),
        }
    }
}
