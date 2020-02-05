use crate::classfile::constant_pool::{self, ConstantType};
use crate::classfile::consts;
use crate::classfile::consts::J_STRING;
use crate::classfile::opcode::OpCode;
use crate::classfile::types::*;
use crate::classfile::ClassFile;
use crate::oop::{
    self, consts as oop_consts, field, ClassRef, MethodIdRef, Oop, OopDesc, OopRef, ValueType,
};
use crate::runtime::{
    self, cmp, require_class, require_class2, require_class3, Exception, JavaCall, JavaThread,
    Local, Stack,
};
use crate::util;
use bytes::{BigEndian, Bytes};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub struct Frame {
    class: ClassRef,
    //avoid lock class to access cp
    cp: ConstantPool,
    pub mir: MethodIdRef,
    code: Arc<Vec<U1>>,

    pub local: Local,
    pub stack: Stack,
    pub pc: i32,
    pub return_v: Option<OopRef>,

    pub meet_ex_here: bool,
}

//new
impl Frame {
    pub fn new(mir: MethodIdRef) -> Self {
        let class = mir.method.class.clone();
        let cp = {
            let class = class.lock().unwrap();
            match &class.kind {
                oop::ClassKind::Instance(cls_obj) => cls_obj.class_file.cp.clone(),
                _ => unreachable!(),
            }
        };
        match &mir.method.code {
            Some(code) => {
                //                trace!("local size = {}, stack_size = {}", code.max_locals, code.max_stack);
                let local = Local::new(code.max_locals as usize);
                let stack = Stack::new(code.max_stack as usize);
                let code = code.code.clone();
                Self {
                    class,
                    cp,
                    mir,
                    code,
                    local,
                    stack,
                    pc: 0,
                    return_v: None,
                    meet_ex_here: false,
                }
            }

            None => Self {
                class,
                cp: Arc::new(Box::new(Vec::new())),
                mir,
                code: Arc::new(vec![]),
                local: Local::new(0),
                stack: Stack::new(0),
                pc: 0,
                return_v: None,
                meet_ex_here: false,
            },
        }
    }
}

