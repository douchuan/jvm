#![allow(unused)]

pub mod class_loader;
pub mod class_path_manager;
pub mod execution;
pub mod system_dictionary;

use bytes::{BigEndian, Bytes};

use crate::classfile::constant_pool::ConstantType;
use crate::classfile::method_info::MethodInfo;
use crate::classfile::types::*;
use crate::classfile::ClassFile;

enum Slot {
    ConstM1,
    Const0,
    Const1,
    Const2,
    Const3,
    Const4,
    Const5,
    Primitive(Bytes),
    Ref,
    Null,
}

struct Locals {
    locals: Vec<Slot>,
}

impl Locals {
    fn new(size: usize) -> Self {
        Self {
            locals: Vec::with_capacity(size),
        }
    }

    fn set_int(&mut self, pos: usize, i: i32) {
        let v = i.to_be_bytes();
        self.set(pos, Bytes::from(vec![v[0], v[1], v[2], v[3]]));
    }

    fn set_long(&mut self, pos: usize, l: i64) {
        let v = l.to_be_bytes();
        self.set(
            pos,
            Bytes::from(vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]),
        );
    }

    fn set_float(&mut self, pos: usize, f: f32) {
        let v = f.to_bits().to_be_bytes();
        self.set(pos, Bytes::from(vec![v[0], v[1], v[2], v[3]]));
    }

    fn set_double(&mut self, pos: usize, d: f64) {
        let v = d.to_bits().to_be_bytes();
        self.set(
            pos,
            Bytes::from(vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]),
        );
    }

    fn set(&mut self, pos: usize, buf: Bytes) {
        self.locals[pos] = Slot::Primitive(buf);
    }

    fn get_int(&self, pos: usize) -> i32 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            i32::from_be_bytes([v[0], v[1], v[2], v[3]])
        } else {
            panic!("Illegal type");
        }
    }

    fn get_long(&self, pos: usize) -> i64 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]])
        } else {
            panic!("Illegal type");
        }
    }

    fn get_float(&self, pos: usize) -> f32 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            let v = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
            f32::from_bits(v)
        } else {
            panic!("Illegal type");
        }
    }

    fn get_double(&self, pos: usize) -> f64 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            let v = u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
            f64::from_bits(v)
        } else {
            panic!("Illegal type");
        }
    }
}

struct Stack {
    inner: Vec<Slot>,
}

impl Stack {
    fn new(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }

    fn push_int(&mut self, i: i32) {
        let v = i.to_be_bytes();
        self.push(Bytes::from(vec![v[0], v[1], v[2], v[3]]));
    }

    fn push_float(&mut self, f: f32) {
        let v = f.to_bits().to_be_bytes();
        self.push(Bytes::from(vec![v[0], v[1], v[2], v[3]]));
    }

    fn push_double(&mut self, d: f64) {
        let v = d.to_bits().to_be_bytes();
        self.push(Bytes::from(vec![
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7],
        ]));
    }

    fn push_long(&mut self, l: i64) {
        let v = l.to_be_bytes();
        self.push(Bytes::from(vec![
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7],
        ]));
    }

    fn push(&mut self, b: Bytes) {
        self.inner.push(Slot::Primitive(b));
    }

    fn push2(&mut self, v: [u8; 4]) {
        let v = vec![v[0], v[1], v[2], v[3]];
        let v = Bytes::from(v);
        self.push(v)
    }

    fn push3(&mut self, v: [u8; 8]) {
        let v = vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
        let v = Bytes::from(v);
        self.push(v);
    }

    fn push_null(&mut self) {
        self.inner.push(Slot::Null);
    }

    fn push_const_m1(&mut self) {
        self.inner.push(Slot::ConstM1);
    }

    fn push_const0(&mut self) {
        self.inner.push(Slot::Const0);
    }

    fn push_const1(&mut self) {
        self.inner.push(Slot::Const1);
    }

    fn push_const2(&mut self) {
        self.inner.push(Slot::Const2);
    }

    fn push_const3(&mut self) {
        self.inner.push(Slot::Const3);
    }

    fn push_const4(&mut self) {
        self.inner.push(Slot::Const4);
    }

    fn push_const5(&mut self) {
        self.inner.push(Slot::Const5);
    }

    fn pop_int(&mut self) -> i32 {
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

    fn pop_float(&mut self) -> f32 {
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

    fn pop_double(&mut self) -> f64 {
        match self.inner.pop().unwrap() {
            Slot::Const0 => 0.0,
            Slot::Const1 => 1.0,
            Slot::Primitive(v) => {
                let v = u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                f64::from_bits(v)
            }
            _ => panic!("Illegal type"),
        }
    }

    fn pop_long(&mut self) -> i64 {
        match self.inner.pop().unwrap() {
            Slot::Const0 => 0,
            Slot::Const1 => 1,
            Slot::Primitive(v) => {
                i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]])
            }
            _ => panic!("Illegal type"),
        }
    }

    fn drop_top(&mut self) {
        let _ = self.inner.pop();
    }

    fn clear(&mut self) {
        self.inner.clear();
    }
}

