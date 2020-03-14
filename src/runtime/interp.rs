use crate::classfile::constant_pool::{self, get_class_name, get_utf8, Type};
use crate::classfile::consts;
use crate::classfile::consts::J_STRING;
use crate::classfile::opcode::OpCode;
use crate::classfile::ClassFile;
use crate::oop::{self, consts as oop_consts, field, ClassKind, Oop, TypeArrayDesc, ValueType};
use crate::runtime::local::Local;
use crate::runtime::stack::Stack;
use crate::runtime::{
    self, cmp, exception, require_class, require_class2, require_class3, DataArea, Frame, FrameRef,
    JavaCall, JavaThread,
};
use crate::types::*;
use crate::util;
use bytes::Bytes;
use nix::sys::socket::SockType::Datagram;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLockReadGuard};

macro_rules! array_store {
    ($thread:ident, $ary:ident, $pos:ident, $v:ident) => {
        let len = $ary.len();
        if ($pos < 0) || ($pos as usize >= len) {
            let msg = format!("length is {}, but index is {}", len, $pos);
            exception::meet_ex(
                $thread,
                crate::classfile::consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                Some(msg),
            );
        } else {
            $ary[$pos as usize] = $v;
        }
    };
}

macro_rules! iarray_load {
    ($thread:ident, $area:ident, $ary:ident, $pos:ident) => {
        let len = $ary.len();
        if ($pos < 0) || ($pos as usize >= len) {
            drop($area);
            let msg = format!("length is {}, but index is {}", len, $pos);
            exception::meet_ex(
                $thread,
                crate::classfile::consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                Some(msg),
            );
        } else {
            let stack = &mut ($area.stack);
            stack.push_int($ary[$pos as usize] as i32);
        }
    };
}

pub struct Interp<'a> {
    frame: RwLockReadGuard<'a, Box<Frame>>,
}

impl<'a> Interp<'a> {
    pub fn new(frame: RwLockReadGuard<'a, Box<Frame>>) -> Self {
        Self { frame }
    }
}

impl<'a> Interp<'a> {
    fn debug_op(&self, code: u8, op: OpCode) {
        let frame_id = self.frame.frame_id;
        //for debug
        let cls_name = { self.frame.mir.method.class.read().unwrap().name.clone() };
        let cls_name = unsafe { std::str::from_utf8_unchecked(cls_name.as_slice()) };
        let method = self.frame.mir.method.get_id();
        let method = unsafe { std::str::from_utf8_unchecked(method.as_slice()) };

        trace!(
            "interp: {:?} ({}/{}) {}:{}",
            op,
            code,
            frame_id,
            cls_name,
            method
        );
    }
}

impl<'a> Interp<'a> {
    pub fn run(&self, thread: &mut JavaThread) {
        loop {
            let code = self.read_opcode();
            match code {
                Some(code) => {
                    let op_code = OpCode::from(*code);
                    self.debug_op(*code, op_code);

                    match op_code {
                        OpCode::athrow => {
                            self.athrow(thread);
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
                        OpCode::checkcast => self.check_cast(thread),
                        OpCode::instanceof => self.instance_of(thread),
                        OpCode::monitorenter => self.monitor_enter(thread),
                        OpCode::monitorexit => self.monitor_exit(thread),
                        OpCode::wide => self.wide(),
                        OpCode::multianewarray => self.multi_anew_array(),
                        OpCode::ifnull => self.if_null(),
                        OpCode::ifnonnull => self.if_non_null(),
                        OpCode::goto_w => self.goto_w(),
                        OpCode::jsr_w => self.jsr_w(),
                        _ => unreachable!(),
                    }

                    if thread.is_meet_ex() {
                        // util::debug::print_stack_trace(thread);
                        let ex = thread.take_ex().unwrap();
                        match self.try_handle_exception(thread, ex) {
                            Ok(_) => (),
                            Err(ex) => {
                                thread.set_ex(ex);
                                break;
                            }
                        }
                    }
                }

                None => break,
            }
        }
    }
}

//helper methods
impl<'a> Interp<'a> {
    fn read_i2(&self) -> i32 {
        let h = self.read_byte() as i16;
        let l = self.read_byte() as i16;
        (h << 8 | l) as i32
    }

    fn read_u1(&self) -> usize {
        let mut area = self.frame.area.borrow_mut();
        let v = self.frame.code[area.pc as usize];
        area.pc += 1;
        v as usize
    }

    fn read_byte(&self) -> u8 {
        let mut area = self.frame.area.borrow_mut();
        let v = self.frame.code[area.pc as usize];
        area.pc += 1;
        v
    }

    fn read_opcode(&self) -> Option<&U1> {
        let mut area = self.frame.area.borrow_mut();
        let v = self.frame.code.get(area.pc as usize);
        area.pc += 1;
        v
    }

    fn read_u2(&self) -> usize {
        self.read_u1() << 8 | self.read_u1()
    }

    fn load_constant(&self, pos: usize, thread: &mut JavaThread) {
        match &self.frame.cp[pos] {
            Type::Integer { v } => {
                let mut area = self.frame.area.borrow_mut();
                area.stack.push_int2(*v)
            }
            Type::Float { v } => {
                let mut area = self.frame.area.borrow_mut();
                area.stack.push_float2(*v)
            }
            Type::Long { v } => {
                let mut area = self.frame.area.borrow_mut();
                area.stack.push_long2(*v)
            }
            Type::Double { v } => {
                let mut area = self.frame.area.borrow_mut();
                area.stack.push_double2(*v)
            }
            Type::String { string_index } => {
                let s = constant_pool::get_utf8(&self.frame.cp, *string_index as usize).unwrap();
                let s = util::oop::new_java_lang_string3(thread, s.as_slice());

                let mut area = self.frame.area.borrow_mut();
                area.stack.push_ref(s);
            }
            Type::Class { name_index } => {
                let name = constant_pool::get_utf8(&self.frame.cp, *name_index as usize).unwrap();
                let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
                let cl = { self.frame.class.read().unwrap().class_loader.clone() };
                trace!("load_constant name={}, cl={:?}", name, cl);
                let class = runtime::require_class3(cl, name.as_bytes()).unwrap();

                {
                    let mut class = class.write().unwrap();
                    class.init_class(thread);
                }

                oop::class::init_class_fully(thread, class.clone());

                let mirror = { class.read().unwrap().get_mirror() };

                let mut area = self.frame.area.borrow_mut();
                area.stack.push_ref(mirror);
            }
            _ => unreachable!(),
        }
    }

