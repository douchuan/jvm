use crate::classfile::constant_pool::ConstantType;
use crate::classfile::types::*;
use crate::classfile::ClassFile;
use crate::oop::{ClassRef, Method};
use crate::runtime::{Local, Stack};
use bytes::{BigEndian, Bytes};
use std::sync::Arc;

pub struct Frame {
    local: Local,
    stack: Stack,
    pid: usize,
    class: ClassRef,
    code: Arc<Vec<U1>>,
}

//new & helper methods
impl Frame {
    pub fn new(class: ClassRef, m: Method) -> Self {
        Self {
            local: Local::new(m.code.max_locals as usize),
            stack: Stack::new(m.code.max_stack as usize),
            pid: 0,
            class,
            code: m.code.code.clone(),
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
        //todo: impl
    }

    pub fn saload(&mut self) {
        //todo: impl
    }

    pub fn caload(&mut self) {
        //todo: impl
    }

    pub fn baload(&mut self) {
        //todo: impl
    }

    pub fn laload(&mut self) {
        //todo: impl
    }

    pub fn faload(&mut self) {
        //todo: impl
    }

    pub fn daload(&mut self) {
        //todo: impl
    }

    pub fn aaload(&mut self) {
        //todo: impl
    }

    pub fn istore(&mut self) {
        //todo: impl
    }

    pub fn lstore(&mut self) {
        //todo: impl
    }

    pub fn fstore(&mut self) {
        //todo: impl
    }

    pub fn dstore(&mut self) {
        //todo: impl
    }

    pub fn astore(&mut self) {
        //todo: impl
    }

    pub fn istore_0(&mut self) {
        //todo: impl
    }
    pub fn istore_1(&mut self) {
        //todo: impl
    }
    pub fn istore_2(&mut self) {
        //todo: impl
    }

    pub fn istore_3(&mut self) {
        //todo: impl
    }

    pub fn lstore_0(&mut self) {
        //todo: impl
    }

    pub fn lstore_1(&mut self) {
        //todo: impl
    }

    pub fn lstore_2(&mut self) {
        //todo: impl
    }

    pub fn lstore_3(&mut self) {
        //todo: impl
    }

    pub fn fstore_0(&mut self) {
        //todo: impl
    }

    pub fn fstore_1(&mut self) {
        //todo: impl
    }

    pub fn fstore_2(&mut self) {
        //todo: impl
    }

    pub fn fstore_3(&mut self) {
        //todo: impl
    }

    pub fn dstore_0(&mut self) {
        //todo: impl
    }

    pub fn dstore_1(&mut self) {
        //todo: impl
    }

    pub fn dstore_2(&mut self) {
        //todo: impl
    }

    pub fn dstore_3(&mut self) {
        //todo: impl
    }

    pub fn astore_0(&mut self) {
        //todo: impl
    }

    pub fn astore_1(&mut self) {
        //todo: impl
    }

    pub fn astore_2(&mut self) {
        //todo: impl
    }

    pub fn astore_3(&mut self) {
        //todo: impl
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
        //todo: impl
    }

    pub fn pop2(&mut self) {
        //todo: impl
    }

    pub fn dup(&mut self) {
        //todo: impl
    }

    pub fn dup_x1(&mut self) {
        //todo: impl
    }

    pub fn dup_x2(&mut self) {
        //todo: impl
    }

    pub fn dup2(&mut self) {
        //todo: impl
    }

    pub fn dup2_x1(&mut self) {
        //todo: impl
    }

    pub fn dup2_x2(&mut self) {
        //todo: impl
    }

    pub fn swap(&mut self) {
        //todo: impl
    }

    pub fn iadd(&mut self) {
        //todo: impl
    }

    pub fn ladd(&mut self) {
        //todo: impl
    }

    pub fn fadd(&mut self) {
        //todo: impl
    }

    pub fn dadd(&mut self) {
        //todo: impl
    }

    pub fn isub(&mut self) {
        //todo: impl
    }

    pub fn lsub(&mut self) {
        //todo: impl
    }
    pub fn fsub(&mut self) {
        //todo: impl
    }
    pub fn dsub(&mut self) {
        //todo: impl
    }
    pub fn imul(&mut self) {
        //todo: impl
    }
    pub fn lmul(&mut self) {
        //todo: impl
    }
    pub fn fmul(&mut self) {
        //todo: impl
    }
    pub fn dmul(&mut self) {
        //todo: impl
    }
    pub fn idiv(&mut self) {
        //todo: impl
    }
    pub fn ldiv(&mut self) {
        //todo: impl
    }
    pub fn fdiv(&mut self) {
        //todo: impl
    }
    pub fn ddiv(&mut self) {
        //todo: impl
    }
    pub fn irem(&mut self) {
        //todo: impl
    }
    pub fn lrem(&mut self) {
        //todo: impl
    }
    pub fn frem(&mut self) {
        //todo: impl
    }
    pub fn drem(&mut self) {
        //todo: impl
    }
    pub fn ineg(&mut self) {
        //todo: impl
    }
    pub fn lneg(&mut self) {
        //todo: impl
    }
    pub fn fneg(&mut self) {
        //todo: impl
    }
    pub fn dneg(&mut self) {
        //todo: impl
    }
    pub fn ishl(&mut self) {
        //todo: impl
    }
    pub fn lshl(&mut self) {
        //todo: impl
    }
    pub fn ishr(&mut self) {
        //todo: impl
    }
    pub fn lshr(&mut self) {
        //todo: impl
    }
    pub fn iushr(&mut self) {
        //todo: impl
    }
    pub fn lushr(&mut self) {
        //todo: impl
    }
    pub fn iand(&mut self) {
        //todo: impl
    }
    pub fn land(&mut self) {
        //todo: impl
    }
    pub fn ior(&mut self) {
        //todo: impl
    }
    pub fn lor(&mut self) {
        //todo: impl
    }
    pub fn ixor(&mut self) {
        //todo: impl
    }
    pub fn lxor(&mut self) {
        //todo: impl
    }
    pub fn iinc(&mut self) {
        //todo: impl
    }
    pub fn i2l(&mut self) {
        //todo: impl
    }
    pub fn i2f(&mut self) {
        //todo: impl
    }
    pub fn i2d(&mut self) {
        //todo: impl
    }
    pub fn l2i(&mut self) {
        //todo: impl
    }
    pub fn l2f(&mut self) {
        //todo: impl
    }
    pub fn l2d(&mut self) {
        //todo: impl
    }
    pub fn f2i(&mut self) {
        //todo: impl
    }
    pub fn f2l(&mut self) {
        //todo: impl
    }
    pub fn f2d(&mut self) {
        //todo: impl
    }
    pub fn d2i(&mut self) {
        //todo: impl
    }
    pub fn d2l(&mut self) {
        //todo: impl
    }
    pub fn d2f(&mut self) {
        //todo: impl
    }
    pub fn i2b(&mut self) {
        //todo: impl
    }
    pub fn i2c(&mut self) {
        //todo: impl
    }
    pub fn i2s(&mut self) {
        //todo: impl
    }
    pub fn lcmp(&mut self) {
        //todo: impl
    }
    pub fn fcmpl(&mut self) {
        //todo: impl
    }
    pub fn fcmpg(&mut self) {
        //todo: impl
    }
    pub fn dcmpl(&mut self) {
        //todo: impl
    }
    pub fn dcmpg(&mut self) {
        //todo: impl
    }
    pub fn ifeq(&mut self) {
        //todo: impl
    }
    pub fn ifne(&mut self) {
        //todo: impl
    }
    pub fn iflt(&mut self) {
        //todo: impl
    }
    pub fn ifge(&mut self) {
        //todo: impl
    }
    pub fn ifgt(&mut self) {
        //todo: impl
    }
    pub fn ifle(&mut self) {
        //todo: impl
    }
    pub fn if_icmpeq(&mut self) {
        //todo: impl
    }
    pub fn if_icmpne(&mut self) {
        //todo: impl
    }
    pub fn if_icmplt(&mut self) {
        //todo: impl
    }
    pub fn if_icmpge(&mut self) {
        //todo: impl
    }
    pub fn if_icmpgt(&mut self) {
        //todo: impl
    }
    pub fn if_icmple(&mut self) {
        //todo: impl
    }
    pub fn if_acmpeq(&mut self) {
        //todo: impl
    }
    pub fn if_acmpne(&mut self) {
        //todo: impl
    }
    pub fn goto(&mut self) {
        //todo: impl
    }
    pub fn jsr(&mut self) {
        //todo: impl
    }
    pub fn ret(&mut self) {
        //todo: impl
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
        //todo: impl
    }

    pub fn if_non_null(&mut self) {
        //todo: impl
    }
    pub fn goto_w(&mut self) {
        //todo: impl
    }
    pub fn jsr_w(&mut self) {
        //todo: impl
    }
    pub fn other_wise(&mut self) {
        //todo: impl
    }
}