impl Frame {
    pub fn interp(&mut self, thread: &mut JavaThread) {
        //for debug
        let method = self.mir.method.get_id();
        let method = String::from_utf8_lossy(method.as_slice());

        loop {
            let code = self.read_opcode();
            match code {
                Some(code) => {
                    let op_code = OpCode::from(*code);
                    trace!("interp: {:?} ({}) {}", op_code, *code, method);
                    match op_code {
                        OpCode::ireturn => {
                            self.ireturn();
                            break;
                        }
                        OpCode::lreturn => {
                            self.lreturn();
                            break;
                        }
                        OpCode::freturn => {
                            self.freturn();
                            break;
                        }
                        OpCode::dreturn => {
                            self.dreturn();
                            break;
                        }
                        OpCode::areturn => {
                            self.areturn();
                            break;
                        }
                        OpCode::return_void => {
                            self.return_void();
                            break;
                        }
                        OpCode::nop => self.nop(),
                        OpCode::aconst_null => self.aconst_null(),
                        OpCode::iconst_m1 => self.iconst_m1(),
                        OpCode::iconst_0 => self.iconst_0(),
                        OpCode::iconst_1 => self.iconst_1(),
                        OpCode::iconst_2 => self.iconst_2(),
                        OpCode::iconst_3 => self.iconst_3(),
                        OpCode::iconst_4 => self.iconst_4(),
                        OpCode::iconst_5 => self.iconst_5(),
                        OpCode::lconst_0 => self.lconst_0(),
                        OpCode::lconst_1 => self.lconst_1(),
                        OpCode::fconst_0 => self.fconst_0(),
                        OpCode::fconst_1 => self.fconst_1(),
                        OpCode::fconst_2 => self.fconst_2(),
                        OpCode::dconst_0 => self.dconst_0(),
                        OpCode::dconst_1 => self.dconst_1(),
                        OpCode::bipush => self.bipush(),
                        OpCode::sipush => self.sipush(),
                        OpCode::ldc => self.ldc(thread),
                        OpCode::ldc_w => self.ldc_w(thread),
                        OpCode::ldc2_w => self.ldc2_w(thread),
                        OpCode::iload => self.iload(),
                        OpCode::lload => self.lload(),
                        OpCode::fload => self.fload(),
                        OpCode::dload => self.dload(),
                        OpCode::aload => self.aload(),
                        OpCode::iload_0 => self.iload_0(),
                        OpCode::iload_1 => self.iload_1(),
                        OpCode::iload_2 => self.iload_2(),
                        OpCode::iload_3 => self.iload_3(),
                        OpCode::lload_0 => self.lload_0(),
                        OpCode::lload_1 => self.lload_1(),
                        OpCode::lload_2 => self.lload_2(),
                        OpCode::lload_3 => self.lload_3(),
                        OpCode::fload_0 => self.fload_0(),
                        OpCode::fload_1 => self.fload_1(),
                        OpCode::fload_2 => self.fload_2(),
                        OpCode::fload_3 => self.fload_3(),
                        OpCode::dload_0 => self.dload_0(),
                        OpCode::dload_1 => self.dload_1(),
                        OpCode::dload_2 => self.dload_2(),
                        OpCode::dload_3 => self.dload_3(),
                        OpCode::aload_0 => self.aload_0(),
                        OpCode::aload_1 => self.aload_1(),
                        OpCode::aload_2 => self.aload_2(),
                        OpCode::aload_3 => self.aload_3(),
                        OpCode::iaload => self.iaload(thread),
                        OpCode::laload => self.laload(thread),
                        OpCode::faload => self.faload(thread),
                        OpCode::daload => self.daload(thread),
                        OpCode::aaload => self.aaload(thread),
                        OpCode::baload => self.baload(thread),
                        OpCode::caload => self.caload(thread),
                        OpCode::saload => self.saload(thread),
                        OpCode::istore => self.istore(),
                        OpCode::lstore => self.lstore(),
                        OpCode::fstore => self.fstore(),
                        OpCode::dstore => self.dstore(),
                        OpCode::astore => self.astore(),
                        OpCode::istore_0 => self.istore_0(),
                        OpCode::istore_1 => self.istore_1(),
                        OpCode::istore_2 => self.istore_2(),
                        OpCode::istore_3 => self.istore_3(),
                        OpCode::lstore_0 => self.lstore_0(),
                        OpCode::lstore_1 => self.lstore_1(),
                        OpCode::lstore_2 => self.lstore_2(),
                        OpCode::lstore_3 => self.lstore_3(),
                        OpCode::fstore_0 => self.fstore_0(),
                        OpCode::fstore_1 => self.fstore_1(),
                        OpCode::fstore_2 => self.fstore_2(),
                        OpCode::fstore_3 => self.fstore_3(),
                        OpCode::dstore_0 => self.dstore_0(),
                        OpCode::dstore_1 => self.dstore_1(),
                        OpCode::dstore_2 => self.dstore_2(),
                        OpCode::dstore_3 => self.dstore_3(),
                        OpCode::astore_0 => self.astore_0(),
                        OpCode::astore_1 => self.astore_1(),
                        OpCode::astore_2 => self.astore_2(),
                        OpCode::astore_3 => self.astore_3(),
                        OpCode::iastore => self.iastore(thread),
                        OpCode::lastore => self.lastore(thread),
                        OpCode::fastore => self.fastore(thread),
                        OpCode::dastore => self.dastore(thread),
                        OpCode::aastore => self.aastore(thread),
                        OpCode::bastore => self.bastore(thread),
                        OpCode::castore => self.castore(thread),
                        OpCode::sastore => self.sastore(thread),
                        OpCode::pop => self.pop(),
                        OpCode::pop2 => self.pop2(),
                        OpCode::dup => self.dup(),
                        OpCode::dup_x1 => self.dup_x1(),
                        OpCode::dup_x2 => self.dup_x2(),
                        OpCode::dup2 => self.dup2(),
                        OpCode::dup2_x1 => self.dup2_x1(),
                        OpCode::dup2_x2 => self.dup2_x2(),
                        OpCode::swap => self.swap(),
                        OpCode::iadd => self.iadd(),
                        OpCode::ladd => self.ladd(),
                        OpCode::fadd => self.fadd(),
                        OpCode::dadd => self.dadd(),
                        OpCode::isub => self.isub(),
                        OpCode::lsub => self.lsub(),
                        OpCode::fsub => self.fsub(),
                        OpCode::dsub => self.dsub(),
                        OpCode::imul => self.imul(),
                        OpCode::lmul => self.lmul(),
                        OpCode::fmul => self.fmul(),
                        OpCode::dmul => self.dmul(),
                        OpCode::idiv => self.idiv(thread),
                        OpCode::ldiv => self.ldiv(thread),
                        OpCode::fdiv => self.fdiv(thread),
                        OpCode::ddiv => self.ddiv(thread),
                        OpCode::irem => self.irem(thread),
                        OpCode::lrem => self.lrem(thread),
                        OpCode::frem => self.frem(),
                        OpCode::drem => self.drem(),
                        OpCode::ineg => self.ineg(),
                        OpCode::lneg => self.lneg(),
                        OpCode::fneg => self.fneg(),
                        OpCode::dneg => self.dneg(),
                        OpCode::ishl => self.ishl(),
                        OpCode::lshl => self.lshl(),
                        OpCode::ishr => self.ishr(),
                        OpCode::lshr => self.lshr(),
                        OpCode::iushr => self.iushr(),
                        OpCode::lushr => self.lushr(),
                        OpCode::iand => self.iand(),
                        OpCode::land => self.land(),
                        OpCode::ior => self.ior(),
                        OpCode::lor => self.lor(),
                        OpCode::ixor => self.ixor(),
                        OpCode::lxor => self.lxor(),
                        OpCode::iinc => self.iinc(),
                        OpCode::i2l => self.i2l(),
                        OpCode::i2f => self.i2f(),
                        OpCode::i2d => self.i2d(),
                        OpCode::l2i => self.l2i(),
                        OpCode::l2f => self.l2f(),
                        OpCode::l2d => self.l2d(),
                        OpCode::f2i => self.f2i(),
                        OpCode::f2l => self.f2l(),
                        OpCode::f2d => self.f2d(),
                        OpCode::d2i => self.d2i(),
                        OpCode::d2l => self.d2l(),
                        OpCode::d2f => self.d2f(),
                        OpCode::i2b => self.i2b(),
                        OpCode::i2c => self.i2c(),
                        OpCode::i2s => self.i2s(),
                        OpCode::lcmp => self.lcmp(),
                        OpCode::fcmpl => self.fcmpl(),
                        OpCode::fcmpg => self.fcmpg(),
                        OpCode::dcmpl => self.dcmpl(),
                        OpCode::dcmpg => self.dcmpg(),
                        OpCode::ifeq => self.ifeq(),
                        OpCode::ifne => self.ifne(),
                        OpCode::iflt => self.iflt(),
                        OpCode::ifge => self.ifge(),
                        OpCode::ifgt => self.ifgt(),
                        OpCode::ifle => self.ifle(),
                        OpCode::if_icmpeq => self.if_icmpeq(),
                        OpCode::if_icmpne => self.if_icmpne(),
                        OpCode::if_icmplt => self.if_icmplt(),
                        OpCode::if_icmpge => self.if_icmpge(),
                        OpCode::if_icmpgt => self.if_icmpgt(),
                        OpCode::if_icmple => self.if_icmple(),
                        OpCode::if_acmpeq => self.if_acmpeq(),
                        OpCode::if_acmpne => self.if_acmpne(),
                        OpCode::goto => self.goto(),
                        OpCode::jsr => self.jsr(),
                        OpCode::ret => self.ret(),
                        OpCode::tableswitch => self.table_switch(),
                        OpCode::lookupswitch => self.lookup_switch(),
                        OpCode::getstatic => self.get_static(thread),
                        OpCode::putstatic => self.put_static(thread),
                        OpCode::getfield => self.get_field(thread),
                        OpCode::putfield => self.put_field(thread),
                        OpCode::invokevirtual => self.invoke_virtual(thread),
                        OpCode::invokespecial => self.invoke_special(thread),
                        OpCode::invokestatic => self.invoke_static(thread),
                        OpCode::invokeinterface => self.invoke_interface(thread),
                        OpCode::invokedynamic => self.invoke_dynamic(),
                        OpCode::new => self.new_(thread),
                        OpCode::newarray => self.new_array(thread),
                        OpCode::anewarray => self.anew_array(thread),
                        OpCode::arraylength => self.array_length(thread),
                        OpCode::athrow => self.athrow(thread),
                        OpCode::checkcast => self.check_cast(thread),
                        OpCode::instanceof => self.instance_of(),
                        OpCode::monitorenter => self.monitor_enter(thread),
                        OpCode::monitorexit => self.monitor_exit(thread),
                        OpCode::wide => self.wide(),
                        OpCode::multianewarray => self.multi_anew_array(),
                        OpCode::ifnull => self.if_null(),
                        OpCode::ifnonnull => self.if_non_null(),
                        OpCode::goto_w => self.goto_w(),
                        OpCode::jsr_w => self.jsr_w(),
                        _ => (),
                    }
                }
                None => break,
            }

            if thread.is_meet_ex() {
                break;
            }
        }
    }
}

