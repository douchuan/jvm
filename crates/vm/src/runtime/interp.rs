use crate::oop::{
    self, consts as oop_consts, field, Class, ClassKind, Oop, OopRef, TypeArrayDesc, ValueType,
};
use crate::runtime::local::Local;
use crate::runtime::stack::Stack;
use crate::runtime::{
    self, cmp, exception, require_class, require_class2, require_class3, DataArea, Frame, JavaCall,
};
use crate::types::*;
use crate::util;
use classfile::{
    constant_pool::get_utf8 as get_cp_utf8,
    consts as cls_const,
    types::{U1, U2},
    ClassFile, ConstantPoolType, OpCode,
};
use nix::sys::socket::SockType::Datagram;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLockReadGuard};

macro_rules! array_store {
    ($ary:ident, $pos:ident, $v:ident) => {
        let len = $ary.len();
        if ($pos < 0) || ($pos as usize >= len) {
            let msg = format!("length is {}, but index is {}", len, $pos);
            exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
        } else {
            $ary[$pos as usize] = $v;
        }
    };
}

macro_rules! iarray_load {
    ($area:ident, $ary:ident, $pos:ident) => {
        let len = $ary.len();
        if ($pos < 0) || ($pos as usize >= len) {
            drop($area);
            let msg = format!("length is {}, but index is {}", len, $pos);
            exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
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

        trace!(
            "interp: {:?} ({}/{}) {:?}",
            op,
            code,
            frame_id,
            self.frame.mir.method
        );
    }
}

impl<'a> Interp<'a> {
    pub fn run(&self) {
        let jt = runtime::thread::current_java_thread();

        loop {
            let code = self.read_opcode();
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

            let is_meet_ex = self.frame.ex_here.load(Ordering::Relaxed);
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

//helper methods
impl<'a> Interp<'a> {
    #[inline]
    fn read_i2(&self) -> i32 {
        let h = self.read_byte() as i16;
        let l = self.read_byte() as i16;
        (h << 8 | l) as i32
    }

    #[inline]
    fn read_u1(&self) -> usize {
        let pc = self.frame.pc.fetch_add(1, Ordering::Relaxed);
        let v = self.frame.code[pc as usize];
        v as usize
    }

    #[inline]
    fn read_byte(&self) -> u8 {
        let pc = self.frame.pc.fetch_add(1, Ordering::Relaxed);
        self.frame.code[pc as usize]
    }

    #[inline]
    fn read_opcode(&self) -> U1 {
        let pc = self.frame.pc.fetch_add(1, Ordering::Relaxed);
        self.frame.code[pc as usize]
    }

    #[inline]
    fn read_u2(&self) -> usize {
        self.read_u1() << 8 | self.read_u1()
    }

    fn load_constant(&self, pos: usize) {
        match &self.frame.cp[pos] {
            ConstantPoolType::Integer { v } => {
                let mut area = self.frame.area.write().unwrap();
                area.stack.push_int2(*v)
            }
            ConstantPoolType::Float { v } => {
                let mut area = self.frame.area.write().unwrap();
                area.stack.push_float2(*v)
            }
            ConstantPoolType::Long { v } => {
                let mut area = self.frame.area.write().unwrap();
                area.stack.push_long2(*v)
            }
            ConstantPoolType::Double { v } => {
                let mut area = self.frame.area.write().unwrap();
                area.stack.push_double2(*v)
            }
            ConstantPoolType::String { string_index } => {
                let s = get_cp_utf8(&self.frame.cp, *string_index as usize).unwrap();
                let s = util::oop::new_java_lang_string3(s.as_slice());

                let mut area = self.frame.area.write().unwrap();
                area.stack.push_ref(s);
            }
            ConstantPoolType::Class { name_index } => {
                let name = get_cp_utf8(&self.frame.cp, *name_index as usize).unwrap();
                let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
                let cl = { self.frame.class.get_class().class_loader };
                trace!("load_constant name={}, cl={:?}", name, cl);
                let class = runtime::require_class3(cl, name.as_bytes()).unwrap();
                oop::class::init_class(&class);
                oop::class::init_class_fully(&class);

                let mirror = { class.get_class().get_mirror() };
                let mut area = self.frame.area.write().unwrap();
                area.stack.push_ref(mirror);
            }
            _ => unreachable!(),
        }
    }

    fn goto_abs(&self, pc: i32) {
        self.frame.pc.store(pc, Ordering::Relaxed);
    }

    fn goto_by_offset(&self, branch: i32) {
        let _ = self.frame.pc.fetch_add(branch, Ordering::Relaxed);
    }

    fn goto_by_offset_with_occupied(&self, branch: i32, occupied: i32) {
        self.goto_by_offset(branch);
        self.goto_by_offset(-(occupied - 1));
    }

    fn goto_by_offset_hardcoded(&self, occupied: i32) {
        let pc = self.frame.pc.load(Ordering::Relaxed);
        let high = self.frame.code[pc as usize] as i16;
        let low = self.frame.code[(pc + 1) as usize] as i16;
        let branch = (high << 8) | low;

        self.goto_by_offset_with_occupied(branch as i32, occupied);
    }

    fn goto_abs_with_occupied(&self, pc: i32, occupied: i32) {
        self.goto_abs(pc);
        self.goto_by_offset(-(occupied - 1));
    }

    fn set_return(&self, v: Option<Oop>) {
        let mut area = self.frame.area.write().unwrap();
        area.return_v = v;
    }

    fn get_field_helper(&self, receiver: Oop, idx: i32, is_static: bool) {
        let fir = { field::get_field_ref(&self.frame.cp, idx as usize, is_static) };

        assert_eq!(fir.field.is_static(), is_static);

        trace!("get_field_helper={:?}, is_static={}", fir.field, is_static);

        let value_type = fir.field.value_type;
        let class = fir.field.class.get_class();
        let v = if is_static {
            class.get_static_field_value(fir.clone())
        } else {
            let rf = receiver.extract_ref();
            Class::get_field_value2(rf, fir.offset)
        };

        match value_type {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => match v {
                Oop::Int(v) => {
                    let mut area = self.frame.area.write().unwrap();
                    area.stack.push_int(v)
                }
                t => unreachable!("t = {:?}", t),
            },
            ValueType::FLOAT => match v {
                Oop::Float(v) => {
                    let mut area = self.frame.area.write().unwrap();
                    area.stack.push_float(v)
                }
                _ => unreachable!(),
            },
            ValueType::DOUBLE => match v {
                Oop::Double(v) => {
                    let mut area = self.frame.area.write().unwrap();
                    area.stack.push_double(v)
                }
                _ => unreachable!(),
            },
            ValueType::LONG => match v {
                Oop::Long(v) => {
                    let mut area = self.frame.area.write().unwrap();
                    area.stack.push_long(v)
                }
                _ => unreachable!(),
            },
            ValueType::OBJECT | ValueType::ARRAY => {
                let mut area = self.frame.area.write().unwrap();
                area.stack.push_ref(v)
            }
            _ => unreachable!(),
        }
    }

    fn put_field_helper(&self, idx: i32, is_static: bool) {
        let fir = { field::get_field_ref(&self.frame.cp, idx as usize, is_static) };

        assert_eq!(fir.field.is_static(), is_static);

        trace!("put_field_helper={:?}, is_static={}", fir.field, is_static);

        let value_type = fir.field.value_type;
        //        info!("value_type = {:?}", value_type);
        let v = match value_type {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => {
                let mut area = self.frame.area.write().unwrap();
                let v = area.stack.pop_int();
                Oop::new_int(v)
            }
            ValueType::FLOAT => {
                let mut area = self.frame.area.write().unwrap();
                let v = area.stack.pop_float();
                Oop::new_float(v)
            }
            ValueType::DOUBLE => {
                let mut area = self.frame.area.write().unwrap();
                let v = area.stack.pop_double();
                Oop::new_double(v)
            }
            ValueType::LONG => {
                let mut area = self.frame.area.write().unwrap();
                let v = area.stack.pop_long();
                Oop::new_long(v)
            }
            ValueType::ARRAY | ValueType::OBJECT => {
                let mut area = self.frame.area.write().unwrap();
                area.stack.pop_ref()
            }
            _ => unreachable!(),
        };

        let mut class = fir.field.class.get_mut_class();
        if is_static {
            class.put_static_field_value(fir.clone(), v);
        } else {
            let receiver = {
                let mut area = self.frame.area.write().unwrap();
                area.stack.pop_ref()
            };
            match receiver {
                Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
                _ => Class::put_field_value2(receiver.extract_ref(), fir.offset, v),
            }
        }
    }

    fn invoke_helper(&self, is_static: bool, idx: usize, force_no_resolve: bool) {
        let mir = { oop::method::get_method_ref(&self.frame.cp, idx) };

        match mir {
            Ok(mir) => {
                assert_eq!(mir.method.is_static(), is_static);

                if let Ok(mut jc) = runtime::invoke::JavaCall::new(self.frame.area.clone(), mir) {
                    jc.invoke(Some(self.frame.area.clone()), force_no_resolve);
                }
            }
            Err(_) => unreachable!("NotFound method"),
        }
    }

    pub fn check_cast_helper(&self, is_cast: bool) {
        let cp_idx = self.read_i2();
        let target_cls = require_class2(cp_idx as U2, &self.frame.cp).unwrap();

        let mut area = self.frame.area.write().unwrap();
        let obj_rf = area.stack.pop_ref();
        drop(area);

        let obj_rf_clone = obj_rf.clone();
        let op_check_cast = |r: bool, obj_cls: ClassRef, target_cls: ClassRef| {
            if r {
                let mut area = self.frame.area.write().unwrap();
                area.stack.push_ref(obj_rf_clone);
            } else {
                let obj_name = { obj_cls.get_class().name.clone() };
                let target_name = { target_cls.get_class().name.clone() };

                let obj_name = String::from_utf8_lossy(obj_name.as_slice()).replace("/", ".");
                let target_name = String::from_utf8_lossy(target_name.as_slice()).replace("/", ".");

                let msg = format!("{} cannot be cast to {}", obj_name, target_name);
                exception::meet_ex(cls_const::J_CCE, Some(msg));
            }
        };
        let op_instance_of = |r: bool| {
            let mut area = self.frame.area.write().unwrap();
            if r {
                area.stack.push_const1(false);
            } else {
                area.stack.push_const0(false);
            }
        };

        match obj_rf {
            Oop::Null => {
                let mut area = self.frame.area.write().unwrap();
                if is_cast {
                    area.stack.push_ref(obj_rf);
                } else {
                    area.stack.push_const0(false);
                }
            }
            Oop::Ref(rf) => {
                let rf = rf.get_raw_ptr();
                unsafe {
                    match &(*rf).v {
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
                            let target_name = target_cls.get_class().name.as_slice();
                            let r = target_name == b"java/lang/Class"
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
            }
            _ => unreachable!(),
        }
    }
}

//handle exception
impl<'a> Interp<'a> {
    fn try_handle_exception(&self, ex: Oop) -> Result<(), Oop> {
        let ex_cls = {
            let rf = ex.extract_ref();
            let inst = rf.extract_inst();
            inst.class.clone()
        };

        let handler = {
            let pc = self.frame.pc.load(Ordering::Relaxed);
            self.frame
                .mir
                .method
                .find_exception_handler(&self.frame.cp, pc as u16, ex_cls)
        };

        match handler {
            Some(pc) => {
                let mut area = self.frame.area.write().unwrap();
                area.stack.clear();
                area.stack.push_ref(ex);
                drop(area);

                let line_num = self.frame.mir.method.get_line_num(pc);

                info!(
                    "Found Exception Handler: line={}, frame_id={}, {:?}",
                    line_num, self.frame.frame_id, self.frame.mir.method
                );

                self.goto_abs(pc as i32);
                Ok(())
            }

            None => {
                let pc = self.frame.pc.load(Ordering::Relaxed);
                let line_num = self.frame.mir.method.get_line_num(pc as u16);

                info!(
                    "NotFound Exception Handler: line={}, frame_id={}, {:?}",
                    line_num, self.frame.frame_id, self.frame.mir.method,
                );

                Err(ex)
            }
        }
    }
}

//byte code impl
impl<'a> Interp<'a> {
    #[inline]
    pub fn nop(&self) {}

    #[inline]
    pub fn aconst_null(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_null();
    }

    #[inline]
    pub fn iconst_m1(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const_m1();
    }

    #[inline]
    pub fn iconst_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const0(false);
    }

    #[inline]
    pub fn lconst_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const0(true);
    }

    #[inline]
    pub fn fconst_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const0(false);
    }

    #[inline]
    pub fn dconst_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const0(true);
    }

    #[inline]
    pub fn iconst_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const1(false);
    }

    #[inline]
    pub fn lconst_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const1(true);
    }

    #[inline]
    pub fn fconst_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const1(false);
    }

    #[inline]
    pub fn dconst_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const1(true);
    }

    #[inline]
    pub fn iconst_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const2();
    }

    #[inline]
    pub fn fconst_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const2();
    }

    #[inline]
    pub fn iconst_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const3();
    }

    #[inline]
    pub fn iconst_4(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const4();
    }

    #[inline]
    pub fn iconst_5(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.push_const5();
    }

    #[inline]
    pub fn sipush(&self) {
        let v = self.read_i2();

        let mut area = self.frame.area.write().unwrap();
        area.stack.push_int(v);
    }

    #[inline]
    pub fn bipush(&self) {
        let v = (self.read_byte() as i8) as i32;

        let mut area = self.frame.area.write().unwrap();
        area.stack.push_int(v);
    }

    #[inline]
    pub fn ldc(&self) {
        let pos = self.read_u1();
        self.load_constant(pos);
    }

    #[inline]
    pub fn ldc_w(&self) {
        let pos = self.read_u2();
        self.load_constant(pos);
    }

    #[inline]
    pub fn ldc2_w(&self) {
        self.ldc_w();
    }

    #[inline]
    pub fn iload(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_int(pos);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn lload(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_long(pos);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fload(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_float(pos);
        area.stack.push_float(v);
    }

    #[inline]
    pub fn dload(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_double(pos);
        area.stack.push_double(v);
    }

    #[inline]
    pub fn aload(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_ref(pos);
        area.stack.push_ref(v);
    }

    #[inline]
    pub fn iload_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_int(0);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn lload_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_long(0);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fload_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_float(0);
        area.stack.push_float(v);
    }

    #[inline]
    pub fn dload_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_double(0);
        area.stack.push_double(v);
    }

    #[inline]
    pub fn aload_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_ref(0);
        area.stack.push_ref(v);
    }

    #[inline]
    pub fn iload_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_int(1);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn lload_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_long(1);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fload_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_float(1);
        area.stack.push_float(v);
    }

    #[inline]
    pub fn dload_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_double(1);
        area.stack.push_double(v);
    }

    #[inline]
    pub fn aload_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_ref(1);
        area.stack.push_ref(v);
    }

    #[inline]
    pub fn iload_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_int(2);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn lload_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_long(2);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fload_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_float(2);
        area.stack.push_float(v);
    }

    #[inline]
    pub fn dload_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_double(2);
        area.stack.push_double(v);
    }

    #[inline]
    pub fn aload_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_ref(2);
        area.stack.push_ref(v);
    }

    #[inline]
    pub fn iload_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_int(3);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn lload_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_long(3);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fload_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_float(3);
        area.stack.push_float(v);
    }

    #[inline]
    pub fn dload_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_double(3);
        area.stack.push_double(v);
    }

    #[inline]
    pub fn aload_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_ref(3);
        area.stack.push_ref(v);
    }

    #[inline]
    pub fn iaload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_ints();
                iarray_load!(area, ary, pos);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn saload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_shorts();
                iarray_load!(area, ary, pos);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn caload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_chars();
                iarray_load!(area, ary, pos);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn baload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = (*rf).get_raw_ptr();
                unsafe {
                    let ary = (*rf).v.extract_type_array();
                    let len = ary.len();

                    if (pos < 0) || (pos as usize >= len) {
                        let msg = format!("length is {}, but index is {}", len, pos);
                        exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                    } else {
                        match ary {
                            TypeArrayDesc::Byte(ary) => {
                                let v = ary[pos as usize];
                                area.stack.push_int(v as i32);
                            }
                            TypeArrayDesc::Bool(ary) => {
                                let v = ary[pos as usize];
                                area.stack.push_int(v as i32);
                            }
                            t => unreachable!("t = {:?}", t),
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn laload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_longs();
                let len = ary.len();
                if (pos < 0) || (pos as usize >= len) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary[pos as usize];
                    area.stack.push_long(v);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn faload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_floats();
                let len = ary.len();
                if (pos < 0) || (pos as usize >= len) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary[pos as usize];
                    area.stack.push_float(v);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn daload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_doubles();
                let len = ary.len();
                if (pos < 0) || (pos as usize >= len) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary[pos as usize];
                    area.stack.push_double(v);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn aaload(&self) {
        let mut area = self.frame.area.write().unwrap();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_array();
                let ary = &ary.elements;
                let len = ary.len();

                if (pos < 0) || (pos as usize >= len) {
                    let msg = format!("length is {}, but index is {}", len, pos);
                    exception::meet_ex(cls_const::J_ARRAY_INDEX_OUT_OF_BOUNDS, Some(msg));
                } else {
                    let v = ary[pos as usize].clone();
                    area.stack.push_ref(v);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn istore(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.local.set_int(pos, v);
    }

    #[inline]
    pub fn lstore(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.local.set_long(pos, v);
    }

    #[inline]
    pub fn fstore(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        area.local.set_float(pos, v);
    }

    #[inline]
    pub fn dstore(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        area.local.set_double(pos, v);
    }

    #[inline]
    pub fn astore(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        area.local.set_ref(pos, v);
    }

    #[inline]
    pub fn istore_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.local.set_int(0, v);
    }

    #[inline]
    pub fn istore_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.local.set_int(1, v);
    }

    #[inline]
    pub fn istore_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.local.set_int(2, v);
    }

    #[inline]
    pub fn istore_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.local.set_int(3, v);
    }

    #[inline]
    pub fn lstore_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.local.set_long(0, v);
    }

    #[inline]
    pub fn lstore_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.local.set_long(1, v);
    }

    #[inline]
    pub fn lstore_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.local.set_long(2, v);
    }

    #[inline]
    pub fn lstore_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.local.set_long(3, v);
    }

    #[inline]
    pub fn fstore_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        area.local.set_float(0, v);
    }

    #[inline]
    pub fn fstore_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        area.local.set_float(1, v);
    }

    #[inline]
    pub fn fstore_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        area.local.set_float(2, v);
    }

    #[inline]
    pub fn fstore_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        area.local.set_float(3, v);
    }

    #[inline]
    pub fn dstore_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        area.local.set_double(0, v);
    }

    #[inline]
    pub fn dstore_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        area.local.set_double(1, v);
    }

    #[inline]
    pub fn dstore_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        area.local.set_double(2, v);
    }

    #[inline]
    pub fn dstore_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        area.local.set_double(3, v);
    }

    #[inline]
    pub fn astore_0(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        area.local.set_ref(0, v);
    }

    #[inline]
    pub fn astore_1(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        area.local.set_ref(1, v);
    }

    #[inline]
    pub fn astore_2(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        area.local.set_ref(2, v);
    }

    #[inline]
    pub fn astore_3(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        area.local.set_ref(3, v);
    }

    #[inline]
    pub fn bastore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let mut rf = (*rf).get_mut_raw_ptr();
                unsafe {
                    let ary = (*rf).v.extract_mut_type_array();
                    match ary {
                        oop::TypeArrayDesc::Byte(ary) => {
                            let v = v as u8;
                            array_store!(ary, pos, v);
                        }
                        oop::TypeArrayDesc::Bool(ary) => {
                            let v = v as u8;
                            array_store!(ary, pos, v);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn castore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_mut_type_array();
                let ary = ary.extract_mut_chars();
                let v = v as u16;
                array_store!(ary, pos, v);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn sastore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_mut_type_array();
                let ary = ary.extract_mut_shorts();
                let v = v as i16;
                array_store!(ary, pos, v);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn iastore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_mut_type_array();
                let ary = ary.extract_mut_ints();
                array_store!(ary, pos, v);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn lastore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_mut_type_array();
                let ary = ary.extract_mut_longs();
                array_store!(ary, pos, v);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn fastore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_mut_type_array();
                let ary = ary.extract_mut_floats();
                array_store!(ary, pos, v);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn dastore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        let pos = area.stack.pop_int();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_mut_type_array();
                let ary = ary.extract_mut_doubles();
                array_store!(ary, pos, v);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn aastore(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        let pos = area.stack.pop_int();
        let ary_rf = area.stack.pop_ref();
        drop(area);

        match ary_rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_mut_array();
                let ary = &mut ary.elements;
                array_store!(ary, pos, v);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn pop(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.drop_top();
    }

    #[inline]
    pub fn pop2(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.drop_top();
        area.stack.drop_top();
    }

    #[inline]
    pub fn dup(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.dup();
    }

    #[inline]
    pub fn dup_x1(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.dup_x1();
    }

    #[inline]
    pub fn dup_x2(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.dup_x2();
    }

    #[inline]
    pub fn dup2(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.dup2();
    }

    #[inline]
    pub fn dup2_x1(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.dup2_x1();
    }

    #[inline]
    pub fn dup2_x2(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.dup2_x2();
    }

    #[inline]
    pub fn swap(&self) {
        let mut area = self.frame.area.write().unwrap();
        area.stack.swap();
    }

    #[inline]
    pub fn iadd(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let v = v1.wrapping_add(v2);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn ladd(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        let v = v1.wrapping_add(v2);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fadd(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        area.stack.push_float(v1 + v2);
    }

    #[inline]
    pub fn dadd(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        area.stack.push_double(v1 + v2);
    }

    #[inline]
    pub fn isub(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let v = v1.wrapping_sub(v2);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn lsub(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        let v = v1.wrapping_sub(v2);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fsub(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        area.stack.push_float(v1 - v2);
    }

    #[inline]
    pub fn dsub(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        area.stack.push_double(v1 - v2);
    }

    #[inline]
    pub fn imul(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let v = v1.wrapping_mul(v2);
        area.stack.push_int(v);
    }

    #[inline]
    pub fn lmul(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        let v = v1.wrapping_mul(v2);
        area.stack.push_long(v);
    }

    #[inline]
    pub fn fmul(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        area.stack.push_float(v1 * v2);
    }

    #[inline]
    pub fn dmul(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        area.stack.push_double(v1 * v2);
    }

    #[inline]
    pub fn idiv(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_int(v1 / v2);
        }
    }

    #[inline]
    pub fn ldiv(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_long(v1 / v2);
        }
    }

    #[inline]
    pub fn fdiv(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_float();
        let v1 = area.stack.pop_float();
        if v2 == 0.0 {
            drop(area);

            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_float(v1 / v2);
        }
    }

    #[inline]
    pub fn ddiv(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_double();
        let v1 = area.stack.pop_double();
        if v2 == 0.0 {
            drop(area);

            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_double(v1 / v2);
        }
    }

    #[inline]
    pub fn irem(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_int(v1 - (v1 / v2) * v2);
        }
    }

    #[inline]
    pub fn lrem(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        if v2 == 0 {
            drop(area);

            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            area.stack.push_long(v1 - (v1 / v2) * v2);
        }
    }

    #[inline]
    pub fn frem(&self) {
        panic!("Use of deprecated instruction frem, please check your Java compiler");
    }

    #[inline]
    pub fn drem(&self) {
        panic!("Use of deprecated instruction drem, please check your Java compiler");
    }

    #[inline]
    pub fn ineg(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.stack.push_int(-v);
    }

    #[inline]
    pub fn lneg(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.stack.push_long(-v);
    }

    #[inline]
    pub fn fneg(&self) {
        panic!("Use of deprecated instruction fneg, please check your Java compiler");
    }

    #[inline]
    pub fn dneg(&self) {
        panic!("Use of deprecated instruction dneg, please check your Java compiler");
    }

    #[inline]
    pub fn ishl(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let s = v2 & 0x1F;
        //        info!("ishl v2={}, v1={}, s={}, v={}", v2, v1, s, (v1 << s));
        area.stack.push_int(v1 << s);
    }

    #[inline]
    pub fn lshl(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        area.stack.push_long(v1 << s);
    }

    #[inline]
    pub fn ishr(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        let s = v2 & 0x1F;
        area.stack.push_int(v1 >> s);
    }

    #[inline]
    pub fn lshr(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        area.stack.push_long(v1 >> s);
    }

    #[inline]
    pub fn iushr(&self) {
        let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn lushr(&self) {
        let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn iand(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        area.stack.push_int(v1 & v2);
    }

    #[inline]
    pub fn land(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        area.stack.push_long(v1 & v2);
    }

    #[inline]
    pub fn ior(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        area.stack.push_int(v1 | v2);
    }

    #[inline]
    pub fn lor(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        area.stack.push_long(v1 | v2);
    }

    #[inline]
    pub fn ixor(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();
        area.stack.push_int(v1 ^ v2);
    }

    #[inline]
    pub fn lxor(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_long();
        let v1 = area.stack.pop_long();
        area.stack.push_long(v1 ^ v2);
    }

    #[inline]
    pub fn iinc(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pos = if op_widen {
            self.read_u2()
        } else {
            self.read_u1()
        };

        let factor = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            (self.read_u2() as i16) as i32
        } else {
            (self.read_byte() as i8) as i32
        };

        let mut area = self.frame.area.write().unwrap();
        let v = area.local.get_int(pos);
        let v = v.wrapping_add(factor);
        area.local.set_int(pos, v);
    }

    #[inline]
    pub fn i2l(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.stack.push_long(v as i64);
    }

    #[inline]
    pub fn i2f(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.stack.push_float(v as f32);
    }

    #[inline]
    pub fn i2d(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        area.stack.push_double(v as f64);
    }

    #[inline]
    pub fn l2i(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.stack.push_int(v as i32);
    }

    #[inline]
    pub fn l2f(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.stack.push_float(v as f32);
    }

    #[inline]
    pub fn l2d(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        area.stack.push_double(v as f64);
    }

    #[inline]
    pub fn f2i(&self) {
        let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn f2l(&self) {
        let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn f2d(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        area.stack.push_double(v as f64);
    }

    #[inline]
    pub fn d2i(&self) {
        let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn d2l(&self) {
        let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn d2f(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        area.stack.push_float(v as f32);
    }

    #[inline]
    pub fn i2b(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let v = v as i8;
        area.stack.push_int(v as i32);
    }

    #[inline]
    pub fn i2c(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let v = v as u16;
        area.stack.push_int(v as i32);
    }

    #[inline]
    pub fn i2s(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let v = v as i16;
        area.stack.push_int(v as i32);
    }

    #[inline]
    pub fn lcmp(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v1 = area.stack.pop_long();
        let v2 = area.stack.pop_long();
        let v = match v1.cmp(&v2) {
            std::cmp::Ordering::Greater => -1,
            std::cmp::Ordering::Less => 1,
            std::cmp::Ordering::Equal => 0,
        };

        area.stack.push_int(v);
    }

    #[inline]
    pub fn fcmpl(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v1 = area.stack.pop_float();
        let v2 = area.stack.pop_float();
        let v = if v1.is_nan() || v2.is_nan() {
            -1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };

        area.stack.push_int(v);
    }

    #[inline]
    pub fn fcmpg(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v1 = area.stack.pop_float();
        let v2 = area.stack.pop_float();
        let v = if v1.is_nan() || v2.is_nan() {
            1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };

        area.stack.push_int(v);
    }

    #[inline]
    pub fn dcmpl(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v1 = area.stack.pop_double();
        let v2 = area.stack.pop_double();
        let v = if v1.is_nan() || v2.is_nan() {
            -1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };

        area.stack.push_int(v);
    }

    #[inline]
    pub fn dcmpg(&self) {
        let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn ifeq(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();

        if v == 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn ifne(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();

        if v != 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn iflt(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();

        if v < 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn ifge(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();

        if v >= 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn ifgt(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();

        if v > 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn ifle(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();

        if v <= 0 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_icmpeq(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 == v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_icmpne(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 != v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_icmplt(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 < v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_icmpge(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 >= v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_icmpgt(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 > v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_icmple(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_int();
        let v1 = area.stack.pop_int();

        if v1 <= v2 {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_acmpeq(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_ref();
        let v1 = area.stack.pop_ref();

        if OopRef::is_eq(&v1, &v2) {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn if_acmpne(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v2 = area.stack.pop_ref();
        let v1 = area.stack.pop_ref();

        if !OopRef::is_eq(&v1, &v2) {
            drop(area);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn goto(&self) {
        self.goto_by_offset_hardcoded(2);
    }

    #[inline]
    pub fn jsr(&self) {
        let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        panic!("Use of deprecated instruction jsr, please check your Java compiler");
    }

    #[inline]
    pub fn ret(&self) {
        let op_widen = self.frame.op_widen.load(Ordering::Relaxed);

        let pc = if op_widen {
            self.frame.op_widen.store(false, Ordering::Relaxed);
            self.read_u2()
        } else {
            self.read_u1()
        };

        self.frame.pc.store(pc as i32, Ordering::Relaxed);
    }

    #[inline]
    pub fn table_switch(&self) {
        let mut bc = self.frame.pc.load(Ordering::Relaxed) - 1;

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
            let mut area = self.frame.area.write().unwrap();
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

    #[inline]
    pub fn lookup_switch(&self) {
        let mut bc = self.frame.pc.load(Ordering::Relaxed) - 1;

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
            let mut area = self.frame.area.write().unwrap();
            area.stack.pop_int()
        };
        match jump_table.get(&(top_value as u32)) {
            Some(position) => self.goto_abs_with_occupied(*position as i32, 1),
            None => self.goto_abs_with_occupied(default_byte as i32 + origin_bc, 1),
        }
    }

    #[inline]
    pub fn ireturn(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_int();
        let v = Oop::new_int(v);
        drop(area);

        self.set_return(Some(v));
    }

    #[inline]
    pub fn lreturn(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_long();
        let v = Oop::new_long(v);
        drop(area);

        self.set_return(Some(v));
    }

    #[inline]
    pub fn freturn(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_float();
        let v = Oop::new_float(v);
        drop(area);

        self.set_return(Some(v));
    }

    #[inline]
    pub fn dreturn(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_double();
        let v = Oop::new_double(v);
        drop(area);

        self.set_return(Some(v));
    }

    #[inline]
    pub fn areturn(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        drop(area);

        self.set_return(Some(v));
    }

    #[inline]
    pub fn return_void(&self) {
        self.set_return(None);
    }

    #[inline]
    pub fn get_static(&self) {
        let cp_idx = self.read_i2();
        self.get_field_helper(oop_consts::get_null(), cp_idx, true);
    }

    #[inline]
    pub fn put_static(&self) {
        let cp_idx = self.read_i2();
        self.put_field_helper(cp_idx, true);
    }

    #[inline]
    pub fn get_field(&self) {
        let cp_idx = self.read_i2();

        let mut area = self.frame.area.write().unwrap();
        let rf = area.stack.pop_ref();
        drop(area);

        match rf {
            Oop::Null => {
                exception::meet_ex(cls_const::J_NPE, None);
            }
            _ => {
                self.get_field_helper(rf, cp_idx, false);
            }
        }
    }

    #[inline]
    pub fn put_field(&self) {
        let cp_idx = self.read_i2();
        self.put_field_helper(cp_idx, false);
    }

    #[inline]
    pub fn invoke_virtual(&self) {
        let cp_idx = self.read_i2();
        self.invoke_helper(false, cp_idx as usize, false);
    }

    #[inline]
    pub fn invoke_special(&self) {
        let cp_idx = self.read_i2();
        self.invoke_helper(false, cp_idx as usize, true);
    }

    #[inline]
    pub fn invoke_static(&self) {
        let cp_idx = self.read_i2();
        self.invoke_helper(true, cp_idx as usize, true);
    }

    #[inline]
    pub fn invoke_interface(&self) {
        let cp_idx = self.read_i2();
        let _count = self.read_u1();
        let zero = self.read_u1();

        if zero != 0 {
            warn!("interpreter: invalid invokeinterface: the value of the fourth operand byte must always be zero.");
        }

        self.invoke_helper(false, cp_idx as usize, false);
    }

    #[inline]
    pub fn invoke_dynamic(&self) {
        //todo: impl
        unimplemented!()
    }

    #[inline]
    pub fn new_(&self) {
        let cp_idx = self.read_i2();

        let class = {
            match runtime::require_class2(cp_idx as u16, &self.frame.cp) {
                Some(class) => {
                    oop::class::init_class(&class);
                    oop::class::init_class_fully(&class);

                    class
                }
                None => unreachable!("Cannot get class info from constant pool"),
            }
        };

        let mut area = self.frame.area.write().unwrap();
        let v = oop::Oop::new_inst(class);
        area.stack.push_ref(v);
    }

    #[inline]
    pub fn new_array(&self) {
        let t = self.read_byte();

        let mut area = self.frame.area.write().unwrap();
        let len = area.stack.pop_int();

        if len < 0 {
            drop(area);
            exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
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

    #[inline]
    pub fn anew_array(&self) {
        let cp_idx = self.read_i2();

        let mut area = self.frame.area.write().unwrap();
        let length = area.stack.pop_int();
        drop(area);

        //        info!("anew_array length={}", length);
        if length < 0 {
            exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
        } else {
            let class = match runtime::require_class2(cp_idx as u16, &self.frame.cp) {
                Some(class) => class,
                None => panic!("Cannot get class info from constant pool"),
            };

            oop::class::init_class(&class);
            oop::class::init_class_fully(&class);

            let (name, cl) = {
                let class = class.get_class();
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
                (name, class.class_loader)
            };

            trace!("anew_array name={}", unsafe {
                std::str::from_utf8_unchecked(name.as_slice())
            });
            match runtime::require_class(cl, name) {
                Some(ary_cls_obj) => {
                    oop::class::init_class(&ary_cls_obj);
                    oop::class::init_class_fully(&ary_cls_obj);

                    let mut area = self.frame.area.write().unwrap();
                    let ary = Oop::new_ref_ary(ary_cls_obj, length as usize);
                    area.stack.push_ref(ary);
                }
                None => unreachable!(),
            }
        }
    }

    #[inline]
    pub fn array_length(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();

        match v {
            Oop::Null => {
                drop(area);
                exception::meet_ex(cls_const::J_NPE, None)
            }
            Oop::Ref(rf) => {
                let v = rf.get_raw_ptr();
                unsafe {
                    match &(*v).v {
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
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn athrow(&self, jt: JavaThreadRef) {
        let mut area = self.frame.area.write().unwrap();
        let ex = area.stack.pop_ref();
        drop(area);

        jt.write().unwrap().set_ex(ex);
    }

    #[inline]
    pub fn check_cast(&self) {
        self.check_cast_helper(true);
    }

    #[inline]
    pub fn instance_of(&self) {
        self.check_cast_helper(false);
    }

    #[inline]
    pub fn monitor_enter(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();
        drop(area);

        match v {
            Oop::Null => {
                exception::meet_ex(cls_const::J_NPE, None);
            }
            Oop::Ref(v) => v.monitor_enter(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn monitor_exit(&self) {
        let mut area = self.frame.area.write().unwrap();
        let mut v = area.stack.pop_ref();
        drop(area);

        match v {
            Oop::Null => {
                exception::meet_ex(cls_const::J_NPE, None);
            }
            Oop::Ref(v) => v.monitor_exit(),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn wide(&self) {
        self.frame.op_widen.store(true, Ordering::Relaxed);
    }

    #[inline]
    pub fn multi_anew_array(&self) {
        let cp_idx = self.read_u2();
        let dimension = self.read_u1();

        let mut lens = Vec::new();
        let mut area = self.frame.area.write().unwrap();
        for _ in 0..dimension {
            let sub = area.stack.pop_int();
            //todo: check java/lang/NegativeArraySizeException
            lens.push(sub);
        }
        drop(area);

        let cls = require_class2(cp_idx as u16, &self.frame.cp).unwrap();
        let ary = new_multi_object_array_helper(cls, &lens, 0);

        let mut area = self.frame.area.write().unwrap();
        area.stack.push_ref(ary);
    }

    #[inline]
    pub fn if_null(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();

        match v {
            Oop::Null => {
                drop(area);
                self.goto_by_offset_hardcoded(2)
            }
            _ => {
                let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
            }
        }
    }

    #[inline]
    pub fn if_non_null(&self) {
        let mut area = self.frame.area.write().unwrap();
        let v = area.stack.pop_ref();

        match v {
            Oop::Null => {
                let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
            }
            _ => {
                drop(area);
                self.goto_by_offset_hardcoded(2)
            }
        }
    }

    #[inline]
    pub fn goto_w(&self) {
        let _ = self.frame.pc.fetch_add(4, Ordering::Relaxed);
        panic!("Use of deprecated instruction goto_w, please check your Java compiler")
    }

    #[inline]
    pub fn jsr_w(&self) {
        let _ = self.frame.pc.fetch_add(4, Ordering::Relaxed);
        panic!("Use of deprecated instruction jsr_w, please check your Java compiler")
    }

    #[inline]
    pub fn other_wise(&self) {
        let pc = self.frame.pc.load(Ordering::Relaxed);
        let pc = pc - 1;
        panic!(
            "Use of undefined bytecode: {} at {}",
            self.frame.code[pc as usize], pc
        );
    }
}

fn new_multi_object_array_helper(cls: ClassRef, lens: &[i32], idx: usize) -> Oop {
    let length = lens[idx] as usize;

    let down_type = {
        let cls = cls.get_class();
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
