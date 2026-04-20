use crate::oop::{Oop, TypeArrayDesc, ValueType};
use crate::runtime::local::Local;
use crate::runtime::stack::Stack;
use crate::runtime::{
    self, cmp, exception as runtime_exception, require_class, require_class2, thread, DataArea,
    Frame, JavaCall,
};
use crate::types::*;
use crate::util;
use classfile::{
    constant_pool::get_utf8 as get_cp_utf8, consts as cls_const, ClassFile, ConstantPool,
    ConstantPoolType, OpCode, U1, U2,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLockReadGuard};

mod arith_ops;
mod array_ops;
mod compare;
mod const_ops;
mod control_flow;
mod conversion;
mod exception;
mod field_ops;
mod load_store;
mod monitor_ops;
mod object_ops;
mod read;
mod stack_ops;

pub struct Interp<'a> {
    frame: RwLockReadGuard<'a, Box<Frame>>,
    local: RefCell<Local>,
    cp: ConstantPool,
    code: Arc<Vec<U1>>,
    op_widen: bool,
}

impl<'a> Interp<'a> {
    pub fn new(frame: RwLockReadGuard<'a, Box<Frame>>, local: Local) -> Self {
        let cp = frame.cp.clone();
        let code = frame.code.clone();
        Self {
            frame,
            local: RefCell::new(local),
            cp,
            code,
            op_widen: false,
        }
    }
}

impl<'a> Interp<'a> {
    pub fn run(&mut self) {
        let jt = runtime::thread::current_java_thread();
        let codes = self.code.clone();

        loop {
            let code = self::read::read_byte(&self.frame.pc, &codes);
            let code = OpCode::from(code);
            match code {
                OpCode::athrow => {
                    self.athrow(jt);
                    break;
                }
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
                OpCode::nop => (),
                OpCode::aconst_null => {
                    self.frame.area.stack.borrow_mut().push_null();
                }
                OpCode::iconst_m1 => {
                    self.frame.area.stack.borrow_mut().push_const_m1();
                }
                OpCode::iconst_0 => {
                    self.frame.area.stack.borrow_mut().push_const0(false);
                }
                OpCode::iconst_1 => {
                    self.frame.area.stack.borrow_mut().push_const1(false);
                }
                OpCode::iconst_2 => {
                    self.frame.area.stack.borrow_mut().push_const2();
                }
                OpCode::iconst_3 => {
                    self.frame.area.stack.borrow_mut().push_const3();
                }
                OpCode::iconst_4 => {
                    self.frame.area.stack.borrow_mut().push_const4();
                }
                OpCode::iconst_5 => {
                    self.frame.area.stack.borrow_mut().push_const5();
                }
                OpCode::lconst_0 => {
                    self.frame.area.stack.borrow_mut().push_const0(true);
                }
                OpCode::lconst_1 => {
                    self.frame.area.stack.borrow_mut().push_const1(true);
                }
                OpCode::fconst_0 => {
                    self.frame.area.stack.borrow_mut().push_const0(false);
                }
                OpCode::fconst_1 => {
                    self.frame.area.stack.borrow_mut().push_const1(false);
                }
                OpCode::fconst_2 => {
                    self.frame.area.stack.borrow_mut().push_const2();
                }
                OpCode::dconst_0 => {
                    self.frame.area.stack.borrow_mut().push_const0(true);
                }
                OpCode::dconst_1 => {
                    self.frame.area.stack.borrow_mut().push_const1(true);
                }
                OpCode::bipush => self.bipush(),
                OpCode::sipush => self.sipush(),
                OpCode::ldc => self.ldc(),
                OpCode::ldc_w => self.ldc_w(),
                OpCode::ldc2_w => self.ldc2_w(),
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
                OpCode::iaload => self.iaload(),
                OpCode::laload => self.laload(),
                OpCode::faload => self.faload(),
                OpCode::daload => self.daload(),
                OpCode::aaload => self.aaload(),
                OpCode::baload => self.baload(),
                OpCode::caload => self.caload(),
                OpCode::saload => self.saload(),
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
                OpCode::iastore => self.iastore(),
                OpCode::lastore => self.lastore(),
                OpCode::fastore => self.fastore(),
                OpCode::dastore => self.dastore(),
                OpCode::aastore => self.aastore(),
                OpCode::bastore => self.bastore(),
                OpCode::castore => self.castore(),
                OpCode::sastore => self.sastore(),
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
                OpCode::idiv => self.idiv(),
                OpCode::ldiv => self.ldiv(),
                OpCode::fdiv => self.fdiv(),
                OpCode::ddiv => self.ddiv(),
                OpCode::irem => self.irem(),
                OpCode::lrem => self.lrem(),
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
                OpCode::getstatic => self.get_static(),
                OpCode::putstatic => self.put_static(),
                OpCode::getfield => self.get_field(),
                OpCode::putfield => self.put_field(),
                OpCode::invokevirtual => self.invoke_virtual(),
                OpCode::invokespecial => self.invoke_special(),
                OpCode::invokestatic => self.invoke_static(),
                OpCode::invokeinterface => self.invoke_interface(),
                OpCode::invokedynamic => self.invoke_dynamic(),
                OpCode::new => self.new_(),
                OpCode::newarray => self.new_array(),
                OpCode::anewarray => self.anew_array(),
                OpCode::arraylength => self.array_length(),
                OpCode::checkcast => self.check_cast(),
                OpCode::instanceof => self.instance_of(),
                OpCode::monitorenter => self.monitor_enter(),
                OpCode::monitorexit => self.monitor_exit(),
                OpCode::wide => self.wide(),
                OpCode::multianewarray => self.multi_anew_array(),
                OpCode::ifnull => self.if_null(),
                OpCode::ifnonnull => self.if_non_null(),
                OpCode::goto_w => self.goto_w(),
                OpCode::jsr_w => self.jsr_w(),
                _ => unreachable!(),
            }

            let is_meet_ex = thread::is_meet_ex();
            if is_meet_ex {
                let mut th = jt.write().unwrap();
                let ex = th.take_ex().unwrap();
                drop(th);
                match self.try_handle_exception(ex) {
                    Ok(_) => (),
                    Err(ex) => {
                        let mut th = jt.write().unwrap();
                        th.set_ex(ex);
                        break;
                    }
                }
            }
        }
    }
}