//helper methods
impl Frame {
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

    fn read_byte(&mut self) -> u8 {
        let v = self.code[self.pc as usize];
        self.pc += 1;
        v
    }

    fn read_opcode(&mut self) -> Option<&U1> {
        let v = self.code.get(self.pc as usize);
        self.pc += 1;
        v
    }

    fn read_u2(&mut self) -> usize {
        self.read_u1() << 8 | self.read_u1()
    }

    fn load_constant(&mut self, pos: usize, thread: &mut JavaThread) {
        match &self.cp[pos] {
            ConstantType::Integer { v } => self.stack.push_int2(*v),
            ConstantType::Float { v } => self.stack.push_float2(*v),
            ConstantType::Long { v } => self.stack.push_long2(*v),
            ConstantType::Double { v } => self.stack.push_double2(*v),
            ConstantType::String { string_index } => {
                let s = constant_pool::get_utf8(&self.cp, *string_index as usize);
                self.stack.push_const_utf8(s.unwrap());
            }
            ConstantType::Class { name_index } => {
                let name = constant_pool::get_utf8(&self.cp, *name_index as usize).unwrap();
                let cl = { self.class.lock().unwrap().class_loader.clone() };
                trace!(
                    "load_constant name={}",
                    String::from_utf8_lossy(name.as_slice())
                );
                let class = runtime::require_class(cl, name).unwrap();

                {
                    let mut class = class.lock().unwrap();
                    class.init_class(thread);
                }

                oop::class::init_class_fully(thread, class.clone());

                let mirror = { class.lock().unwrap().get_mirror() };

                self.stack.push_ref(mirror);
            }
            _ => unreachable!(),
        }
    }

    /*
    fn handle_exception(&mut self, thread: &mut JavaThread) {
        self.stack.clear();
        let ext = thread.exception.clone();
        self.stack.push_ref(ext.unwrap());
        thread.clear_ext();
        self.athrow(thread);
    }
    */

    fn meet_ex(&mut self, jt: &mut JavaThread, cls_name: &'static [u8], msg: Option<String>) {
        let ex = Exception {
            cls_name,
            msg,
            ex_oop: None,
        };

        self.meet_ex_here = true;
        jt.set_ex(Some(ex));
    }

    fn goto_abs(&mut self, pc: i32) {
        self.pc = pc;
    }

    fn goto_by_offset(&mut self, branch: i32) {
        self.pc += branch;
    }

    fn goto_by_offset_with_occupied(&mut self, branch: i32, occupied: i32) {
        self.goto_by_offset(branch);
        self.goto_by_offset(-(occupied - 1));
    }