    fn goto_abs(&self, pc: i32) {
        let mut area = self.frame.area.borrow_mut();
        area.pc = pc;
    }

    fn goto_by_offset(&self, branch: i32) {
        let mut area = self.frame.area.borrow_mut();
        area.pc += branch;
    }

    fn goto_by_offset_with_occupied(&self, branch: i32, occupied: i32) {
        self.goto_by_offset(branch);
        self.goto_by_offset(-(occupied - 1));
    }

    fn goto_by_offset_hardcoded(&self, occupied: i32) {
        let area = self.frame.area.borrow();
        let high = self.frame.code[area.pc as usize] as i16;
        let low = self.frame.code[(area.pc + 1) as usize] as i16;
        let branch = (high << 8) | low;
        drop(area);

        self.goto_by_offset_with_occupied(branch as i32, occupied);
    }

    fn goto_abs_with_occupied(&self, pc: i32, occupied: i32) {
        self.goto_abs(pc);
        self.goto_by_offset(-(occupied - 1));
    }

    fn set_return(&self, v: Option<Oop>) {
        let mut area = self.frame.area.borrow_mut();
        area.return_v = v;
    }

    fn get_field_helper(&self, thread: &mut JavaThread, receiver: Oop, idx: i32, is_static: bool) {
        let fir = { field::get_field_ref(thread, &self.frame.cp, idx as usize, is_static) };

        assert_eq!(fir.field.is_static(), is_static);

        let fid = fir.field.get_id();
        trace!(
            "get_field_helper = {}, is_static = {}",
            unsafe { std::str::from_utf8_unchecked(fid.as_slice()) },
            is_static
        );

        let value_type = fir.field.value_type.clone();
        let class = fir.field.class.read().unwrap();
        let v = if is_static {
            class.get_static_field_value(fir.clone())
        } else {
            class.get_field_value(&receiver, fir.clone())
        };

        match value_type {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => match v {
                Oop::Int(v) => {
                    let mut area = self.frame.area.borrow_mut();
                    area.stack.push_int(v)
                }
                t => unreachable!("t = {:?}", t),
            },
            ValueType::FLOAT => match v {
                Oop::Float(v) => {
                    let mut area = self.frame.area.borrow_mut();
                    area.stack.push_float(v)
                }
                _ => unreachable!(),
            },
            ValueType::DOUBLE => match v {
                Oop::Double(v) => {
                    let mut area = self.frame.area.borrow_mut();
                    area.stack.push_double(v)
                }
                _ => unreachable!(),
            },
            ValueType::LONG => match v {
                Oop::Long(v) => {
                    let mut area = self.frame.area.borrow_mut();
                    area.stack.push_long(v)
                }
                _ => unreachable!(),
            },
            ValueType::OBJECT | ValueType::ARRAY => {
                let mut area = self.frame.area.borrow_mut();
                area.stack.push_ref(v)
            }
            _ => unreachable!(),
        }
    }

    fn put_field_helper(&self, thread: &mut JavaThread, idx: i32, is_static: bool) {
        let fir = { field::get_field_ref(thread, &self.frame.cp, idx as usize, is_static) };

        assert_eq!(fir.field.is_static(), is_static);

        let fid = fir.field.get_id();
        trace!(
            "put_field_helper={}, is_static={}",
            unsafe { std::str::from_utf8_unchecked(fid.as_slice()) },
            is_static
        );

        let value_type = fir.field.value_type.clone();
        //        info!("value_type = {:?}", value_type);
        let v = match value_type {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => {
                let mut area = self.frame.area.borrow_mut();
                let v = area.stack.pop_int();
                Oop::new_int(v)
            }
            ValueType::FLOAT => {
                let mut area = self.frame.area.borrow_mut();
                let v = area.stack.pop_float();
                Oop::new_float(v)
            }
            ValueType::DOUBLE => {
                let mut area = self.frame.area.borrow_mut();
                let v = area.stack.pop_double();
                Oop::new_double(v)
            }
            ValueType::LONG => {
                let mut area = self.frame.area.borrow_mut();
                let v = area.stack.pop_long();
                Oop::new_long(v)
            }
            ValueType::ARRAY | ValueType::OBJECT => {
                let mut area = self.frame.area.borrow_mut();
                area.stack.pop_ref()
            }
            _ => unreachable!(),
        };

        let mut class = fir.field.class.write().unwrap();
        if is_static {
            class.put_static_field_value(fir.clone(), v);
        } else {
            let receiver = {
                let mut area = self.frame.area.borrow_mut();
                area.stack.pop_ref()
            };
            match receiver {
                Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
                _ => {
                    class.put_field_value(receiver, fir.clone(), v);
                }
            }
        }
    }

    fn invoke_helper(
        &self,
        jt: &mut JavaThread,
        is_static: bool,
        idx: usize,
        force_no_resolve: bool,
    ) {
        let mir = { oop::method::get_method_ref(jt, &self.frame.cp, idx) };

        match mir {
            Ok(mir) => {
                assert_eq!(mir.method.is_static(), is_static);

                match runtime::java_call::JavaCall::new(jt, &self.frame.area, mir) {
                    Ok(mut jc) => {
                        jc.invoke(jt, Some(&self.frame.area), force_no_resolve);
                    }

                    //ignored, let interp main loop handle exception
                    _ => (),
                }
            }
            Err(_) => unreachable!("NotFound method"),
        }
    }

