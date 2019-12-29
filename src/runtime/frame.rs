use crate::classfile::consts;
use crate::classfile::constant_pool::ConstantType;
use crate::classfile::types::*;
use crate::classfile::ClassFile;
use crate::oop::{Oop, ClassRef, Method};
use crate::runtime::{Local, Stack, JavaThread};
use bytes::{BigEndian, Bytes};
use std::sync::Arc;
use std::ops::Deref;

pub struct Frame {
    thread: Arc<JavaThread>,
    local: Local,
    stack: Stack,
    pc: i32,
    class: ClassRef,
    code: Arc<Vec<U1>>,
}

//new & helper methods
impl Frame {
    pub fn new(thread: Arc<JavaThread>, class: ClassRef, m: Method) -> Self {
        Self {
            thread,
            local: Local::new(m.code.max_locals as usize),
            stack: Stack::new(m.code.max_stack as usize),
            pc: 0,
            class,
            code: m.code.code.clone(),
        }
    }

    fn read_i1(&mut self) -> i32 {
        let v = self.code[self.pc as usize];
        self.pc += 1;
        v as i32
    }

    fn read_i2(&mut self) -> i32 {
        self.read_i1() << 8 | self.read_i1()
    }

    fn read_i4(&mut self) -> i32 {
        self.read_i2() << 16 | self.read_i2()
    }

    fn read_u1(&mut self) -> usize {
        let v = self.code[self.pc as usize];
        self.pc += 1;
        v as usize
    }

    fn read_u2(&mut self) -> usize {
        self.read_u1() << 8 | self.read_u1()
    }