    fn goto_by_offset_hardcoded(&mut self, occupied: i32) {
        let high = self.code[self.pc as usize] as i16;
        let low = self.code[(self.pc + 1) as usize] as i16;
        let branch = (high << 8) | low;
        self.goto_by_offset_with_occupied(branch as i32, occupied);
    }

    fn goto_abs_with_occupied(&mut self, pc: i32, occupied: i32) {
        self.goto_abs(pc);
        self.goto_by_offset(-(occupied - 1));
    }

    fn set_return(&mut self, v: Option<OopRef>) {
        self.return_v = v;
    }

    fn get_field_helper(
        &mut self,
        thread: &mut JavaThread,
        receiver: OopRef,
        idx: i32,
        is_static: bool,
    ) {
        let fir = { field::get_field_ref(thread, &self.cp, idx as usize, is_static) };

        assert_eq!(fir.field.is_static(), is_static);

        trace!(
            "get_field_helper = {}, is_static = {}",
            String::from_utf8_lossy(fir.field.get_id().as_slice()),
            is_static
        );

        let value_type = fir.field.value_type.clone();
        let class = fir.field.class.lock().unwrap();
        let v = if is_static {
            class.get_static_field_value(fir.clone())
        } else {
            class.get_field_value(receiver, fir.clone())
        };

        let v_ref = v.clone();
        let v = v.lock().unwrap();
        match value_type {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => match &v.v {
                Oop::Int(v) => self.stack.push_int(*v),
                _ => unreachable!(),
            },
            ValueType::FLOAT => match &v.v {
                Oop::Float(v) => self.stack.push_float(*v),
                _ => unreachable!(),
            },
            ValueType::DOUBLE => match &v.v {
                Oop::Double(v) => self.stack.push_double(*v),
                _ => unreachable!(),
            },
            ValueType::LONG => match &v.v {
                Oop::Long(v) => self.stack.push_long(*v),
                _ => unreachable!(),
            },
            ValueType::OBJECT | ValueType::ARRAY => self.stack.push_ref(v_ref),
            _ => unreachable!(),
        }
    }

    fn put_field_helper(&mut self, thread: &mut JavaThread, idx: i32, is_static: bool) {
        let fir = { field::get_field_ref(thread, &self.cp, idx as usize, is_static) };

        assert_eq!(fir.field.is_static(), is_static);

        trace!(
            "put_field_helper = {}, is_static = {}",
            String::from_utf8_lossy(fir.field.get_id().as_slice()),
            is_static
        );

        let value_type = fir.field.value_type.clone();

        let v = match value_type {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => {
                let v = self.stack.pop_int();
                OopDesc::new_int(v)
            }
            ValueType::FLOAT => {
                let v = self.stack.pop_float();
                OopDesc::new_float(v)
            }
            ValueType::DOUBLE => {
                let v = self.stack.pop_double();
                OopDesc::new_double(v)
            }
            ValueType::LONG => {
                let v = self.stack.pop_long();
                OopDesc::new_long(v)
            }
            ValueType::ARRAY | ValueType::OBJECT => self.stack.pop_ref(),
            _ => unreachable!(),
        };

        let mut class = fir.field.class.lock().unwrap();
        if is_static {
            class.put_static_field_value(fir.clone(), v);
        } else {
            let receiver = self.stack.pop_ref();
            if Arc::ptr_eq(&receiver, &oop_consts::get_null()) {
                self.meet_ex(thread, consts::J_NPE, None);
            } else {
                //                trace!("put_field = {}", String::from_utf8_lossy(fir.field.get_id().as_slice()));
                class.put_field_value(receiver, fir.clone(), v);
            }
        }
    }