    pub fn check_cast_helper(&self, thread: &mut JavaThread, is_cast: bool) {
        let cp_idx = self.read_i2();
        let target_cls = require_class2(cp_idx as U2, &self.frame.cp).unwrap();

        let mut area = self.frame.area.borrow_mut();
        let obj_rf = area.stack.pop_ref();
        drop(area);

        let obj_rf_clone = obj_rf.clone();
        let op_check_cast = |r: bool, obj_cls: ClassRef, target_cls: ClassRef| {
            if r {
                let mut area = self.frame.area.borrow_mut();
                area.stack.push_ref(obj_rf_clone);
            } else {
                let obj_name = { obj_cls.read().unwrap().name.clone() };
                let target_name = { target_cls.read().unwrap().name.clone() };

                let obj_name = String::from_utf8_lossy(obj_name.as_slice()).replace("/", ".");
                let target_name = String::from_utf8_lossy(target_name.as_slice()).replace("/", ".");

                let msg = format!("{} cannot be cast to {}", obj_name, target_name);
                exception::meet_ex(thread, consts::J_CCE, Some(msg));
            }
        };
        let op_instance_of = |r: bool| {
            let mut area = self.frame.area.borrow_mut();
            if r {
                area.stack.push_const1(false);
            } else {
                area.stack.push_const0(false);
            }
        };

        match obj_rf {
            Oop::Null => {
                let mut area = self.frame.area.borrow_mut();
                if is_cast {
                    area.stack.push_ref(obj_rf);
                } else {
                    area.stack.push_const0(false);
                }
            }
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::Inst(inst) => {
                        let obj_cls = inst.class.clone();
                        let r = cmp::instance_of(obj_cls.clone(), target_cls.clone());

                        if is_cast {
                            op_check_cast(r, obj_cls, target_cls);
                        } else {
                            op_instance_of(r);
                        }
                    }
                    oop::RefKind::Array(ary) => {
                        let obj_cls = ary.class.clone();
                        let r = cmp::instance_of(obj_cls.clone(), target_cls.clone());
                        if is_cast {
                            op_check_cast(r, obj_cls, target_cls);
                        } else {
                            op_instance_of(r);
                        }
                    }
                    oop::RefKind::Mirror(mirror) => {
                        //run here codes:
                        //$JDK_TEST/Appendable/Basic.java
                        //Will eventually call java.security.Security.getSpiClass ("MessageDigest"):
                        //Exception in thread "main" java.lang.ClassCastException: java.security.MessageDigestSpi cannot be cast to java.lang.Class

                        let obj_cls = mirror.target.clone().unwrap();
                        let target_name = { target_cls.read().unwrap().name.clone() };
                        let r = target_name.as_slice() == b"java/lang/Class"
                            || cmp::instance_of(obj_cls.clone(), target_cls.clone());

                        if is_cast {
                            op_check_cast(r, obj_cls, target_cls);
                        } else {
                            op_instance_of(r);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

//handle exception
impl<'a> Interp<'a> {
    fn try_handle_exception(&self, jt: &mut JavaThread, ex: Oop) -> Result<(), Oop> {
        let ex_cls = {
            let ex = util::oop::extract_ref(&ex);
            let v = ex.read().unwrap();
            match &v.v {
                oop::RefKind::Inst(inst) => inst.class.clone(),
                _ => unreachable!(),
            }
        };

        let method_cls_name = { self.frame.mir.method.class.read().unwrap().name.clone() };
        let method_cls_name = String::from_utf8_lossy(method_cls_name.as_slice());
        let method_name = self.frame.mir.method.get_id();
        let method_name = String::from_utf8_lossy(method_name.as_slice());

        let handler = {
            let area = self.frame.area.borrow();
            self.frame
                .mir
                .method
                .find_exception_handler(&self.frame.cp, area.pc as u16, ex_cls)
        };

        match handler {
            Some(pc) => {
                let mut area = self.frame.area.borrow_mut();
                area.stack.clear();
                area.stack.push_ref(ex);
                drop(area);

                let line_num = self.frame.mir.method.get_line_num(pc);

                info!(
                    "Found Exception Handler: line={}, frame_id={}, {}:{}",
                    line_num, self.frame.frame_id, method_cls_name, method_name
                );

                self.goto_abs(pc as i32);
                Ok(())
            }

            None => {
                let area = self.frame.area.borrow();
                let line_num = self.frame.mir.method.get_line_num(area.pc as u16);

                info!(
                    "NotFound Exception Handler: line={}, frame_id={}, {}:{}, ex_here={}",
                    line_num, self.frame.frame_id, method_cls_name, method_name, area.ex_here
                );

                Err(ex)
            }
        }
    }
}

//byte code impl
impl<'a> Interp<'a> {
    pub fn nop(&self) {}

    pub fn aconst_null(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_null();
    }

    pub fn iconst_m1(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const_m1();
    }

    pub fn iconst_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const0(false);
    }

    pub fn lconst_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const0(true);
    }

    pub fn fconst_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const0(false);
    }

    pub fn dconst_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const0(true);
    }

    pub fn iconst_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const1(false);
    }

    pub fn lconst_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const1(true);
    }

