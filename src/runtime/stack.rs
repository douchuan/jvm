use crate::classfile::constant_pool::ConstantType;
use crate::classfile::method_info::MethodInfo;
use crate::classfile::types::*;
use crate::classfile::ClassFile;
use crate::oop::{consts, OopDesc, OopRef};
use crate::runtime::Slot;
use bytes::{BigEndian, Bytes};
use std::sync::Arc;

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

    pub fn push_int(&mut self, i: i32) {
        let v = i.to_be_bytes();
        self.push_primitive2(v);
    }

    pub fn push_int2(&mut self, v: [u8; 4]) {
        self.push_primitive2(v)
    }

    pub fn push_float(&mut self, f: f32) {
        let v = f.to_bits().to_be_bytes();
        self.push_primitive2(v);
    }

    pub fn push_float2(&mut self, v: [u8; 4]) {
        self.push_primitive2(v);
    }

    pub fn push_double(&mut self, d: f64) {
        let v = d.to_bits().to_be_bytes();
        self.push_double2(v);
    }

    pub fn push_double2(&mut self, v: [u8; 8]) {
        self.push_nop();
        self.push_primitive3(v);
    }

    pub fn push_long(&mut self, l: i64) {
        let v = l.to_be_bytes();
        self.push_long2(v);
    }

    pub fn push_long2(&mut self, v: [u8; 8]) {
        self.push_nop();
        self.push_primitive3(v);
    }

    pub fn push_null(&mut self) {
        self.inner.push(Slot::Ref(consts::get_null()));
    }

    pub fn push_const_m1(&mut self) {
        self.inner.push(Slot::ConstM1);
    }

    /*
    double & long, with_nop = true
    */
    pub fn push_const0(&mut self, with_nop: bool) {
        if with_nop {
            self.push_nop();
        }
        self.inner.push(Slot::Const0);
    }

    /*
       double & long, with_nop = true
    */
    pub fn push_const1(&mut self, with_nop: bool) {
        if with_nop {
            self.push_nop();
        }
        self.inner.push(Slot::Const1);
    }

    pub fn push_const2(&mut self) {
        self.inner.push(Slot::Const2);
    }

    pub fn push_const3(&mut self) {
        self.inner.push(Slot::Const3);
    }

    pub fn push_const4(&mut self) {
        self.inner.push(Slot::Const4);
    }

    pub fn push_const5(&mut self) {
        self.inner.push(Slot::Const5);
    }

    pub fn push_const_utf8(&mut self, v: BytesRef) {
        let v = OopDesc::new_str(v);
        self.inner.push(Slot::Ref(v));
    }

    pub fn push_ref(&mut self, v: OopRef) {
        self.inner.push(Slot::Ref(v));
    }

    pub fn pop_int(&mut self) -> i32 {
        match self.inner.pop().unwrap() {
            Slot::ConstM1 => -1,
            Slot::Const0 => 0,
            Slot::Const1 => 1,
            Slot::Const2 => 2,
            Slot::Const3 => 3,
            Slot::Const4 => 4,
            Slot::Const5 => 5,
            Slot::Primitive(v) => i32::from_be_bytes([v[0], v[1], v[2], v[3]]),
            _ => panic!("Illegal type"),
        }
    }

    pub fn pop_float(&mut self) -> f32 {
        match self.inner.pop().unwrap() {
            Slot::Const0 => 0.0,
            Slot::Const1 => 1.0,
            Slot::Const2 => 2.0,
            Slot::Primitive(v) => {
                let v = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                f32::from_bits(v)
            }
            _ => panic!("Illegal type"),
        }
    }

    pub fn pop_double(&mut self) -> f64 {
        match self.inner.pop() {
            Some(v) => {
                self.pop_nop();
                match v {
                    Slot::Const0 => 0.0,
                    Slot::Const1 => 1.0,
                    Slot::Primitive(v) => {
                        let v =
                            u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                        f64::from_bits(v)
                    }
                    _ => panic!("Illegal type"),
                }
            }
            None => panic!("Empty Stack!"),
        }
    }

    pub fn pop_long(&mut self) -> i64 {
        match self.inner.pop() {
            Some(v) => {
                self.pop_nop();
                match v {
                    Slot::Const0 => 0,
                    Slot::Const1 => 1,
                    Slot::Primitive(v) => {
                        i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]])
                    }
                    _ => panic!("Illegal type"),
                }
            }
            _ => panic!("Empty Stack!"),
        }
    }

    pub fn pop_ref(&mut self) -> OopRef {
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

    pub fn dup(&mut self) {
        let v = self.inner.pop().unwrap();
        self.inner.push(v.clone());
        self.inner.push(v.clone());
    }

    pub fn dup_x1(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        self.inner.push(v1.clone());
        self.inner.push(v2);
        self.inner.push(v1);
    }

    pub fn dup_x2(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        let v3 = self.inner.pop().unwrap();
        self.inner.push(v1.clone());
        self.inner.push(v3);
        self.inner.push(v2);
        self.inner.push(v1);
    }

    pub fn dup2(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        self.inner.push(v2.clone());
        self.inner.push(v1.clone());
        self.inner.push(v2);
        self.inner.push(v1);
    }

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

    pub fn swap(&mut self) {
        let v1 = self.inner.pop().unwrap();
        let v2 = self.inner.pop().unwrap();
        self.inner.push(v1);
        self.inner.push(v2);
    }
}

impl Stack {
    fn push_primitive(&mut self, b: Vec<u8>) {
        self.inner.push(Slot::Primitive(b));
    }

    fn push_primitive2(&mut self, v: [u8; 4]) {
        let v = vec![v[0], v[1], v[2], v[3]];
        self.push_primitive(v);
    }

    fn push_primitive3(&mut self, v: [u8; 8]) {
        let v = vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
        self.push_primitive(v);
    }

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