    fn invoke_helper(&mut self, jt: &mut JavaThread, is_static: bool, idx: usize) {
        let mir = { oop::method::get_method_ref(jt, &self.cp, idx) };

        match mir {
            Ok(mir) => {
                assert_eq!(mir.method.is_static(), is_static);

                match runtime::java_call::JavaCall::new(jt, &mut self.stack, mir) {
                    Ok(mut jc) => jc.invoke(jt, &mut self.stack),

                    //ignored, let interp main loop handle exception
                    _ => (),
                }
            }
            Err(_) => unreachable!("NotFound method"),
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
        self.stack.push_const0(false);
    }

    pub fn lconst_0(&mut self) {
        self.stack.push_const0(true);
    }

    pub fn fconst_0(&mut self) {
        self.stack.push_const0(false);
    }

    pub fn dconst_0(&mut self) {
        self.stack.push_const0(true);
    }

    pub fn iconst_1(&mut self) {
        self.stack.push_const1(false);
    }

    pub fn lconst_1(&mut self) {
        self.stack.push_const1(true);
    }

    pub fn fconst_1(&mut self) {
        self.stack.push_const1(false);
    }

    pub fn dconst_1(&mut self) {
        self.stack.push_const1(true);
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

    pub fn ldc(&mut self, thread: &mut JavaThread) {
        let pos = self.read_u1();
        self.load_constant(pos, thread);
    }

    pub fn ldc_w(&mut self, thread: &mut JavaThread) {
        let pos = self.read_u2();
        self.load_constant(pos, thread);
    }

    pub fn ldc2_w(&mut self, thread: &mut JavaThread) {
        self.ldc_w(thread);
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

    pub fn iaload(&mut self, thread: &mut JavaThread) {
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        let rf = rf.lock().unwrap();
        match &rf.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= len) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary.get_elm_at(pos as usize);
                    let v = v.lock().unwrap();
                    match &v.v {
                        Oop::Int(v) => {
                            self.stack.push_int(*v);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Oop::Str(s) => {
                let len = s.len();
                if (pos < 0) || (pos as usize >= len) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = s.as_slice()[pos as usize];
                    self.stack.push_int(v as i32);
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn saload(&mut self, thread: &mut JavaThread) {
        self.iaload(thread);
    }

    pub fn caload(&mut self, thread: &mut JavaThread) {
        self.iaload(thread);
    }

    pub fn baload(&mut self, thread: &mut JavaThread) {
        self.iaload(thread);
    }

    pub fn laload(&mut self, thread: &mut JavaThread) {
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        let rf = rf.lock().unwrap();
        match &rf.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary.get_elm_at(pos as usize);
                    let v = v.lock().unwrap();
                    match &v.v {
                        Oop::Long(v) => {
                            self.stack.push_long(*v);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn faload(&mut self, thread: &mut JavaThread) {
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        let rf = rf.lock().unwrap();
        match &rf.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary.get_elm_at(pos as usize);
                    let v = v.lock().unwrap();
                    match &v.v {
                        Oop::Float(v) => {
                            self.stack.push_float(*v);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn daload(&mut self, thread: &mut JavaThread) {
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        let rf = rf.lock().unwrap();
        match &rf.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary.get_elm_at(pos as usize);
                    let v = v.lock().unwrap();
                    match &v.v {
                        Oop::Double(v) => {
                            self.stack.push_double(*v);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn aaload(&mut self, thread: &mut JavaThread) {
        let pos = self.stack.pop_int();
        let rf = self.stack.pop_ref();
        let rf = rf.lock().unwrap();
        match &rf.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= len) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary.get_elm_at(pos as usize);
                    self.stack.push_ref(v);
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
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

    pub fn bastore(&mut self, thread: &mut JavaThread) {
        self.iastore(thread);
    }

    pub fn castore(&mut self, thread: &mut JavaThread) {
        self.iastore(thread);
    }

    pub fn sastore(&mut self, thread: &mut JavaThread) {
        self.iastore(thread);
    }

    pub fn iastore(&mut self, thread: &mut JavaThread) {
        let v = self.stack.pop_int();
        let pos = self.stack.pop_int();
        let mut rf = self.stack.pop_ref();
        let mut rff = rf.lock().unwrap();
        match &mut rff.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = OopDesc::new_int(v);
                    ary.set_elm_at(pos as usize, v);
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn lastore(&mut self, thread: &mut JavaThread) {
        let v = self.stack.pop_long();
        let pos = self.stack.pop_int();
        let mut rf = self.stack.pop_ref();
        let mut rff = rf.lock().unwrap();
        match &mut rff.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = OopDesc::new_long(v);
                    ary.set_elm_at(pos as usize, v);
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn fastore(&mut self, thread: &mut JavaThread) {
        let v = self.stack.pop_float();
        let pos = self.stack.pop_int();
        let mut rf = self.stack.pop_ref();
        let mut rff = rf.lock().unwrap();
        match &mut rff.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = OopDesc::new_float(v);
                    ary.set_elm_at(pos as usize, v);
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn dastore(&mut self, thread: &mut JavaThread) {
        let v = self.stack.pop_double();
        let pos = self.stack.pop_int();
        let mut rf = self.stack.pop_ref();
        let mut rff = rf.lock().unwrap();
        match &mut rff.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = OopDesc::new_double(v);
                    ary.set_elm_at(pos as usize, v);
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn aastore(&mut self, thread: &mut JavaThread) {
        let v = self.stack.pop_ref();
        let pos = self.stack.pop_int();
        let mut rf = self.stack.pop_ref();
        let mut rff = rf.lock().unwrap();
        match &mut rff.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                if (pos < 0) || (pos as usize >= ary.get_length()) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    self.meet_ex(thread, consts::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    ary.set_elm_at(pos as usize, v);
                }
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn pop(&mut self) {
        self.stack.drop_top();
    }

    pub fn pop2(&mut self) {
        self.stack.drop_top();
        self.stack.drop_top();
    }

    pub fn dup(&mut self) {
        self.stack.dup();
    }

    pub fn dup_x1(&mut self) {
        self.stack.dup_x1();
    }

    pub fn dup_x2(&mut self) {
        self.stack.dup_x2();
    }

    pub fn dup2(&mut self) {
        self.stack.dup2();
    }

    pub fn dup2_x1(&mut self) {
        self.stack.dup2_x1();
    }

    pub fn dup2_x2(&mut self) {
        self.stack.dup2_x2();
    }

    pub fn swap(&mut self) {
        self.stack.swap();
    }

    pub fn iadd(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        self.stack.push_int(v1 + v2);
    }

    pub fn ladd(&mut self) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        //        trace!("ladd v1 = {}, v2 = {}, r = {}", v1, v2, v1 + v2);
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

    pub fn idiv(&mut self, thread: &mut JavaThread) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v2 == 0 {
            self.meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            self.stack.push_int(v1 / v2);
        }
    }

    pub fn ldiv(&mut self, thread: &mut JavaThread) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        if v2 == 0 {
            self.meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            self.stack.push_long(v1 / v2);
        }
    }

    pub fn fdiv(&mut self, thread: &mut JavaThread) {
        let v2 = self.stack.pop_float();
        let v1 = self.stack.pop_float();
        if v2 == 0.0 {
            self.meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            self.stack.push_float(v1 / v2);
        }
    }

    pub fn ddiv(&mut self, thread: &mut JavaThread) {
        let v2 = self.stack.pop_double();
        let v1 = self.stack.pop_double();
        if v2 == 0.0 {
            self.meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            self.stack.push_double(v1 / v2);
        }
    }

    pub fn irem(&mut self, thread: &mut JavaThread) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v2 == 0 {
            self.meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            self.stack.push_int(v1 - (v1 / v2) * v2);
        }
    }

    pub fn lrem(&mut self, thread: &mut JavaThread) {
        let v2 = self.stack.pop_long();
        let v1 = self.stack.pop_long();
        if v2 == 0 {
            self.meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            self.stack.push_long(v1 - (v1 / v2) * v2);
        }
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
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn ifne(&mut self) {
        let v = self.stack.pop_int();
        if v != 0 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn iflt(&mut self) {
        let v = self.stack.pop_int();
        if v < 0 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn ifge(&mut self) {
        let v = self.stack.pop_int();
        if v >= 0 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn ifgt(&mut self) {
        let v = self.stack.pop_int();
        if v > 0 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn ifle(&mut self) {
        let v = self.stack.pop_int();
        if v <= 0 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpeq(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 == v2 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpne(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 != v2 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmplt(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 < v2 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpge(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 >= v2 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmpgt(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 > v2 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_icmple(&mut self) {
        let v2 = self.stack.pop_int();
        let v1 = self.stack.pop_int();
        if v1 <= v2 {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_acmpeq(&mut self) {
        let v2 = self.stack.pop_ref();
        let v1 = self.stack.pop_ref();
        if Arc::ptr_eq(&v1, &v2) {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn if_acmpne(&mut self) {
        let v2 = self.stack.pop_ref();
        let v1 = self.stack.pop_ref();
        if !Arc::ptr_eq(&v1, &v2) {
            self.goto_by_offset_hardcoded(2);
        } else {
            self.pc += 2;
        }
    }

    pub fn goto(&mut self) {
        self.goto_by_offset_hardcoded(2);
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
        let mut bc = self.pc - 1;
        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += (4 - bc % 4);
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;
        let default_byte = [
            self.code[ptr],
            self.code[ptr + 1],
            self.code[ptr + 2],
            self.code[ptr + 3],
        ];
        let default_byte = u32::from_be_bytes(default_byte);
        let low_byte = [
            self.code[ptr + 4],
            self.code[ptr + 5],
            self.code[ptr + 6],
            self.code[ptr + 7],
        ];
        let low_byte = u32::from_be_bytes(low_byte);
        let high_byte = [
            self.code[ptr + 8],
            self.code[ptr + 9],
            self.code[ptr + 10],
            self.code[ptr + 11],
        ];
        let high_byte = u32::from_be_bytes(high_byte);
        let num = high_byte - low_byte + 1;
        ptr += 12;

        // switch-case jump table
        let mut jump_table = Vec::with_capacity(num as usize);
        for pos in 0..num {
            let pos = [
                self.code[ptr],
                self.code[ptr + 1],
                self.code[ptr + 2],
                self.code[ptr + 3],
            ];
            let pos = u32::from_be_bytes(pos);
            let jump_pos = pos + origin_bc as u32;
            ptr += 4;
            jump_table.push(jump_pos);
        }
        // default
        jump_table.push(default_byte + origin_bc as u32);

        let top_value = self.stack.pop_int();
        if (top_value > (jump_table.len() - 1 + low_byte as usize) as i32)
            || top_value < low_byte as i32
        {
            self.goto_abs_with_occupied(*jump_table.last().unwrap() as i32, 1);
        } else {
            self.goto_abs_with_occupied(
                jump_table[(top_value - low_byte as i32) as usize] as i32,
                1,
            );
        }
    }

    pub fn lookup_switch(&mut self) {
        let mut bc = self.pc - 1;
        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += (4 - bc % 4);
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;

        let default_byte = [
            self.code[ptr],
            self.code[ptr + 1],
            self.code[ptr + 2],
            self.code[ptr + 3],
        ];
        let default_byte = u32::from_be_bytes(default_byte);
        let count = [
            self.code[ptr + 4],
            self.code[ptr + 5],
            self.code[ptr + 6],
            self.code[ptr + 7],
        ];
        let count = u32::from_be_bytes(count);
        ptr += 8;

        let mut jump_table: HashMap<u32, u32> = HashMap::new();
        for i in 0..count {
            let value = [
                self.code[ptr],
                self.code[ptr + 1],
                self.code[ptr + 2],
                self.code[ptr + 3],
            ];
            let value = u32::from_be_bytes(value);
            let position = [
                self.code[ptr + 4],
                self.code[ptr + 5],
                self.code[ptr + 6],
                self.code[ptr + 7],
            ];
            let position = u32::from_be_bytes(position) + origin_bc as u32;
            ptr += 8;
            jump_table.insert(value, position);
        }

        let top_value = self.stack.pop_int();
        match jump_table.get(&(top_value as u32)) {
            Some(position) => self.goto_abs_with_occupied(*position as i32, 1),
            None => self.goto_abs_with_occupied(default_byte as i32 + origin_bc, 1),
        }
    }

    pub fn ireturn(&mut self) {
        let v = self.stack.pop_int();
        let v = OopDesc::new_int(v);
        self.set_return(Some(v));
    }

    pub fn lreturn(&mut self) {
        let v = self.stack.pop_long();
        let v = OopDesc::new_long(v);
        self.set_return(Some(v));
    }

    pub fn freturn(&mut self) {
        let v = self.stack.pop_float();
        let v = OopDesc::new_float(v);
        self.set_return(Some(v));
    }

    pub fn dreturn(&mut self) {
        let v = self.stack.pop_double();
        let v = OopDesc::new_double(v);
        self.set_return(Some(v));
    }

    pub fn areturn(&mut self) {
        let v = self.stack.pop_ref();
        self.set_return(Some(v));
    }

    pub fn return_void(&mut self) {
        self.set_return(None);
    }

    pub fn get_static(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.get_field_helper(thread, oop_consts::get_null(), cp_idx, true);
    }

    pub fn put_static(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.put_field_helper(thread, cp_idx, true);
    }

    pub fn get_field(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        let rf = self.stack.pop_ref();
        if Arc::ptr_eq(&rf, &oop_consts::get_null()) {
            self.meet_ex(thread, consts::J_NPE, None);
        } else {
            self.get_field_helper(thread, rf, cp_idx, false);
        }
    }

    pub fn put_field(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.put_field_helper(thread, cp_idx, false);
    }

    pub fn invoke_virtual(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.invoke_helper(thread, false, cp_idx as usize);
    }

    pub fn invoke_special(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.invoke_helper(thread, false, cp_idx as usize);
    }

    pub fn invoke_static(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.invoke_helper(thread, true, cp_idx as usize);
    }

    pub fn invoke_interface(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        let _count = self.read_u1();
        let zero = self.read_u1();

        if zero != 0 {
            warn!("interpreter: invalid invokeinterface: the value of the fourth operand byte must always be zero.");
        }

        self.invoke_helper(thread, false, cp_idx as usize);
    }

    pub fn invoke_dynamic(&mut self) {
        //todo: impl
        unimplemented!()
    }

    pub fn new_(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();

        let class = {
            match runtime::require_class2(cp_idx as u16, &self.cp) {
                Some(class) => {
                    {
                        let mut class = class.lock().unwrap();
                        class.init_class(thread);
                    }

                    oop::class::init_class_fully(thread, class.clone());

                    class
                }
                None => unreachable!("Cannot get class info from constant pool"),
            }
        };

        let v = oop::OopDesc::new_inst(class);
        self.stack.push_ref(v);
    }

    pub fn new_array(&mut self, thread: &mut JavaThread) {
        let t = self.read_byte();
        let length = self.stack.pop_int();
        if length < 0 {
            self.meet_ex(thread, consts::J_NASE, Some("length < 0".to_string()));
        } else {
            let mut ary = Vec::with_capacity(length as usize);
            let ary_cls = match t {
                //boolean
                4 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_int(0));
                    }

                    require_class3(None, b"[Z").unwrap()
                }
                //char
                5 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_int(0));
                    }

                    require_class3(None, b"[C").unwrap()
                }
                //float
                6 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_float(0.0));
                    }

                    require_class3(None, b"[F").unwrap()
                }
                //double
                7 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_double(0.0));
                    }

                    require_class3(None, b"[D").unwrap()
                }
                //byte
                8 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_int(0));
                    }

                    require_class3(None, b"[B").unwrap()
                }
                //short
                9 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_int(0));
                    }

                    require_class3(None, b"[S").unwrap()
                }
                //int
                10 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_int(0));
                    }

                    require_class3(None, b"[I").unwrap()
                }
                //long
                11 => {
                    for _ in 0..length {
                        ary.push(OopDesc::new_long(0));
                    }

                    require_class3(None, b"[J").unwrap()
                }
                _ => unreachable!(),
            };

            let obj = OopDesc::new_ary2(ary_cls, ary);
            self.stack.push_ref(obj);
        }
    }

    pub fn anew_array(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        let length = self.stack.pop_int();
        if length < 0 {
            self.meet_ex(thread, consts::J_NASE, Some("length < 0".to_string()));
        } else {
            let class = match runtime::require_class2(cp_idx as u16, &self.cp) {
                Some(class) => class,
                None => panic!("Cannot get class info from constant pool"),
            };

            {
                let mut class = class.lock().unwrap();
                class.init_class(thread);
            }

            oop::class::init_class_fully(thread, class.clone());

            let (name, cl) = {
                let class = class.lock().unwrap();
                let t = class.get_class_kind_type();
                let name = match t {
                    oop::class::ClassKindType::Instance | oop::class::ClassKindType::ObjectAry => {
                        let mut v = Vec::with_capacity(class.name.len() + 2);
                        v.push(b'[');
                        v.extend_from_slice(class.name.as_slice());
                        v.push(b';');

                        v
                    }
                    oop::class::ClassKindType::TypAry => {
                        let mut v = Vec::with_capacity(class.name.len() + 1);
                        v.push(b'[');
                        v.extend_from_slice(class.name.as_slice());

                        v
                    }
                };

                let name = new_ref!(name);
                (name, class.class_loader.clone())
            };

            match runtime::require_class(cl, name) {
                Some(ary_cls_obj) => {
                    {
                        {
                            let mut class = ary_cls_obj.lock().unwrap();
                            class.init_class(thread);
                        }

                        oop::class::init_class_fully(thread, ary_cls_obj.clone());
                    }

                    let ary = OopDesc::new_ary(ary_cls_obj, length as usize);
                    self.stack.push_ref(ary);
                }
                None => unreachable!(),
            }
        }
    }

    pub fn array_length(&mut self, thread: &mut JavaThread) {
        let rf = self.stack.pop_ref();
        let rf = rf.lock().unwrap();
        match &rf.v {
            Oop::Array(ary) => {
                let len = ary.get_length();
                self.stack.push_int(len as i32);
            }
            Oop::Str(s) => {
                self.stack.push_int(s.len() as i32);
            }
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => unreachable!(),
        }
    }

    pub fn athrow(&mut self, thread: &mut JavaThread) {
        let rf = self.stack.pop_ref();
        let rf_ref = rf.clone();
        let rf = rf.lock().unwrap();
        match rf.v {
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => {
                unimplemented!()
                /*
                let handler = thread.try_handle_exception(rf_ref.clone());
                if handler > 0 {
                    trace!("athrow: exception handler found at offset: {}", handler);
                    self.stack.clear();
                    self.stack.push_ref(rf_ref);
                    self.goto_abs(handler);
                } else {
                    trace!("athrow: exception handler not found, rethrowing it to caller");
                    self.set_return(Some(rf_ref));
                }
                */
            }
        }
    }

    pub fn check_cast(&mut self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        let target_cls = require_class2(cp_idx as U2, &self.cp).unwrap();

        let rf = self.stack.pop_ref();
        let rf_back = rf.clone();
        let rff = rf.lock().unwrap();
        match &rff.v {
            Oop::Null => self.stack.push_ref(rf_back),
            Oop::Inst(inst) => {
                let obj_cls = inst.class.clone();
                let r = cmp::instance_of(obj_cls.clone(), target_cls.clone());
                if r {
                    self.stack.push_ref(rf_back);
                } else {
                    let s_name = { obj_cls.lock().unwrap().name.clone() };
                    let t_name = { target_cls.lock().unwrap().name.clone() };

                    let s_name = String::from_utf8_lossy(s_name.as_slice())
                        .replace(util::PATH_SEP_STR, util::DOT_STR);
                    let t_name = String::from_utf8_lossy(t_name.as_slice())
                        .replace(util::PATH_SEP_STR, util::DOT_STR);

                    let msg = format!("{} cannot be cast to {}", s_name, t_name);
                    self.meet_ex(thread, consts::J_CCE, Some(msg));
                }
            }
            Oop::Str(_) => {
                let obj_cls = require_class3(None, J_STRING).unwrap();
                let r = cmp::instance_of(obj_cls.clone(), target_cls.clone());
                if r {
                    self.stack.push_ref(rf_back);
                } else {
                    let s_name = { obj_cls.lock().unwrap().name.clone() };
                    let t_name = { target_cls.lock().unwrap().name.clone() };

                    let s_name = String::from_utf8_lossy(s_name.as_slice())
                        .replace(util::PATH_SEP_STR, util::DOT_STR);
                    let t_name = String::from_utf8_lossy(t_name.as_slice())
                        .replace(util::PATH_SEP_STR, util::DOT_STR);

                    let msg = format!("{} cannot be cast to {}", s_name, t_name);
                    self.meet_ex(thread, consts::J_CCE, Some(msg));
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn instance_of(&mut self) {
        let cp_idx = self.read_i2();
        let target_cls = require_class2(cp_idx as U2, &self.cp).unwrap();

        let rf = self.stack.pop_ref();
        let rff = rf.lock().unwrap();
        let result = match &rff.v {
            Oop::Null => false,
            Oop::Inst(inst) => {
                let obj_cls = inst.class.clone();
                cmp::instance_of(obj_cls, target_cls)
            }
            Oop::Str(_) => {
                let obj_cls = require_class3(None, J_STRING).unwrap();
                cmp::instance_of(obj_cls, target_cls)
            }
            _ => unimplemented!(),
        };

        if result {
            self.stack.push_const1(false);
        } else {
            self.stack.push_const0(false);
        }
    }

    pub fn monitor_enter(&mut self, thread: &mut JavaThread) {
        let mut rf = self.stack.pop_ref();
        let mut rff = rf.lock().unwrap();
        match rff.v {
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => {
                rff.monitor_enter();
            }
        }
    }

    pub fn monitor_exit(&mut self, thread: &mut JavaThread) {
        let mut rf = self.stack.pop_ref();
        let mut rff = rf.lock().unwrap();
        match rff.v {
            Oop::Null => {
                self.meet_ex(thread, consts::J_NPE, None);
            }
            _ => {
                rff.monitor_exit();
            }
        }
    }

    pub fn wide(&mut self) {
        panic!("Use of deprecated instruction wide, please check your Java compiler")
    }

    pub fn multi_anew_array(&mut self) {
        //todo: impl
        unimplemented!()
    }

    pub fn if_null(&mut self) {
        let v = self.stack.pop_ref();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Null => self.goto_by_offset_hardcoded(2),
            _ => self.pc += 2,
        }
    }

    pub fn if_non_null(&mut self) {
        let v = self.stack.pop_ref();
        let v = v.lock().unwrap();
        match v.v {
            Oop::Null => self.pc += 2,
            _ => self.goto_by_offset_hardcoded(2),
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