    pub fn fconst_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const1(false);
    }

    pub fn dconst_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const1(true);
    }

    pub fn iconst_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const2();
    }

    pub fn fconst_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const2();
    }

    pub fn iconst_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const3();
    }

    pub fn iconst_4(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const4();
    }

    pub fn iconst_5(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.push_const5();
    }

    pub fn sipush(&self) {
        let v = self.read_i2();

        let mut area = self.frame.area.borrow_mut();
        area.stack.push_int(v);
    }

    pub fn bipush(&self) {
        let v = (self.read_byte() as i8) as i32;

        let mut area = self.frame.area.borrow_mut();
        area.stack.push_int(v);
    }

    pub fn ldc(&self, thread: &mut JavaThread) {
        let pos = self.read_u1();

        self.load_constant(pos, thread);
    }

    pub fn ldc_w(&self, thread: &mut JavaThread) {
        let pos = self.read_u2();

        self.load_constant(pos, thread);
    }

    pub fn ldc2_w(&self, thread: &mut JavaThread) {
        self.ldc_w(thread);
    }

    pub fn iload(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_int(pos);
        area.stack.push_int(v);

        area.op_widen = false;
    }

    pub fn lload(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_long(pos);
        area.stack.push_long(v);

        area.op_widen = false;
    }

    pub fn fload(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_float(pos);
        area.stack.push_float(v);

        area.op_widen = false;
    }

    pub fn dload(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_double(pos);
        area.stack.push_double(v);

        area.op_widen = false;
    }

    pub fn aload(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_ref(pos);
        area.stack.push_ref(v);

        area.op_widen = false;
    }

    pub fn iload_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_int(0);
        area.stack.push_int(v);
    }

    pub fn lload_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_long(0);
        area.stack.push_long(v);
    }

    pub fn fload_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_float(0);
        area.stack.push_float(v);
    }

    pub fn dload_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_double(0);
        area.stack.push_double(v);
    }

    pub fn aload_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_ref(0);
        area.stack.push_ref(v);
    }

    pub fn iload_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_int(1);
        area.stack.push_int(v);
    }

    pub fn lload_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_long(1);
        area.stack.push_long(v);
    }

    pub fn fload_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_float(1);
        area.stack.push_float(v);
    }

    pub fn dload_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_double(1);
        area.stack.push_double(v);
    }

    pub fn aload_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_ref(1);
        area.stack.push_ref(v);
    }

    pub fn iload_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_int(2);
        area.stack.push_int(v);
    }

    pub fn lload_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_long(2);
        area.stack.push_long(v);
    }

    pub fn fload_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_float(2);
        area.stack.push_float(v);
    }

    pub fn dload_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_double(2);
        area.stack.push_double(v);
    }

    pub fn aload_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_ref(2);
        area.stack.push_ref(v);
    }

    pub fn iload_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_int(3);
        area.stack.push_int(v);
    }

    pub fn lload_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_long(3);
        area.stack.push_long(v);
    }

    pub fn fload_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_float(3);
        area.stack.push_float(v);
    }

    pub fn dload_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_double(3);
        area.stack.push_double(v);
    }

    pub fn aload_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_ref(3);
        area.stack.push_ref(v);
    }

    pub fn iaload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Int(ary) => {
                            iarray_load!(thread, area, ary, pos);
                        }
                        t => unreachable!("t = {:?}", t),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn saload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Short(ary) => {
                            iarray_load!(thread, area, ary, pos);
                        }
                        t => unreachable!("t = {:?}", t),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn caload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Char(ary) => {
                            iarray_load!(thread, area, ary, pos);
                        }
                        t => unreachable!("t = {:?}", t),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn baload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Byte(ary) => {
                            iarray_load!(thread, area, ary, pos);
                        }
                        oop::TypeArrayDesc::Bool(ary) => {
                            iarray_load!(thread, area, ary, pos);
                        }
                        t => unreachable!("t = {:?}", t),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn laload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Long(ary) => {
                            let len = ary.len();
                            if (pos < 0) || (pos as usize >= len) {
                                let msg = format!("length is {}, but index is {}", len, pos);
                                exception::meet_ex(
                                    thread,
                                    consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                    Some(msg),
                                );
                            } else {
                                area.stack.push_long(ary[pos as usize]);
                            }
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn faload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Float(ary) => {
                            let len = ary.len();
                            if (pos < 0) || (pos as usize >= len) {
                                let msg = format!("length is {}, but index is {}", len, pos);
                                exception::meet_ex(
                                    thread,
                                    consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                    Some(msg),
                                );
                            } else {
                                area.stack.push_float(ary[pos as usize]);
                            }
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn daload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Double(ary) => {
                            let len = ary.len();
                            if (pos < 0) || (pos as usize >= len) {
                                let msg = format!("length is {}, but index is {}", len, pos);
                                exception::meet_ex(
                                    thread,
                                    consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                    Some(msg),
                                );
                            } else {
                                area.stack.push_double(ary[pos as usize]);
                            }
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn aaload(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::Array(ary) => {
                        let len = ary.elements.len();
                        //                info!("aaload pos={}, len={}", pos, len);
                        if (pos < 0) || (pos as usize >= len) {
                            let msg = format!("length is {}, but index is {}", len, pos);
                            exception::meet_ex(
                                thread,
                                consts::J_ARRAY_INDEX_OUT_OF_BOUNDS,
                                Some(msg),
                            );
                        } else {
                            let v = ary.elements[pos as usize].clone();
                            area.stack.push_ref(v);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn istore(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.local.set_int(pos, v);

        area.op_widen = false;
    }

    pub fn lstore(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.local.set_long(pos, v);

        area.op_widen = false;
    }

    pub fn fstore(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        area.local.set_float(pos, v);

        area.op_widen = false;
    }

    pub fn dstore(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        area.local.set_double(pos, v);

        area.op_widen = false;
    }

    pub fn astore(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        area.local.set_ref(pos, v);

        area.op_widen = false;
    }

    pub fn istore_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.local.set_int(0, v);
    }

    pub fn istore_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.local.set_int(1, v);
    }

    pub fn istore_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.local.set_int(2, v);
    }

    pub fn istore_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.local.set_int(3, v);
    }

    pub fn lstore_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.local.set_long(0, v);
    }

    pub fn lstore_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.local.set_long(1, v);
    }

    pub fn lstore_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.local.set_long(2, v);
    }

    pub fn lstore_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.local.set_long(3, v);
    }

    pub fn fstore_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        area.local.set_float(0, v);
    }

    pub fn fstore_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        area.local.set_float(1, v);
    }

    pub fn fstore_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        area.local.set_float(2, v);
    }

    pub fn fstore_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        area.local.set_float(3, v);
    }

    pub fn dstore_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        area.local.set_double(0, v);
    }

    pub fn dstore_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        area.local.set_double(1, v);
    }

    pub fn dstore_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        area.local.set_double(2, v);
    }

    pub fn dstore_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        area.local.set_double(3, v);
    }

    pub fn astore_0(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        area.local.set_ref(0, v);
    }

    pub fn astore_1(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        area.local.set_ref(1, v);
    }

    pub fn astore_2(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        area.local.set_ref(2, v);
    }

    pub fn astore_3(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        area.local.set_ref(3, v);
    }

    pub fn bastore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Byte(ary) => {
                            let v = v as u8;
                            array_store!(thread, ary, pos, v);
                        }
                        oop::TypeArrayDesc::Bool(ary) => {
                            let v = v as u8;
                            array_store!(thread, ary, pos, v);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn castore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Char(ary) => {
                            let v = v as u16;
                            array_store!(thread, ary, pos, v);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn sastore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Short(ary) => {
                            let v = v as i16;
                            array_store!(thread, ary, pos, v);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn iastore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Int(ary) => {
                            array_store!(thread, ary, pos, v);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn lastore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Long(ary) => {
                            array_store!(thread, ary, pos, v);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn fastore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Float(ary) => {
                            array_store!(thread, ary, pos, v);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn dastore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::TypeArray(ary) => match ary {
                        oop::TypeArrayDesc::Double(ary) => {
                            array_store!(thread, ary, pos, v);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn aastore(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        let pos = area.stack.pop_int();
        let ary_rf = area.stack.pop_ref();
        drop(area);

        match ary_rf {
            Oop::Null => exception::meet_ex(thread, consts::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::Array(ary) => {
                        let ary = &mut ary.elements;
                        array_store!(thread, ary, pos, v);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn pop(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.drop_top();
    }

    pub fn pop2(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.drop_top();
        area.stack.drop_top();
    }

    pub fn dup(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.dup();
    }

    pub fn dup_x1(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.dup_x1();
    }

    pub fn dup_x2(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.dup_x2();
    }

    pub fn dup2(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.dup2();
    }

    pub fn dup2_x1(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.dup2_x1();
    }

    pub fn dup2_x2(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.dup2_x2();
    }

    pub fn swap(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.stack.swap();
    }

    pub fn iadd(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let v = v1.wrapping_add(v2);
        area.stack.push_int(v);
    }

    pub fn ladd(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        let v = v1.wrapping_add(v2);
        area.stack.push_long(v);
    }

    pub fn fadd(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        area.stack.push_float(v1 + v2);
    }

    pub fn dadd(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        area.stack.push_double(v1 + v2);
    }

    pub fn isub(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let v = v1.wrapping_sub(v2);
        area.stack.push_int(v);
    }

    pub fn lsub(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        let v = v1.wrapping_sub(v2);
        area.stack.push_long(v);
    }

    pub fn fsub(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        area.stack.push_float(v1 - v2);
    }

    pub fn dsub(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        area.stack.push_double(v1 - v2);
    }

    pub fn imul(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let v = v1.wrapping_mul(v2);
        area.stack.push_int(v);
    }

    pub fn lmul(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        let v = v1.wrapping_mul(v2);
        area.stack.push_long(v);
    }

    pub fn fmul(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        area.stack.push_float(v1 * v2);
    }

    pub fn dmul(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        area.stack.push_double(v1 * v2);
    }

    pub fn idiv(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_int(v1 / v2);
        }
    }

    pub fn ldiv(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_long(v1 / v2);
        }
    }

    pub fn fdiv(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        if v2 == 0.0 {
            drop(area);

            exception::meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_float(v1 / v2);
        }
    }

    pub fn ddiv(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        if v2 == 0.0 {
            drop(area);

            exception::meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_double(v1 / v2);
        }
    }

    pub fn irem(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_int(v1 - (v1 / v2) * v2);
        }
    }

    pub fn lrem(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                thread,
                consts::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_long(v1 - (v1 / v2) * v2);
        }
    }

    pub fn frem(&self) {
        panic!("Use of deprecated instruction frem, please check your Java compiler");
    }

    pub fn drem(&self) {
        panic!("Use of deprecated instruction drem, please check your Java compiler");
    }

    pub fn ineg(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.stack.push_int(-v);
    }

    pub fn lneg(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.stack.push_long(-v);
    }

    pub fn fneg(&self) {
        panic!("Use of deprecated instruction fneg, please check your Java compiler");
    }

    pub fn dneg(&self) {
        panic!("Use of deprecated instruction dneg, please check your Java compiler");
    }

    pub fn ishl(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let s = v2 & 0x1F;
        //        info!("ishl v2={}, v1={}, s={}, v={}", v2, v1, s, (v1 << s));
        area.stack.push_int(v1 << s);
    }

    pub fn lshl(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        area.stack.push_long(v1 << s);
    }

    pub fn ishr(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let s = v2 & 0x1F;
        area.stack.push_int(v1 >> s);
    }

    pub fn lshr(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        area.stack.push_long(v1 >> s);
    }

    pub fn iushr(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int() as u32;
        let s = (v2 & 0x1F) as u32;
        area.stack.push_int((v1 >> s) as i32);
        /*
        if v1 >= 0 {
            area.stack.push_int(v1 >> s);
        } else {
            area.stack.push_int((v1 >> s) + (2 << !s));
        }
        */
    }

    pub fn lushr(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_long() as u64;
        let s = (v2 & 0x3F) as u64;
        area.stack.push_long((v1 >> s) as i64);
        /*
        if v1 >= 0 {
            area.stack.push_long(v1 >> s);
        } else {
            area.stack.push_long((v1 >> s) + (2 << !s));
        }
        */
    }

    pub fn iand(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        area.stack.push_int(v1 & v2);
    }

    pub fn land(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        area.stack.push_long(v1 & v2);
    }

    pub fn ior(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        area.stack.push_int(v1 | v2);
    }

    pub fn lor(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        area.stack.push_long(v1 | v2);
    }

    pub fn ixor(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        area.stack.push_int(v1 ^ v2);
    }

    pub fn lxor(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        area.stack.push_long(v1 ^ v2);
    }

    pub fn iinc(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let factor = if op_widen {
            (self.read_u2() as i16) as i32
        } else {
            (self.read_byte() as i8) as i32
        };

        let mut area = self.frame.area.borrow_mut();
        let v = area.local.get_int(pos);
        let v = v.wrapping_add(factor);
        area.local.set_int(pos, v);

        area.op_widen = false;
    }

    pub fn i2l(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.stack.push_long(v as i64);
    }

    pub fn i2f(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.stack.push_float(v as f32);
    }

    pub fn i2d(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        area.stack.push_double(v as f64);
    }

    pub fn l2i(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.stack.push_int(v as i32);
    }

    pub fn l2f(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.stack.push_float(v as f32);
    }

    pub fn l2d(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        area.stack.push_double(v as f64);
    }

    pub fn f2i(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        if v.is_nan() {
            area.stack.push_int(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                area.stack.push_int(std::i32::MAX);
            } else {
                area.stack.push_int(std::i32::MIN);
            }
        } else {
            area.stack.push_int(v as i32);
        }
    }

    pub fn f2l(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        if v.is_nan() {
            area.stack.push_long(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                area.stack.push_long(std::i64::MAX);
            } else {
                area.stack.push_long(std::i64::MIN);
            }
        } else {
            area.stack.push_long(v as i64);
        }
    }

    pub fn f2d(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        area.stack.push_double(v as f64);
    }

    pub fn d2i(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        if v.is_nan() {
            area.stack.push_int(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                area.stack.push_int(std::i32::MAX);
            } else {
                area.stack.push_int(std::i32::MIN);
            }
        } else {
            area.stack.push_int(v as i32);
        }
    }

    pub fn d2l(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        if v.is_nan() {
            area.stack.push_long(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                area.stack.push_long(std::i64::MAX);
            } else {
                area.stack.push_long(std::i64::MIN);
            }
        } else {
            area.stack.push_long(v as i64);
        }
    }

    pub fn d2f(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        area.stack.push_float(v as f32);
    }

    pub fn i2b(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let v = v as i8;
        area.stack.push_int(v as i32);
    }

    pub fn i2c(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let v = v as u16;
        area.stack.push_int(v as i32);
    }

    pub fn i2s(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let v = v as i16;
        area.stack.push_int(v as i32);
    }

    pub fn lcmp(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v1 = area.stack.pop_long();
        let v2 = area.stack.pop_long();
        if v1 > v2 {
            area.stack.push_int(-1);
        } else if v1 < v2 {
            area.stack.push_int(1);
        } else {
            area.stack.push_int(0);
        }
    }

    pub fn fcmpl(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v1 = area.stack.pop_float();
        let v2 = area.stack.pop_float();
        if v1.is_nan() || v2.is_nan() {
            area.stack.push_int(-1);
        } else if v1 > v2 {
            area.stack.push_int(-1);
        } else if v1 < v2 {
            area.stack.push_int(1);
        } else {
            area.stack.push_int(0);
        }
    }

    pub fn fcmpg(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v1 = area.stack.pop_float();
        let v2 = area.stack.pop_float();
        if v1.is_nan() || v2.is_nan() {
            area.stack.push_int(1);
        } else if v1 > v2 {
            area.stack.push_int(-1);
        } else if v1 < v2 {
            area.stack.push_int(1);
        } else {
            area.stack.push_int(0);
        }
    }

    pub fn dcmpl(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v1 = area.stack.pop_double();
        let v2 = area.stack.pop_double();
        if v1.is_nan() || v2.is_nan() {
            area.stack.push_int(-1);
        } else if v1 > v2 {
            area.stack.push_int(-1);
        } else if v1 < v2 {
            area.stack.push_int(1);
        } else {
            area.stack.push_int(0);
        }
    }

    pub fn dcmpg(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v1 = area.stack.pop_double();
        let v2 = area.stack.pop_double();
        if v1.is_nan() || v2.is_nan() {
            area.stack.push_int(1);
        } else if v1 > v2 {
            area.stack.push_int(-1);
        } else if v1 < v2 {
            area.stack.push_int(1);
        } else {
            area.stack.push_int(0);
        }
    }

    pub fn ifeq(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();

        if v == 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn ifne(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();

        if v != 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn iflt(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();

        if v < 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn ifge(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();

        if v >= 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn ifgt(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();

        if v > 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn ifle(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();

        if v <= 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_icmpeq(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 == v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_icmpne(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 != v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_icmplt(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 < v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_icmpge(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 >= v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_icmpgt(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 > v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_icmple(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 <= v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_acmpeq(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_ref();
        let v1 = area.stack.pop_ref();

        if util::oop::if_acmpeq(&v1, &v2) {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn if_acmpne(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v2 = area.stack.pop_ref();
        let v1 = area.stack.pop_ref();

        if !util::oop::if_acmpeq(&v1, &v2) {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            area.pc += 2;
        }
    }

    pub fn goto(&self) {
        self.goto_by_offset_hardcoded(2);
    }

    pub fn jsr(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.pc += 2;
        panic!("Use of deprecated instruction jsr, please check your Java compiler");
    }

    pub fn ret(&self) {
        let op_widen = {
            let area = self.frame.area.borrow();
            area.op_widen
        };

        let pc = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.borrow_mut();
        area.pc = pc as i32;

        area.op_widen = true;
    }

    pub fn table_switch(&self) {
        let mut bc = {
            let area = self.frame.area.borrow();
            area.pc - 1
        };

        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += (4 - bc % 4);
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;
        let default_byte = [
            self.frame.code[ptr],
            self.frame.code[ptr + 1],
            self.frame.code[ptr + 2],
            self.frame.code[ptr + 3],
        ];
        let default_byte = i32::from_be_bytes(default_byte);
        let low_byte = [
            self.frame.code[ptr + 4],
            self.frame.code[ptr + 5],
            self.frame.code[ptr + 6],
            self.frame.code[ptr + 7],
        ];
        let low_byte = i32::from_be_bytes(low_byte);
        let high_byte = [
            self.frame.code[ptr + 8],
            self.frame.code[ptr + 9],
            self.frame.code[ptr + 10],
            self.frame.code[ptr + 11],
        ];
        let high_byte = i32::from_be_bytes(high_byte);
        let num = high_byte - low_byte + 1;
        ptr += 12;

        // switch-case jump table
        let mut jump_table = Vec::with_capacity(num as usize);
        for pos in 0..num {
            let pos = [
                self.frame.code[ptr],
                self.frame.code[ptr + 1],
                self.frame.code[ptr + 2],
                self.frame.code[ptr + 3],
            ];
            let pos = i32::from_be_bytes(pos);
            let jump_pos = pos + origin_bc;
            ptr += 4;
            jump_table.push(jump_pos);
        }
        // default
        jump_table.push(default_byte + origin_bc);

        let top_value = {
            let mut area = self.frame.area.borrow_mut();
            area.stack.pop_int()
        };
        if (top_value > (jump_table.len() as i32 - 1 + low_byte)) || top_value < low_byte {
            self.goto_abs_with_occupied(*jump_table.last().unwrap() as i32, 1);
        } else {
            self.goto_abs_with_occupied(
                jump_table[(top_value - low_byte as i32) as usize] as i32,
                1,
            );
        }
    }

    pub fn lookup_switch(&self) {
        let mut bc = {
            let area = self.frame.area.borrow();
            area.pc - 1
        };

        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += (4 - bc % 4);
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;

        let default_byte = [
            self.frame.code[ptr],
            self.frame.code[ptr + 1],
            self.frame.code[ptr + 2],
            self.frame.code[ptr + 3],
        ];
        let default_byte = u32::from_be_bytes(default_byte);
        let count = [
            self.frame.code[ptr + 4],
            self.frame.code[ptr + 5],
            self.frame.code[ptr + 6],
            self.frame.code[ptr + 7],
        ];
        let count = u32::from_be_bytes(count);
        ptr += 8;

        let mut jump_table: HashMap<u32, u32> = HashMap::new();
        for i in 0..count {
            let value = [
                self.frame.code[ptr],
                self.frame.code[ptr + 1],
                self.frame.code[ptr + 2],
                self.frame.code[ptr + 3],
            ];
            let value = u32::from_be_bytes(value);
            let position = [
                self.frame.code[ptr + 4],
                self.frame.code[ptr + 5],
                self.frame.code[ptr + 6],
                self.frame.code[ptr + 7],
            ];
            let position = u32::from_be_bytes(position) + origin_bc as u32;
            ptr += 8;
            jump_table.insert(value, position);
        }

        let top_value = {
            let mut area = self.frame.area.borrow_mut();
            area.stack.pop_int()
        };
        match jump_table.get(&(top_value as u32)) {
            Some(position) => self.goto_abs_with_occupied(*position as i32, 1),
            None => self.goto_abs_with_occupied(default_byte as i32 + origin_bc, 1),
        }
    }

    pub fn ireturn(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_int();
        let v = Oop::new_int(v);
        drop(area);

        self.set_return(Some(v));
    }

    pub fn lreturn(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_long();
        let v = Oop::new_long(v);
        drop(area);

        self.set_return(Some(v));
    }

    pub fn freturn(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_float();
        let v = Oop::new_float(v);
        drop(area);

        self.set_return(Some(v));
    }

    pub fn dreturn(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_double();
        let v = Oop::new_double(v);
        drop(area);

        self.set_return(Some(v));
    }

    pub fn areturn(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        drop(area);

        self.set_return(Some(v));
    }

    pub fn return_void(&self) {
        self.set_return(None);
    }

    pub fn get_static(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.get_field_helper(thread, oop_consts::get_null(), cp_idx, true);
    }

    pub fn put_static(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.put_field_helper(thread, cp_idx, true);
    }

    pub fn get_field(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();

        let mut area = self.frame.area.borrow_mut();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => {
                exception::meet_ex(thread, consts::J_NPE, None);
            }
            _ => {
                self.get_field_helper(thread, rf, cp_idx, false);
            }
        }
    }

    pub fn put_field(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.put_field_helper(thread, cp_idx, false);
    }

    pub fn invoke_virtual(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.invoke_helper(thread, false, cp_idx as usize, false);
    }

    pub fn invoke_special(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.invoke_helper(thread, false, cp_idx as usize, true);
    }

    pub fn invoke_static(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        self.invoke_helper(thread, true, cp_idx as usize, true);
    }

    pub fn invoke_interface(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();
        let _count = self.read_u1();
        let zero = self.read_u1();

        if zero != 0 {
            warn!("interpreter: invalid invokeinterface: the value of the fourth operand byte must always be zero.");
        }

        self.invoke_helper(thread, false, cp_idx as usize, false);
    }

    pub fn invoke_dynamic(&self) {
        //todo: impl
        unimplemented!()
    }

    pub fn new_(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();

        let class = {
            match runtime::require_class2(cp_idx as u16, &self.frame.cp) {
                Some(class) => {
                    {
                        let mut class = class.write().unwrap();
                        class.init_class(thread);
                    }

                    oop::class::init_class_fully(thread, class.clone());

                    class
                }
                None => unreachable!("Cannot get class info from constant pool"),
            }
        };

        let mut area = self.frame.area.borrow_mut();
        let v = oop::Oop::new_inst(class);
        area.stack.push_ref(v);
    }

    pub fn new_array(&self, thread: &mut JavaThread) {
        let t = self.read_byte();

        let mut area = self.frame.area.borrow_mut();
        let len = area.stack.pop_int();

        if len < 0 {
            drop(area);
            exception::meet_ex(thread, consts::J_NASE, Some("length < 0".to_string()));
        } else {
            let len = len as usize;
            let ary = match t {
                //boolean
                4 => Oop::new_bool_ary(len),
                //char
                5 => Oop::new_char_ary(len),
                //float
                6 => Oop::new_float_ary(len),
                //double
                7 => Oop::new_double_ary(len),
                //byte
                8 => Oop::new_byte_ary(len),
                //short
                9 => Oop::new_short_ary(len),
                //int
                10 => Oop::new_int_ary(len),
                //long
                11 => Oop::new_long_ary(len),
                _ => unreachable!(),
            };

            area.stack.push_ref(ary);
        }
    }

    pub fn anew_array(&self, thread: &mut JavaThread) {
        let cp_idx = self.read_i2();

        let mut area = self.frame.area.borrow_mut();
        let length = area.stack.pop_int();
        drop(area);

        //        info!("anew_array length={}", length);
        if length < 0 {
            exception::meet_ex(thread, consts::J_NASE, Some("length < 0".to_string()));
        } else {
            let class = match runtime::require_class2(cp_idx as u16, &self.frame.cp) {
                Some(class) => class,
                None => panic!("Cannot get class info from constant pool"),
            };

            {
                let mut class = class.write().unwrap();
                class.init_class(thread);
            }

            oop::class::init_class_fully(thread, class.clone());

            let (name, cl) = {
                let class = class.read().unwrap();
                let t = class.get_class_kind_type();
                let name = match t {
                    oop::class::ClassKindType::Instance | oop::class::ClassKindType::ObjectAry => {
                        let mut v = Vec::with_capacity(class.name.len() + 2);
                        v.push(b'[');
                        v.push(b'L');
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

                let name = Arc::new(name);
                (name, class.class_loader.clone())
            };

            trace!("anew_array name={}", unsafe {
                std::str::from_utf8_unchecked(name.as_slice())
            });
            match runtime::require_class(cl, name) {
                Some(ary_cls_obj) => {
                    {
                        {
                            let mut class = ary_cls_obj.write().unwrap();
                            class.init_class(thread);
                        }

                        oop::class::init_class_fully(thread, ary_cls_obj.clone());
                    }

                    let mut area = self.frame.area.borrow_mut();
                    let ary = Oop::new_ref_ary(ary_cls_obj, length as usize);
                    area.stack.push_ref(ary);
                }
                None => unreachable!(),
            }
        }
    }

    pub fn array_length(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();

        match v {
            Oop::Null => {
                drop(area);
                exception::meet_ex(thread, consts::J_NPE, None)
            }
            Oop::Ref(rf) => {
                let v = rf.read().unwrap();
                match &v.v {
                    oop::RefKind::Array(ary) => {
                        let len = ary.elements.len();
                        area.stack.push_int(len as i32);
                    }
                    oop::RefKind::TypeArray(ary) => {
                        let len = ary.len();
                        area.stack.push_int(len as i32);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn athrow(&self, jt: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let ex = area.stack.pop_ref();
        drop(area);

        jt.set_ex(ex);
    }

    pub fn check_cast(&self, thread: &mut JavaThread) {
        self.check_cast_helper(thread, true);
    }

    pub fn instance_of(&self, thread: &mut JavaThread) {
        self.check_cast_helper(thread, false);
    }

    pub fn monitor_enter(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();
        drop(area);

        match v {
            Oop::Null => {
                exception::meet_ex(thread, consts::J_NPE, None);
            }
            Oop::Ref(v) => {
                let mut v = v.write().unwrap();
                v.monitor_enter();
            }
            _ => unreachable!(),
        }
    }

    pub fn monitor_exit(&self, thread: &mut JavaThread) {
        let mut area = self.frame.area.borrow_mut();
        let mut v = area.stack.pop_ref();
        drop(area);

        match v {
            Oop::Null => {
                exception::meet_ex(thread, consts::J_NPE, None);
            }
            Oop::Ref(v) => {
                let mut v = v.write().unwrap();
                v.monitor_exit();
            }
            _ => unreachable!(),
        }
    }

    pub fn wide(&self) {
        let mut area = self.frame.area.borrow_mut();
        info!("opcode wide");
        area.op_widen = true;
    }

    pub fn multi_anew_array(&self) {
        let cp_idx = self.read_u2();
        let dimension = self.read_u1();

        let mut lens = Vec::new();
        let mut area = self.frame.area.borrow_mut();
        for _ in 0..dimension {
            let sub = area.stack.pop_int();
            //todo: check java/lang/NegativeArraySizeException
            lens.push(sub);
        }
        drop(area);

        let cls = require_class2(cp_idx as u16, &self.frame.cp).unwrap();
        let ary = new_multi_object_array_helper(cls, &lens, 0);

        let mut area = self.frame.area.borrow_mut();
        area.stack.push_ref(ary);
    }

    pub fn if_null(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();

        match v {
            Oop::Null => {
                drop(area);
                self.goto_by_offset_hardcoded(2)
            }
            _ => area.pc += 2,
        }
    }

    pub fn if_non_null(&self) {
        let mut area = self.frame.area.borrow_mut();
        let v = area.stack.pop_ref();

        match v {
            Oop::Null => area.pc += 2,
            _ => {
                drop(area);
                self.goto_by_offset_hardcoded(2)
            }
        }
    }

    pub fn goto_w(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.pc += 4;
        panic!("Use of deprecated instruction goto_w, please check your Java compiler")
    }

    pub fn jsr_w(&self) {
        let mut area = self.frame.area.borrow_mut();
        area.pc += 4;
        panic!("Use of deprecated instruction jsr_w, please check your Java compiler")
    }

    pub fn other_wise(&self) {
        let mut area = self.frame.area.borrow_mut();
        let pc = area.pc - 1;
        panic!(
            "Use of undefined bytecode: {} at {}",
            self.frame.code[pc as usize], pc
        );
    }
}

fn new_multi_object_array_helper(cls: ClassRef, lens: &Vec<i32>, idx: usize) -> Oop {
    let length = lens[idx] as usize;

    let down_type = {
        let cls = cls.read().unwrap();
        match &cls.kind {
            oop::ClassKind::Instance(_) => unreachable!(),
            ClassKind::ObjectArray(obj_ary) => obj_ary.down_type.clone().unwrap(),
            ClassKind::TypeArray(typ_ary) => typ_ary.down_type.clone().unwrap(),
        }
    };

    if idx < lens.len() - 1 {
        let mut elms = Vec::with_capacity(length);
        for i in 0..length {
            let e = new_multi_object_array_helper(down_type.clone(), lens, idx + 1);
            elms.push(e);
        }

        Oop::new_ref_ary2(cls, elms)
    } else {
        Oop::new_ref_ary(cls, length)
    }
}