    fn load_constant(&mut self, pos: usize) {
        let cp = &self.class.lock().unwrap().class_file.cp;

        match &cp[pos] {
            ConstantType::Integer { v } => self.stack.push_int2(*v),
            ConstantType::Float { v } => self.stack.push_float2(*v),
            ConstantType::Long { v } => self.stack.push_long2(*v),
            ConstantType::Double { v } => self.stack.push_double2(*v),
            ConstantType::String { string_index } => {
                if let ConstantType::Utf8 { length, bytes } = &cp[*string_index as usize] {
                    self.stack.push_const_utf8(bytes.clone());
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

//byte code impl
impl Frame {
    pub fn nop(&mut self) {}

    pub fn aconst_null(&mut self) {
        self.stack.push_null();
    }

    pub fn iconst_m1(&mut self) {
        self.stack.push_const_m1();
    }

    pub fn iconst_0(&mut self) {
        self.stack.push_const0();
    }

    pub fn lconst_0(&mut self) {
        self.stack.push_const0();
    }

    pub fn fconst_0(&mut self) {
        self.stack.push_const0();
    }

    pub fn dconst_0(&mut self) {
        self.stack.push_const0();
    }

    pub fn iconst_1(&mut self) {
        self.stack.push_const1();
    }

    pub fn lconst_1(&mut self) {
        self.stack.push_const1();
    }

    pub fn fconst_1(&mut self) {
        self.stack.push_const1();
    }

    pub fn dconst_1(&mut self) {
        self.stack.push_const1();
    }

    pub fn iconst_2(&mut self) {
        self.stack.push_const2();
    }

    pub fn fconst_2(&mut self) {
        self.stack.push_const2();
    }

    pub fn iconst_3(&mut self) {
        self.stack.push_const3();
    }

    pub fn iconst_4(&mut self) {
        self.stack.push_const4();
    }

    pub fn iconst_5(&mut self) {
        self.stack.push_const5();
    }

    pub fn sipush(&mut self) {
        let v = self.read_i2();
        self.stack.push_int(v);
    }

    pub fn bipush(&mut self) {
        let v = self.read_i1();
        self.stack.push_int(v);
    }

    pub fn ldc(&mut self) {
        let pos = self.read_u1();
        self.load_constant(pos);
    }

    pub fn ldc_w(&mut self) {
        let pos = self.read_u2();
        self.load_constant(pos);
    }

    pub fn ldc2_w(&mut self) {
        self.ldc_w()
    }

    pub fn iload(&mut self) {
        let pos = self.read_u1();
        let v = self.local.get_int(pos);
        self.stack.push_int(v);
    }

    pub fn lload(&mut self) {
        let pos = self.read_u1();
        let v = self.local.get_long(pos);
        self.stack.push_long(v);
    }

    pub fn fload(&mut self) {
        let pos = self.read_u1();
        let v = self.local.get_float(pos);
        self.stack.push_float(v);
    }

    pub fn dload(&mut self) {
        let pos = self.read_u1();
        let v = self.local.get_double(pos);
        self.stack.push_double(v);
    }

    pub fn aload(&mut self) {
        let pos = self.read_u1();
        let v = self.local.get_ref(pos);
        self.stack.push_ref(v);
    }

    pub fn iload_0(&mut self) {
        let v = self.local.get_int(0);
        self.stack.push_int(v);
    }

    pub fn lload_0(&mut self) {
        let v = self.local.get_long(0);
        self.stack.push_long(v);
    }

    pub fn fload_0(&mut self) {
        let v = self.local.get_float(0);
        self.stack.push_float(v);
    }

    pub fn dload_0(&mut self) {
        let v = self.local.get_double(0);
        self.stack.push_double(v);
    }

    pub fn aload_0(&mut self) {
        let v = self.local.get_ref(0);
        self.stack.push_ref(v);
    }

    pub fn iload_1(&mut self) {
        let v = self.local.get_int(1);
        self.stack.push_int(v);
    }

    pub fn lload_1(&mut self) {
        let v = self.local.get_long(1);
        self.stack.push_long(v);
    }

    pub fn fload_1(&mut self) {
        let v = self.local.get_float(1);
        self.stack.push_float(v);
    }

    pub fn dload_1(&mut self) {
        let v = self.local.get_double(1);
        self.stack.push_double(v);
    }

    pub fn aload_1(&mut self) {
        let v = self.local.get_ref(1);
        self.stack.push_ref(v);
    }

    pub fn iload_2(&mut self) {
        let v = self.local.get_int(2);
        self.stack.push_int(v);
    }

    pub fn lload_2(&mut self) {
        let v = self.local.get_long(2);
        self.stack.push_long(v);
    }

    pub fn fload_2(&mut self) {
        let v = self.local.get_float(2);
        self.stack.push_float(v);
    }

    pub fn dload_2(&mut self) {
        let v = self.local.get_double(2);
        self.stack.push_double(v);
    }

    pub fn aload_2(&mut self) {
        let v = self.local.get_ref(2);
        self.stack.push_ref(v);
    }

    pub fn iload_3(&mut self) {
        let v = self.local.get_int(3);
        self.stack.push_int(v);
    }

    pub fn lload_3(&mut self) {
        let v = self.local.get_long(3);
        self.stack.push_long(v);
    }

    pub fn fload_3(&mut self) {
        let v = self.local.get_float(3);
        self.stack.push_float(v);
    }

    pub fn dload_3(&mut self) {
        let v = self.local.get_double(3);
        self.stack.push_double(v);
    }

    pub fn aload_3(&mut self) {
        let v = self.local.get_ref(3);
        self.stack.push_ref(v);
    }

    pub fn iaload(&mut self) {
        let thread = self.thread.clone();
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        match rf {
            Some(rf) => {
                match rf.deref() {
                    Oop::Array(ary) => {
                        let len = ary.get_length();
                        if (pos < 0) || (pos as usize >= ary.get_length()) {
                            let msg = format!("length is {}, but index is {}", len, pos);
                           JavaThread::throw_ext_with_msg(thread,
                            consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                                          false,
                                                          msg);
                        } else {
                            let v = ary.get_elm_at(pos as usize);
                            match v {
                                Some(v) => {
                                    match v.deref() {
                                        Oop::Int(v) => {
                                            self.stack.push_int(*v);
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            },
            None => JavaThread::throw_ext(thread, consts::J_NPE, false),
        }
    }

    pub fn saload(&mut self) {
        self.iaload();
    }

    pub fn caload(&mut self) {
        self.iaload();
    }

    pub fn baload(&mut self) {
        self.iaload();
    }

    pub fn laload(&mut self) {
        let thread = self.thread.clone();
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        match rf {
            Some(rf) => {
                match rf.deref() {
                    Oop::Array(ary) => {
                        let len = ary.get_length();
                        if (pos < 0) || (pos as usize >= ary.get_length()) {
                            let msg = format!("length is {}, but index is {}", len, pos);
                            JavaThread::throw_ext_with_msg(thread,
                                                           consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                                           false,
                                                           msg);
                        } else {
                            let v = ary.get_elm_at(pos as usize);
                            match v {
                                Some(v) => {
                                    match v.deref() {
                                        Oop::Long(v) => {
                                            self.stack.push_long(*v);
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            },
            None => JavaThread::throw_ext(thread, consts::J_NPE, false),
        }
    }

    pub fn faload(&mut self) {
        let thread = self.thread.clone();
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        match rf {
            Some(rf) => {
                match rf.deref() {
                    Oop::Array(ary) => {
                        let len = ary.get_length();
                        if (pos < 0) || (pos as usize >= ary.get_length()) {
                            let msg = format!("length is {}, but index is {}", len, pos);
                            JavaThread::throw_ext_with_msg(thread,
                                                           consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                                           false,
                                                           msg);
                        } else {
                            let v = ary.get_elm_at(pos as usize);
                            match v {
                                Some(v) => {
                                    match v.deref() {
                                        Oop::Float(v) => {
                                            self.stack.push_float(*v);
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            },
            None => JavaThread::throw_ext(thread, consts::J_NPE, false),
        }
    }

    pub fn daload(&mut self) {
        let thread = self.thread.clone();
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        match rf {
            Some(rf) => {
                match rf.deref() {
                    Oop::Array(ary) => {
                        let len = ary.get_length();
                        if (pos < 0) || (pos as usize >= ary.get_length()) {
                            let msg = format!("length is {}, but index is {}", len, pos);
                            JavaThread::throw_ext_with_msg(thread,
                                                           consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                                           false,
                                                           msg);
                        } else {
                            let v = ary.get_elm_at(pos as usize);
                            match v {
                                Some(v) => {
                                    match v.deref() {
                                        Oop::Double(v) => {
                                            self.stack.push_double(*v);
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            },
            None => JavaThread::throw_ext(thread, consts::J_NPE, false),
        }
    }

    pub fn aaload(&mut self) {
        let thread = self.thread.clone();
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        match rf {
            Some(rf) => {
                match rf.deref() {
                    Oop::Array(ary) => {
                        let len = ary.get_length();
                        if (pos < 0) || (pos as usize >= ary.get_length()) {
                            let msg = format!("length is {}, but index is {}", len, pos);
                            JavaThread::throw_ext_with_msg(thread,
                                                           consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                                           false,
                                                           msg);
                        } else {
                            let v = ary.get_elm_at(pos as usize);
                            self.stack.push_ref(v);
                        }
                    }
                    _ => unreachable!(),
                }
            },
            None => JavaThread::throw_ext(thread, consts::J_NPE, false),
        }
    }

    pub fn istore(&mut self) {
        let pos = self.read_u1();
        let v = self.stack.pop_int();
        self.local.set_int(pos, v);
    }

    pub fn lstore(&mut self) {
        let pos = self.read_u1();
        let v = self.stack.pop_long();
        self.local.set_long(pos, v);
    }

    pub fn fstore(&mut self) {
        let pos = self.read_u1();
        let v = self.stack.pop_float();
        self.local.set_float(pos, v);
    }

    pub fn dstore(&mut self) {
        let pos = self.read_u1();
        let v = self.stack.pop_double();
        self.local.set_double(pos, v);
    }

    pub fn astore(&mut self) {
        let pos = self.read_u1();
        let v = self.stack.pop_ref();
        self.local.set_ref(pos, v);
    }

    pub fn istore_0(&mut self) {
        let v = self.stack.pop_int();
        self.local.set_int(0, v);
    }

    pub fn istore_1(&mut self) {
        let v = self.stack.pop_int();
        self.local.set_int(1, v);
    }

    pub fn istore_2(&mut self) {
        let v = self.stack.pop_int();
        self.local.set_int(2, v);
    }

    pub fn istore_3(&mut self) {
        let v = self.stack.pop_int();
        self.local.set_int(3, v);
    }

    pub fn lstore_0(&mut self) {
        let v = self.stack.pop_long();
        self.local.set_long(0, v);
    }

    pub fn lstore_1(&mut self) {
        let v = self.stack.pop_long();
        self.local.set_long(1, v);
    }

    pub fn lstore_2(&mut self) {
        let v = self.stack.pop_long();
        self.local.set_long(2, v);
    }

    pub fn lstore_3(&mut self) {
        let v = self.stack.pop_long();
        self.local.set_long(3, v);
    }

    pub fn fstore_0(&mut self) {
        let v = self.stack.pop_float();
        self.local.set_float(0, v);
    }

    pub fn fstore_1(&mut self) {
        let v = self.stack.pop_float();
        self.local.set_float(1, v);
    }

    pub fn fstore_2(&mut self) {
        let v = self.stack.pop_float();
        self.local.set_float(2, v);
    }

    pub fn fstore_3(&mut self) {
        let v = self.stack.pop_float();
        self.local.set_float(3, v);
    }

    pub fn dstore_0(&mut self) {
        let v = self.stack.pop_double();
        self.local.set_double(0, v);
    }

    pub fn dstore_1(&mut self) {
        let v = self.stack.pop_double();
        self.local.set_double(1, v);
    }

    pub fn dstore_2(&mut self) {
        let v = self.stack.pop_double();
        self.local.set_double(2, v);
    }

    pub fn dstore_3(&mut self) {
        let v = self.stack.pop_double();
        self.local.set_double(3, v);
    }

    pub fn astore_0(&mut self) {
        let v = self.stack.pop_ref();
        self.local.set_ref(0, v);
    }

    pub fn astore_1(&mut self) {
        let v = self.stack.pop_ref();
        self.local.set_ref(1, v);
    }

    pub fn astore_2(&mut self) {
        let v = self.stack.pop_ref();
        self.local.set_ref(2, v);
    }

    pub fn astore_3(&mut self) {
        let v = self.stack.pop_ref();
        self.local.set_ref(3, v);
    }

    pub fn bastore(&mut self) {
        //todo: impl
    }

    pub fn castore(&mut self) {
        //todo: impl
    }

    pub fn sastore(&mut self) {
        //todo: impl
    }

    pub fn iastore(&mut self) {
        //todo: impl
    }

    pub fn lastore(&mut self) {
        //todo: impl
    }

    pub fn fastore(&mut self) {
        //todo: impl
    }

    pub fn dastore(&mut self) {
        //todo: impl
    }

    pub fn aastore(&mut self) {
        //todo: impl
    }

    pub fn pop(&mut self) {
        self.stack.drop_top();
    }

    pub fn pop2(&mut self) {
        self.stack.drop_top();
        self.stack.drop_top();
    }

    pub fn dup(&mut self) {
        let v = self.stack.pop_ref();
        self.stack.push_ref(v.clone());
        self.stack.push_ref(v);
    }

    pub fn dup_x1(&mut self) {
        let v1 = self.stack.pop_ref();
        let v2 = self.stack.pop_ref();
        self.stack.push_ref(v1.clone());
        self.stack.push_ref(v2);
        self.stack.push_ref(v1);
    }

    pub fn dup_x2(&mut self) {
        let v1 = self.stack.pop_ref();
        let v2 = self.stack.pop_ref();
        let v3 = self.stack.pop_ref();
        self.stack.push_ref(v1.clone());
        self.stack.push_ref(v3);
        self.stack.push_ref(v2);
        self.stack.push_ref(v1);
    }

    pub fn dup2(&mut self) {
        let v1 = self.stack.pop_ref();
        let v2 = self.stack.pop_ref();
        self.stack.push_ref(v2.clone());
        self.stack.push_ref(v1.clone());
        self.stack.push_ref(v2);
        self.stack.push_ref(v1);
    }

    pub fn dup2_x1(&mut self) {
        let v1 = self.stack.pop_ref();
        let v2 = self.stack.pop_ref();
        let v3 = self.stack.pop_ref();
        self.stack.push_ref(v2.clone());
        self.stack.push_ref(v1.clone());
        self.stack.push_ref(v3);
        self.stack.push_ref(v2);
        self.stack.push_ref(v1);
    }

    pub fn dup2_x2(&mut self) {
        let v1 = self.stack.pop_ref();
        let v2 = self.stack.pop_ref();
        let v3 = self.stack.pop_ref();
        let v4 = self.stack.pop_ref();
        self.stack.push_ref(v2.clone());
        self.stack.push_ref(v1.clone());
        self.stack.push_ref(v4);
        self.stack.push_ref(v3);
        self.stack.push_ref(v2);
        self.stack.push_ref(v1);
    }

    pub fn swap(&mut self) {
        let v1 = self.stack.pop_ref();
        let v2 = self.stack.pop_ref();
        self.stack.push_ref(v1);
        self.stack.push_ref(v2);
    }

    pub fn iadd(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        self.stack.push_int(v1 + v2);
    }

    pub fn ladd(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        self.stack.push_long(v1 + v2);
    }

    pub fn fadd(&mut self) {
        let v2 = self.stack.pop_float();
        let v1 = self.stack.pop_float();
        self.stack.push_float(v1 + v2);
    }

    pub fn dadd(&mut self) {
        let v2 = self.stack.pop_double();
        let v1 = self.stack.pop_double();
        self.stack.push_double(v1 + v2);
    }

    pub fn isub(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        self.stack.push_int(v1 - v2);
    }

    pub fn lsub(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        self.stack.push_long(v1 - v2);
    }

    pub fn fsub(&mut self) {
        let v2 = self.stack.pop_float();
        let v1 = self.stack.pop_float();
        self.stack.push_float(v1 - v2);
    }

    pub fn dsub(&mut self) {
        let v2 = self.stack.pop_double();
        let v1 = self.stack.pop_double();
        self.stack.push_double(v1 - v2);
    }

    pub fn imul(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        self.stack.push_int(v1 * v2);
    }

    pub fn lmul(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        self.stack.push_long(v1 * v2);
    }

    pub fn fmul(&mut self) {
        let v2 = self.stack.pop_float();
        let v1 = self.stack.pop_float();
        self.stack.push_float(v1 * v2);
    }

    pub fn dmul(&mut self) {
        let v2 = self.stack.pop_double();
        let v1 = self.stack.pop_double();
        self.stack.push_double(v1 * v2);
    }

    pub fn idiv(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v2 == 0 {
            //todo: handle exception
        }
        self.stack.push_int(v1 / v2);
    }

    pub fn ldiv(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        if v2 == 0 {
            //todo: handle exception
        }
        self.stack.push_long(v1 / v2);
    }

    pub fn fdiv(&mut self) {
        let v2 = self.stack.pop_float();
        let v1 = self.stack.pop_float();
        if v2 == 0.0 {
            //todo: handle exception
        }
        self.stack.push_float(v1 / v2);
    }

    pub fn ddiv(&mut self) {
        let v2 = self.stack.pop_double();
        let v1 = self.stack.pop_double();
        if v2 == 0.0 {
            //todo: handle exception
        }
        self.stack.push_double(v1 / v2);
    }

    pub fn irem(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v2 == 0 {
            //todo: handle exception
        }
        self.stack.push_int(v1 - (v1 / v2) * v2);
    }

    pub fn lrem(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        if v2 == 0 {
            //todo: handle exception
        }
        self.stack.push_long(v1 - (v1 / v2) * v2);
    }

    pub fn frem(&mut self) {
        panic!("Use of deprecated instruction frem, please check your Java compiler");
    }

    pub fn drem(&mut self) {
        panic!("Use of deprecated instruction drem, please check your Java compiler");
    }

    pub fn ineg(&mut self) {
        let v = self.stack.pop_int();
        self.stack.push_int(-v);
    }

    pub fn lneg(&mut self) {
        let v = self.stack.pop_long();
        self.stack.push_long(-v);
    }

    pub fn fneg(&mut self) {
        panic!("Use of deprecated instruction fneg, please check your Java compiler");
    }

    pub fn dneg(&mut self) {
        panic!("Use of deprecated instruction dneg, please check your Java compiler");
    }

    pub fn ishl(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        let s = v2 & 0x1F;
        self.stack.push_int(v1 << s);
    }

    pub fn lshl(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        self.stack.push_long(v1 << s);
    }

    pub fn ishr(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        let s = v2 & 0x1F;
        self.stack.push_int(v1 >> s);
    }

    pub fn lshr(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        self.stack.push_long(v1 >> s);
    }

    pub fn iushr(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        let s = v2 & 0x1F;
        if v1 >= 0 {
            self.stack.push_int(v1 >> s);
        } else {
            self.stack.push_int((v1 >> s) + (2 << !s));
        }
    }

    pub fn lushr(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        if v1 >= 0 {
            self.stack.push_long(v1 >> s);
        } else {
            self.stack.push_long((v1 >> s) + (2 << !s));
        }
    }

    pub fn iand(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        self.stack.push_int(v1 & v2);
    }

    pub fn land(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        self.stack.push_long(v1 & v2);
    }

    pub fn ior(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        self.stack.push_int(v1 | v2);
    }

    pub fn lor(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        self.stack.push_long(v1 | v2);
    }

    pub fn ixor(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        self.stack.push_int(v1 ^ v2);
    }

    pub fn lxor(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        self.stack.push_long(v1 ^ v2);
    }

    pub fn iinc(&mut self) {
        let pos = self.read_u1();
        let factor = self.read_i1();

        let v = self.local.get_int(pos);
        let v = v + factor;
        self.local.set_int(pos, v);
    }

    pub fn i2l(&mut self) {
        let v = self.stack.pop_int();
        self.stack.push_long(v as i64);
    }

    pub fn i2f(&mut self) {
        let v = self.stack.pop_int();
        self.stack.push_float(v as f32);
    }

    pub fn i2d(&mut self) {
        let v = self.stack.pop_int();
        self.stack.push_double(v as f64);
    }

    pub fn l2i(&mut self) {
        let v = self.stack.pop_long();
        self.stack.push_int(v as i32);
    }

    pub fn l2f(&mut self) {
        let v = self.stack.pop_long();
        self.stack.push_float(v as f32);
    }

    pub fn l2d(&mut self) {
        let v = self.stack.pop_long();
        self.stack.push_double(v as f64);
    }

    pub fn f2i(&mut self) {
        let v = self.stack.pop_float();
        if v.is_nan() {
            self.stack.push_int(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                self.stack.push_int(std::i32::MAX);
            } else {
                self.stack.push_int(std::i32::MIN);
            }
        } else {
            self.stack.push_int(v as i32);
        }
    }

    pub fn f2l(&mut self) {
        let v = self.stack.pop_float();
        if v.is_nan() {
            self.stack.push_long(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                self.stack.push_long(std::i64::MAX);
            } else {
                self.stack.push_long(std::i64::MIN);
            }
        } else {
            self.stack.push_long(v as i64);
        }
    }

    pub fn f2d(&mut self) {
        let v = self.stack.pop_float();
        self.stack.push_double(v as f64);
    }

    pub fn d2i(&mut self) {
        let v = self.stack.pop_double();
        if v.is_nan() {
            self.stack.push_int(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                self.stack.push_int(std::i32::MAX);
            } else {
                self.stack.push_int(std::i32::MIN);
            }
        } else {
            self.stack.push_int(v as i32);
        }
    }

    pub fn d2l(&mut self) {
        let v = self.stack.pop_double();
        if v.is_nan() {
            self.stack.push_long(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                self.stack.push_long(std::i64::MAX);
            } else {
                self.stack.push_long(std::i64::MIN);
            }
        } else {
            self.stack.push_long(v as i64);
        }
    }

    pub fn d2f(&mut self) {
        let v = self.stack.pop_double();
        self.stack.push_float(v as f32);
    }

    pub fn i2b(&mut self) {
        let v = self.stack.pop_int();
        let v = v as i8;
        self.stack.push_int(v as i32);
    }

    pub fn i2c(&mut self) {
        let v = self.stack.pop_int();
        let v = v as u16;
        self.stack.push_int(v as i32);
    }

    pub fn i2s(&mut self) {
        let v = self.stack.pop_int();
        let v = v as i16;
        self.stack.push_int(v as i32);
    }

    pub fn lcmp(&mut self) {
        let v1 = self.stack.pop_long();
        let v2 = self.stack.pop_long();
        if v1 > v2 {
            self.stack.push_int(-1);
        } else if v1 < v2 {
            self.stack.push_int(1);
        } else {
            self.stack.push_int(0);
        }
    }

    pub fn fcmpl(&mut self) {
        let v1 = self.stack.pop_float();
        let v2 = self.stack.pop_float();
        if v1.is_nan() || v2.is_nan() {
            self.stack.push_int(-1);
        } else if v1 > v2 {
            self.stack.push_int(-1);
        } else if v1 < v2 {
            self.stack.push_int(1);
        } else {
            self.stack.push_int(0);
        }
    }

    pub fn fcmpg(&mut self) {
        let v1 = self.stack.pop_float();
        let v2 = self.stack.pop_float();
        if v1.is_nan() || v2.is_nan() {
            self.stack.push_int(1);
        } else if v1 > v2 {
            self.stack.push_int(-1);
        } else if v1 < v2 {
            self.stack.push_int(1);
        } else {
            self.stack.push_int(0);
        }
    }

    pub fn dcmpl(&mut self) {
        let v1 = self.stack.pop_double();
        let v2 = self.stack.pop_double();
        if v1.is_nan() || v2.is_nan() {
            self.stack.push_int(-1);
        } else if v1 > v2 {
            self.stack.push_int(-1);
        } else if v1 < v2 {
            self.stack.push_int(1);
        } else {
            self.stack.push_int(0);
        }
    }

    pub fn dcmpg(&mut self) {
        let v1 = self.stack.pop_double();
        let v2 = self.stack.pop_double();
        if v1.is_nan() || v2.is_nan() {
            self.stack.push_int(1);
        } else if v1 > v2 {
            self.stack.push_int(-1);
        } else if v1 < v2 {
            self.stack.push_int(1);
        } else {
            self.stack.push_int(0);
        }
    }

    pub fn ifeq(&mut self) {
        let v = self.stack.pop_int();
        if v == 0 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn ifne(&mut self) {
        let v = self.stack.pop_int();
        if v != 0 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn iflt(&mut self) {
        let v = self.stack.pop_int();
        if v < 0 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn ifge(&mut self) {
        let v = self.stack.pop_int();
        if v >= 0 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn ifgt(&mut self) {
        let v = self.stack.pop_int();
        if v > 0 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn ifle(&mut self) {
        let v = self.stack.pop_int();
        if v <= 0 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpeq(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 == v2 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpne(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 != v2 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmplt(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 < v2 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpge(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 >= v2 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpgt(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 > v2 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmple(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 <= v2 {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_acmpeq(&mut self) {
        let v2 = self.stack.pop_ref();
        let v1 = self.stack.pop_ref();
        let eq = match (v1, v2) {
            (Some(v1), Some(v2)) => Arc::ptr_eq(&v1, &v2),
            (None, None) => true,
            _ => false,
        };

        if eq {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_acmpne(&mut self) {
        let v2 = self.stack.pop_ref();
        let v1 = self.stack.pop_ref();
        let eq = match (v1, v2) {
            (Some(v1), Some(v2)) => Arc::ptr_eq(&v1, &v2),
            (None, None) => true,
            _ => false,
        };

        if !eq {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn goto(&mut self) {
        let branch = self.read_i2();
        self.pc += branch;
        self.pc += -1;
    }

    pub fn jsr(&mut self) {
        self.pc += 2;
        panic!("Use of deprecated instruction jsr, please check your Java compiler");
    }

    pub fn ret(&mut self) {
        self.pc += 1;
        panic!("Use of deprecated instruction ret, please check your Java compiler");
    }

    pub fn table_switch(&mut self) {
        //todo: impl
    }

    pub fn lookup_switch(&mut self) {
        //todo: impl
    }
    pub fn ireturn(&mut self) {
        //todo: impl
    }
    pub fn lreturn(&mut self) {
        //todo: impl
    }
    pub fn freturn(&mut self) {
        //todo: impl
    }
    pub fn dreturn(&mut self) {
        //todo: impl
    }
    pub fn areturn(&mut self) {
        //todo: impl
    }
    pub fn return_(&mut self) {
        //todo: impl
    }
    pub fn get_static(&mut self) {
        //todo: impl
    }
    pub fn put_static(&mut self) {
        //todo: impl
    }
    pub fn get_field(&mut self) {
        //todo: impl
    }
    pub fn put_field(&mut self) {
        //todo: impl
    }
    pub fn invoke_virtual(&mut self) {
        //todo: impl
    }
    pub fn invoke_static(&mut self) {
        //todo: impl
    }
    pub fn invoke_interface(&mut self) {
        //todo: impl
    }
    pub fn invoke_dynamic(&mut self) {
        //todo: impl
    }
    pub fn new_(&mut self) {
        //todo: impl
    }
    pub fn new_array(&mut self) {
        //todo: impl
    }
    pub fn anew_array(&mut self) {
        //todo: impl
    }
    pub fn array_length(&mut self) {
        //todo: impl
    }
    pub fn athrow(&mut self) {
        //todo: impl
    }
    pub fn check_cast(&mut self) {
        //todo: impl
    }
    pub fn instance_of(&mut self) {
        //todo: impl
    }
    pub fn monitor_enter(&mut self) {
        //todo: impl
    }
    pub fn monitor_exit(&mut self) {
        //todo: impl
    }
    pub fn wide(&mut self) {
        //todo: impl
        panic!("Use of deprecated instruction wide, please check your Java compiler")
    }
    pub fn multi_anew_array(&mut self) {
        //todo: impl
    }
    pub fn if_null(&mut self) {
        let v = self.stack.pop_ref();
        if v.is_none() {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn if_non_null(&mut self) {
        let v = self.stack.pop_ref();
        if v.is_some() {
            let branch = self.read_i2();
            self.pc += branch;
            self.pc += -1;
        } else {
            self.pc += 2;
        }
    }

    pub fn goto_w(&mut self) {
        self.pc += 4;
        panic!("Use of deprecated instruction goto_w, please check your Java compiler")
    }

    pub fn jsr_w(&mut self) {
        self.pc += 4;
        panic!("Use of deprecated instruction jsr_w, please check your Java compiler")
    }

    pub fn other_wise(&mut self) {
        let pc = self.pc - 1;
        panic!(
            "Use of undefined bytecode: {} at {}",
            self.code[pc as usize], pc
        );
    }
}