struct Frame<'cls> {
    locals: Locals,
    stack: Stack,
    pid: usize,
    class: &'cls ClassFile,
    code: &'cls Vec<U1>,
}

impl<'cls> Frame<'cls> {
    fn new(class: &'cls ClassFile, m: &'cls MethodInfo) -> Self {
        let code = m.get_code();

        Self {
            locals: Locals::new(code.max_locals as usize),
            stack: Stack::new(code.max_stack as usize),
            pid: 0,
            class,
            code: &code.code,
        }
    }

    fn read_i1(&mut self) -> i32 {
        let v = self.code[self.pid];
        self.pid += 1;
        v as i32
    }

    fn read_i2(&mut self) -> i32 {
        self.read_i1() << 8 | self.read_i1()
    }

    fn read_i4(&mut self) -> i32 {
        self.read_i2() << 16 | self.read_i2()
    }

    fn read_u1(&mut self) -> usize {
        let v = self.code[self.pid];
        self.pid += 1;
        v as usize
    }

    fn read_u2(&mut self) -> usize {
        self.read_u1() << 8 | self.read_u1()
    }

    fn load_constant(&mut self, pos: usize) {
        match &self.class.cp[pos] {
            ConstantType::Integer { v } => self.stack.push2(*v),
            ConstantType::Float { v } => self.stack.push2(*v),
            ConstantType::Long { v } => self.stack.push3(*v),
            ConstantType::Double { v } => self.stack.push3(*v),
            ConstantType::String { string_index } => {
                if let ConstantType::Utf8 { length, bytes } = &self.class.cp[*string_index as usize]
                {
                    //todo: try to optimize, avoid copy bytes, just push string_index, can work?
                    self.stack.push(Bytes::from(bytes.as_slice()));
                } else {
                    unreachable!()
                }
            }
            ConstantType::Class { name_index } => {
                //todo: impl me
                unimplemented!()
            }
            _ => unreachable!(),
        }
    }
}

impl<'cls> Frame<'cls> {
    fn nop(&mut self) {}

    fn aconst_null(&mut self) {
        self.stack.push_null();
    }

    //todo: opt opcode *const* by
    //      const I_NEGATIVE_1 = vec![xx ], const I_ZERO, L_ZERO...
    fn iconst_m1(&mut self) {
        self.stack.push_const_m1();
    }

    fn iconst_0(&mut self) {
        self.stack.push_const0();
    }

    fn lconst_0(&mut self) {
        self.stack.push_const0();
    }

    fn fconst_0(&mut self) {
        self.stack.push_const0();
    }

    fn dconst_0(&mut self) {
        self.stack.push_const0();
    }

    fn iconst_1(&mut self) {
        self.stack.push_const1();
    }

    fn lconst_1(&mut self) {
        self.stack.push_const1();
    }

    fn fconst_1(&mut self) {
        self.stack.push_const1();
    }

    fn dconst_1(&mut self) {
        self.stack.push_const1();
    }

    fn iconst_2(&mut self) {
        self.stack.push_const2();
    }

    fn fconst_2(&mut self) {
        self.stack.push_const2();
    }

    fn iconst_3(&mut self) {
        self.stack.push_const3();
    }

    fn iconst_4(&mut self) {
        self.stack.push_const4();
    }

    fn iconst_5(&mut self) {
        self.stack.push_const5();
    }

    fn sipush(&mut self) {
        let v = self.read_i2();
        self.stack.push_int(v);
    }

    fn bipush(&mut self) {
        let v = self.read_i1();
        self.stack.push_int(v);
    }

    fn ldc(&mut self) {
        let pos = self.read_u1();
        self.load_constant(pos);
    }

    fn ldc_w(&mut self) {
        let pos = self.read_u2();
        self.load_constant(pos);
    }

    fn ldc2_w(&mut self) {
        self.ldc_w()
    }

    fn iload(&mut self) {
        let pos = self.read_u1();
        let v = self.locals.get_int(pos);
        self.stack.push_int(v);
    }

    fn lload(&mut self) {
        let pos = self.read_u1();
        let v = self.locals.get_long(pos);
        self.stack.push_long(v);
    }

    fn fload(&mut self) {
        let pos = self.read_u1();
        let v = self.locals.get_float(pos);
        self.stack.push_float(v);
    }

    fn dload(&mut self) {
        let pos = self.read_u1();
        let v = self.locals.get_double(pos);
        self.stack.push_double(v);
    }

    fn aload(&mut self) {
        let pos = self.read_u1();
        //
    }

    fn iload_0(&mut self) {
        let v = self.locals.get_int(0);
        self.stack.push_int(v);
    }

    fn lload_0(&mut self) {
        let v = self.locals.get_long(0);
        self.stack.push_long(v);
    }

    fn fload_0(&mut self) {
        let v = self.locals.get_float(0);
        self.stack.push_float(v);
    }

    fn dload_0(&mut self) {
        let v = self.locals.get_double(0);
        self.stack.push_double(v);
    }

    fn aload_0(&mut self) {
        //todo: impl
        unimplemented!()
    }

    fn iload_1(&mut self) {
        let v = self.locals.get_int(1);
        self.stack.push_int(v);
    }

    fn lload_1(&mut self) {
        let v = self.locals.get_long(1);
        self.stack.push_long(v);
    }

    fn fload_1(&mut self) {
        let v = self.locals.get_float(1);
        self.stack.push_float(v);
    }

    fn dload_1(&mut self) {
        let v = self.locals.get_double(1);
        self.stack.push_double(v);
    }

    fn aload_1(&mut self) {
        //todo: impl
        unimplemented!()
    }

    fn iload_2(&mut self) {
        let v = self.locals.get_int(2);
        self.stack.push_int(v);
    }

    fn lload_2(&mut self) {
        let v = self.locals.get_long(2);
        self.stack.push_long(v);
    }

    fn fload_2(&mut self) {
        let v = self.locals.get_float(2);
        self.stack.push_float(v);
    }

    fn dload_2(&mut self) {
        let v = self.locals.get_double(2);
        self.stack.push_double(v);
    }

    fn aload_2(&mut self) {
        //todo: impl
        unimplemented!()
    }

    fn iload_3(&mut self) {
        let v = self.locals.get_int(3);
        self.stack.push_int(v);
    }

    fn lload_3(&mut self) {
        let v = self.locals.get_long(3);
        self.stack.push_long(v);
    }

    fn fload_3(&mut self) {
        let v = self.locals.get_float(3);
        self.stack.push_float(v);
    }

    fn dload_3(&mut self) {
        let v = self.locals.get_double(3);
        self.stack.push_double(v);
    }

    fn aload_3(&mut self) {
        //todo: impl
        unimplemented!()
    }

    fn wide(&mut self) {
        panic!("Use of deprecated instruction wide, please check your Java compiler")
    }
}
