use crate::oop::{
    self, consts as oop_consts, field, Class, ClassKind, Oop, OopPtr, TypeArrayDesc, ValueType,
};
use crate::runtime::local::Local;
use crate::runtime::stack::Stack;
use crate::runtime::{
    self, cmp, exception, require_class, require_class2, require_class3, thread, DataArea, Frame,
    JavaCall,
};
use crate::types::*;
use crate::util;
use classfile::{
    constant_pool::get_utf8 as get_cp_utf8, consts as cls_const, ClassFile, ConstantPool,
    ConstantPoolType, OpCode, U1, U2,
};
use nix::sys::socket::SockType::Datagram;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
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
            $area.push_int($ary[$pos as usize] as i32);
        }
    };
}

macro_rules! read_byte {
    ($pc:expr, $code:expr) => {{
        let pc = $pc.fetch_add(1, Ordering::Relaxed);
        $code[pc as usize]
    }};
}

macro_rules! read_i2 {
    ($pc:expr, $code:expr) => {{
        let h = read_byte!($pc, $code) as i16;
        let l = read_byte!($pc, $code) as i16;
        (h << 8 | l) as i32
    }};
}

macro_rules! read_u1 {
    ($pc:expr, $code:expr) => {{
        let pc = $pc.fetch_add(1, Ordering::Relaxed);
        $code[pc as usize] as usize
    }};
}

macro_rules! read_u2 {
    ($pc:expr, $code:expr) => {{
        read_u1!($pc, $code) << 8 | read_u1!($pc, $code)
    }};
}

macro_rules! opcode_const {
    (null, $interp:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_null();
    };
    (m1, $interp:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_const_m1();
    };
    (0, $interp:ident, $with_nop:expr) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_const0($with_nop);
    };
    (1, $interp:ident, $with_nop:expr) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_const1($with_nop);
    };
    (2, $interp:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_const2();
    };
    (3, $interp:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_const3();
    };
    (4, $interp:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_const4();
    };
    (5, $interp:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_const5();
    };
}

macro_rules! opcode_load {
    (int, $interp:ident, $pos:expr) => {
        let v = $interp.local.get_int($pos);
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_int(v);
    };
    (long, $interp:ident, $pos:expr) => {
        let v = $interp.local.get_long($pos);
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_long(v);
    };
    (float, $interp:ident, $pos:expr) => {
        let v = $interp.local.get_float($pos);
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_float(v);
    };
    (double, $interp:ident, $pos:expr) => {
        let v = $interp.local.get_double($pos);
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_double(v);
    };
    (a, $interp:ident, $pos:expr) => {
        let v = $interp.local.get_ref($pos);
        let mut stack = $interp.frame.area.stack.borrow_mut();
        stack.push_ref(v, false);
    };
}

macro_rules! opcode_store {
    (int, $interp:ident, $pos:expr) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        $interp.local.set_int($pos, v);
    };
    (long, $interp:ident, $pos:expr) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        $interp.local.set_long($pos, v);
    };
    (float, $interp:ident, $pos:expr) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        $interp.local.set_float($pos, v);
    };
    (double, $interp:ident, $pos:expr) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        $interp.local.set_double($pos, v);
    };
    (a, $interp:ident, $pos:expr) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        $interp.local.set_ref($pos, v);
    };
}

macro_rules! opcode_math_op {
    (int, $interp:ident, $op:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        let v = v1.$op(v2);
        stack.push_int(v);
    };
    (long, $interp:ident, $op:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        let v = v1.$op(v2);
        stack.push_long(v);
    };
    (float, $interp:ident, $op:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v2 = stack.pop_float();
        let v1 = stack.pop_float();
        let v = v1.$op(v2);
        stack.push_float(v);
    };
    (double, $interp:ident, $op:ident) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v2 = stack.pop_double();
        let v1 = stack.pop_double();
        let v = v1.$op(v2);
        stack.push_double(v);
    };
}

macro_rules! opcode_if {
    ($interp:ident, $op:tt) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v1 $op v2 {
            drop(stack);
            $interp.goto_by_offset_hardcoded(2);
        } else {
            let _ = $interp.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    };
    ($interp:ident, $op:tt, 0) => {
        let mut stack = $interp.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        if v $op 0 {
            drop(stack);
            $interp.goto_by_offset_hardcoded(2);
        } else {
            let _ = $interp.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    };
}

pub struct Interp<'a> {
    frame: RwLockReadGuard<'a, Box<Frame>>,
    local: Local,
    cp: ConstantPool,
    code: Arc<Vec<U1>>,
    op_widen: bool,
}

impl<'a> Interp<'a> {
    pub fn new(frame: RwLockReadGuard<'a, Box<Frame>>, local: Local) -> Self {
        let cp = frame.cp.clone();
        let code = frame.code.clone();
        let op_widen = false;
        Self {
            frame,
            local,
            cp,
            code,
            op_widen,
        }
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
    pub fn run(&mut self) {
        let jt = runtime::thread::current_java_thread();
        let codes = self.code.clone();

        loop {
            let code = read_byte!(self.frame.pc, codes);
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
                    opcode_const!(null, self);
                }
                OpCode::iconst_m1 => {
                    opcode_const!(m1, self);
                }
                OpCode::iconst_0 => {
                    opcode_const!(0, self, false);
                }
                OpCode::iconst_1 => {
                    opcode_const!(1, self, false);
                }
                OpCode::iconst_2 => {
                    opcode_const!(2, self);
                }
                OpCode::iconst_3 => {
                    opcode_const!(3, self);
                }
                OpCode::iconst_4 => {
                    opcode_const!(4, self);
                }
                OpCode::iconst_5 => {
                    opcode_const!(5, self);
                }
                OpCode::lconst_0 => {
                    opcode_const!(0, self, true);
                }
                OpCode::lconst_1 => {
                    opcode_const!(1, self, true);
                }
                OpCode::fconst_0 => {
                    opcode_const!(0, self, false);
                }
                OpCode::fconst_1 => {
                    opcode_const!(1, self, false);
                }
                OpCode::fconst_2 => {
                    opcode_const!(2, self);
                }
                OpCode::dconst_0 => {
                    opcode_const!(0, self, true);
                }
                OpCode::dconst_1 => {
                    opcode_const!(1, self, true);
                }
                OpCode::bipush => self.bipush(),
                OpCode::sipush => self.sipush(),
                OpCode::ldc => self.ldc(),
                OpCode::ldc_w => self.ldc_w(),
                OpCode::ldc2_w => self.ldc2_w(),
                OpCode::iload => {
                    let pos = self.opcode_pos();
                    opcode_load!(int, self, pos);
                }
                OpCode::lload => {
                    let pos = self.opcode_pos();
                    opcode_load!(long, self, pos);
                }
                OpCode::fload => {
                    let pos = self.opcode_pos();
                    opcode_load!(float, self, pos);
                }
                OpCode::dload => {
                    let pos = self.opcode_pos();
                    opcode_load!(double, self, pos);
                }
                OpCode::aload => {
                    let pos = self.opcode_pos();
                    opcode_load!(a, self, pos);
                }
                OpCode::iload_0 => {
                    opcode_load!(int, self, 0);
                }
                OpCode::iload_1 => {
                    opcode_load!(int, self, 1);
                }
                OpCode::iload_2 => {
                    opcode_load!(int, self, 2);
                }
                OpCode::iload_3 => {
                    opcode_load!(int, self, 3);
                }
                OpCode::lload_0 => {
                    opcode_load!(long, self, 0);
                }
                OpCode::lload_1 => {
                    opcode_load!(long, self, 1);
                }
                OpCode::lload_2 => {
                    opcode_load!(long, self, 2);
                }
                OpCode::lload_3 => {
                    opcode_load!(long, self, 3);
                }
                OpCode::fload_0 => {
                    opcode_load!(float, self, 0);
                }
                OpCode::fload_1 => {
                    opcode_load!(float, self, 1);
                }
                OpCode::fload_2 => {
                    opcode_load!(float, self, 2);
                }
                OpCode::fload_3 => {
                    opcode_load!(float, self, 3);
                }
                OpCode::dload_0 => {
                    opcode_load!(double, self, 0);
                }
                OpCode::dload_1 => {
                    opcode_load!(double, self, 1);
                }
                OpCode::dload_2 => {
                    opcode_load!(double, self, 2);
                }
                OpCode::dload_3 => {
                    opcode_load!(double, self, 3);
                }
                OpCode::aload_0 => {
                    opcode_load!(a, self, 0);
                }
                OpCode::aload_1 => {
                    opcode_load!(a, self, 1);
                }
                OpCode::aload_2 => {
                    opcode_load!(a, self, 2);
                }
                OpCode::aload_3 => {
                    opcode_load!(a, self, 3);
                }
                OpCode::iaload => self.iaload(),
                OpCode::laload => self.laload(),
                OpCode::faload => self.faload(),
                OpCode::daload => self.daload(),
                OpCode::aaload => self.aaload(),
                OpCode::baload => self.baload(),
                OpCode::caload => self.caload(),
                OpCode::saload => self.saload(),
                OpCode::istore => {
                    let pos = self.opcode_pos();
                    opcode_store!(int, self, pos);
                }
                OpCode::lstore => {
                    let pos = self.opcode_pos();
                    opcode_store!(long, self, pos);
                }
                OpCode::fstore => {
                    let pos = self.opcode_pos();
                    opcode_store!(float, self, pos);
                }
                OpCode::dstore => {
                    let pos = self.opcode_pos();
                    opcode_store!(double, self, pos);
                }
                OpCode::astore => {
                    let pos = self.opcode_pos();
                    opcode_store!(a, self, pos);
                }
                OpCode::istore_0 => {
                    opcode_store!(int, self, 0);
                }
                OpCode::istore_1 => {
                    opcode_store!(int, self, 1);
                }
                OpCode::istore_2 => {
                    opcode_store!(int, self, 2);
                }
                OpCode::istore_3 => {
                    opcode_store!(int, self, 3);
                }
                OpCode::lstore_0 => {
                    opcode_store!(long, self, 0);
                }
                OpCode::lstore_1 => {
                    opcode_store!(long, self, 1);
                }
                OpCode::lstore_2 => {
                    opcode_store!(long, self, 2);
                }
                OpCode::lstore_3 => {
                    opcode_store!(long, self, 3);
                }
                OpCode::fstore_0 => {
                    opcode_store!(float, self, 0);
                }
                OpCode::fstore_1 => {
                    opcode_store!(float, self, 1);
                }
                OpCode::fstore_2 => {
                    opcode_store!(float, self, 2);
                }
                OpCode::fstore_3 => {
                    opcode_store!(float, self, 3);
                }
                OpCode::dstore_0 => {
                    opcode_store!(double, self, 0);
                }
                OpCode::dstore_1 => {
                    opcode_store!(double, self, 1);
                }
                OpCode::dstore_2 => {
                    opcode_store!(double, self, 2);
                }
                OpCode::dstore_3 => {
                    opcode_store!(double, self, 3);
                }
                OpCode::astore_0 => {
                    opcode_store!(a, self, 0);
                }
                OpCode::astore_1 => {
                    opcode_store!(a, self, 1);
                }
                OpCode::astore_2 => {
                    opcode_store!(a, self, 2);
                }
                OpCode::astore_3 => {
                    opcode_store!(a, self, 3);
                }
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
                OpCode::iadd => {
                    opcode_math_op!(int, self, wrapping_add);
                }
                OpCode::ladd => {
                    opcode_math_op!(long, self, wrapping_add);
                }
                OpCode::fadd => {
                    use std::ops::Add;
                    opcode_math_op!(float, self, add);
                }
                OpCode::dadd => {
                    use std::ops::Add;
                    opcode_math_op!(double, self, add);
                }
                OpCode::isub => {
                    opcode_math_op!(int, self, wrapping_sub);
                }
                OpCode::lsub => {
                    opcode_math_op!(long, self, wrapping_sub);
                }
                OpCode::fsub => {
                    use std::ops::Sub;
                    opcode_math_op!(float, self, sub);
                }
                OpCode::dsub => {
                    use std::ops::Sub;
                    opcode_math_op!(double, self, sub);
                }
                OpCode::imul => {
                    opcode_math_op!(int, self, wrapping_mul);
                }
                OpCode::lmul => {
                    opcode_math_op!(long, self, wrapping_mul);
                }
                OpCode::fmul => {
                    use std::ops::Mul;
                    opcode_math_op!(float, self, mul);
                }
                OpCode::dmul => {
                    use std::ops::Mul;
                    opcode_math_op!(double, self, mul);
                }
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
                OpCode::iand => {
                    use std::ops::BitAnd;
                    opcode_math_op!(int, self, bitand);
                }
                OpCode::land => {
                    use std::ops::BitAnd;
                    opcode_math_op!(long, self, bitand);
                }
                OpCode::ior => {
                    use std::ops::BitOr;
                    opcode_math_op!(int, self, bitor);
                }
                OpCode::lor => {
                    use std::ops::BitOr;
                    opcode_math_op!(long, self, bitor);
                }
                OpCode::ixor => {
                    use std::ops::BitXor;
                    opcode_math_op!(int, self, bitxor);
                }
                OpCode::lxor => {
                    use std::ops::BitXor;
                    opcode_math_op!(long, self, bitxor);
                }
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
                OpCode::ifeq => {
                    opcode_if!(self, ==, 0);
                }
                OpCode::ifne => {
                    opcode_if!(self, !=, 0);
                }
                OpCode::iflt => {
                    opcode_if!(self, <, 0);
                }
                OpCode::ifge => {
                    opcode_if!(self, >=, 0);
                }
                OpCode::ifgt => {
                    opcode_if!(self, >, 0);
                }
                OpCode::ifle => {
                    opcode_if!(self, <=, 0);
                }
                OpCode::if_icmpeq => {
                    opcode_if!(self, ==);
                }
                OpCode::if_icmpne => {
                    opcode_if!(self, !=);
                }
                OpCode::if_icmplt => {
                    opcode_if!(self, <);
                }
                OpCode::if_icmpge => {
                    opcode_if!(self, >=);
                }
                OpCode::if_icmpgt => {
                    opcode_if!(self, >);
                }
                OpCode::if_icmple => {
                    opcode_if!(self, <=);
                }
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

//helper methods
impl<'a> Interp<'a> {
    fn load_constant(&self, pos: usize) {
        match &self.cp[pos] {
            ConstantPoolType::Integer { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_int2(v)
            }
            ConstantPoolType::Float { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_float2(v)
            }
            ConstantPoolType::Long { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_long2(v)
            }
            ConstantPoolType::Double { v } => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_double2(v)
            }
            ConstantPoolType::String { string_index } => {
                let s = get_cp_utf8(&self.cp, *string_index as usize);
                let s = util::oop::new_java_lang_string3(s.as_slice());

                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_ref(s, false);
            }
            ConstantPoolType::Class { name_index } => {
                let name = get_cp_utf8(&self.cp, *name_index as usize);
                let name = unsafe { std::str::from_utf8_unchecked(name.as_slice()) };
                let cl = { self.frame.class.get_class().class_loader };
                trace!("load_constant name={}, cl={:?}", name, cl);
                let class = runtime::require_class3(cl, name.as_bytes()).unwrap();
                oop::class::init_class(&class);
                oop::class::init_class_fully(&class);

                let mirror = { class.get_class().get_mirror() };
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_ref(mirror, false);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn goto_abs(&self, pc: i32) {
        self.frame.pc.store(pc, Ordering::Relaxed);
    }

    #[inline]
    fn goto_by_offset(&self, branch: i32) {
        let _ = self.frame.pc.fetch_add(branch, Ordering::Relaxed);
    }

    #[inline]
    fn goto_by_offset_with_occupied(&self, branch: i32, occupied: i32) {
        self.goto_by_offset(branch);
        self.goto_by_offset(-(occupied - 1));
    }

    #[inline]
    fn goto_by_offset_hardcoded(&self, occupied: i32) {
        let codes = &self.code;
        let pc = self.frame.pc.load(Ordering::Relaxed);
        let high = codes[pc as usize] as i16;
        let low = codes[(pc + 1) as usize] as i16;
        let branch = (high << 8) | low;

        self.goto_by_offset_with_occupied(branch as i32, occupied);
    }

    #[inline]
    fn goto_abs_with_occupied(&self, pc: i32, occupied: i32) {
        self.goto_abs(pc);
        self.goto_by_offset(-(occupied - 1));
    }

    fn set_return(&self, v: Option<Oop>) {
        let mut return_v = self.frame.area.return_v.borrow_mut();
        *return_v = v;
    }

    fn get_field_helper(&self, receiver: Oop, idx: usize, is_static: bool) {
        let class = self.frame.class.extract_inst();
        let fir = class.cp_cache.get_field(idx, is_static);
        debug_assert_eq!(fir.field.is_static(), is_static);
        trace!("get_field_helper={:?}, is_static={}", fir.field, is_static);
        let value_type = fir.field.value_type;
        let v = if is_static {
            let class = fir.field.class.get_class();
            class.get_static_field_value(fir.clone())
        } else {
            let rf = receiver.extract_ref();
            Class::get_field_value2(rf, fir.offset)
        };
        let with_nop = match value_type {
            ValueType::DOUBLE | ValueType::LONG => true,
            _ => false,
        };
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.push_ref(v, with_nop);
    }

    fn pop_value(&self, vt: ValueType) -> Oop {
        let mut stack = self.frame.area.stack.borrow_mut();
        match vt {
            ValueType::INT
            | ValueType::SHORT
            | ValueType::CHAR
            | ValueType::BOOLEAN
            | ValueType::BYTE => {
                let v = stack.pop_int();
                Oop::new_int(v)
            }
            ValueType::FLOAT => {
                let v = stack.pop_float();
                Oop::new_float(v)
            }
            ValueType::DOUBLE => {
                let v = stack.pop_double();
                Oop::new_double(v)
            }
            ValueType::LONG => {
                let v = stack.pop_long();
                Oop::new_long(v)
            }
            ValueType::ARRAY | ValueType::OBJECT => stack.pop_ref(),
            _ => unreachable!(),
        }
    }

    fn put_field_helper(&self, idx: usize, is_static: bool) {
        let class = self.frame.class.extract_inst();
        let fir = class.cp_cache.get_field(idx, is_static);
        debug_assert_eq!(fir.field.is_static(), is_static);
        trace!("put_field_helper={:?}, is_static={}", fir.field, is_static);
        let value_type = fir.field.value_type;
        //        info!("value_type = {:?}", value_type);
        let v = self.pop_value(value_type);
        if is_static {
            let mut class = fir.field.class.get_mut_class();
            class.put_static_field_value(fir.clone(), v);
        } else {
            let receiver = {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.pop_ref()
            };
            match receiver {
                Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
                _ => Class::put_field_value2(receiver.extract_ref(), fir.offset, v),
            }
        }
    }

    fn invoke_helper(&self, is_static: bool, idx: usize, force_no_resolve: bool) {
        let class = self.frame.class.extract_inst();
        let mir = class.cp_cache.get_method(idx);
        let caller = match &mir.method.signature.retype {
            classfile::SignatureType::Void => None,
            _ => Some(&self.frame.area),
        };
        debug_assert_eq!(mir.method.is_static(), is_static);
        if let Ok(mut jc) = runtime::invoke::JavaCall::new(&self.frame.area, mir) {
            jc.invoke(caller, force_no_resolve);
        }
    }

    pub fn check_cast_helper(&self, is_cast: bool) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = read_i2!(pc, codes);
        let target_cls = require_class2(cp_idx as U2, &self.cp).unwrap();
        let obj_rf = self.pop_value(ValueType::OBJECT);
        let obj_rf_clone = obj_rf.clone();
        let op_check_cast = |r: bool, obj_cls: ClassRef, target_cls: ClassRef| {
            if r {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.push_ref(obj_rf_clone, false);
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
            let mut stack = self.frame.area.stack.borrow_mut();
            if r {
                stack.push_const1(false);
            } else {
                stack.push_const0(false);
            }
        };

        match obj_rf {
            Oop::Null => {
                let mut stack = self.frame.area.stack.borrow_mut();
                if is_cast {
                    stack.push_ref(obj_rf, false);
                } else {
                    stack.push_const0(false);
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

    #[inline]
    fn opcode_pos(&mut self) -> usize {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let op_widen = self.op_widen;
        if op_widen {
            self.op_widen = false;
            read_u2!(pc, codes)
        } else {
            read_u1!(pc, codes)
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
                .find_exception_handler(&self.cp, pc as u16, ex_cls)
        };

        match handler {
            Some(pc) => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.clear();
                stack.push_ref(ex, false);
                drop(stack);

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
    fn sipush(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let v = read_i2!(pc, codes);
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.push_int(v);
    }

    #[inline]
    fn bipush(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let v = (read_byte!(pc, codes) as i8) as i32;
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.push_int(v);
    }

    #[inline]
    fn ldc(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let pos = read_u1!(pc, codes);
        self.load_constant(pos);
    }

    #[inline]
    fn ldc_w(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let pos = read_u2!(pc, codes);
        self.load_constant(pos);
    }

    #[inline]
    fn ldc2_w(&self) {
        self.ldc_w();
    }

    #[inline]
    fn iaload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_ints();
                iarray_load!(stack, ary, pos);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn saload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_shorts();
                iarray_load!(stack, ary, pos);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn caload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        match rf {
            Oop::Null => exception::meet_ex(cls_const::J_NPE, None),
            Oop::Ref(rf) => {
                let ary = rf.extract_type_array();
                let ary = ary.extract_chars();
                iarray_load!(stack, ary, pos);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn baload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
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
                                stack.push_int(v as i32);
                            }
                            TypeArrayDesc::Bool(ary) => {
                                let v = ary[pos as usize];
                                stack.push_int(v as i32);
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
    fn laload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
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
                    stack.push_long(v);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn faload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
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
                    stack.push_float(v);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn daload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
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
                    stack.push_double(v);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn aaload(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
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
                    stack.push_ref(v, false);
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn bastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);

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
    fn castore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);

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
    fn sastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);

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
    fn iastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);

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
    fn lastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);

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
    fn fastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);

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
    fn dastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        let pos = stack.pop_int();
        let rf = stack.pop_ref();
        drop(stack);

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
    fn aastore(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        let pos = stack.pop_int();
        let ary_rf = stack.pop_ref();
        drop(stack);

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
    fn pop(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.drop_top();
    }

    #[inline]
    fn pop2(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.drop_top();
        stack.drop_top();
    }

    #[inline]
    fn dup(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.dup();
    }

    #[inline]
    fn dup_x1(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.dup_x1();
    }

    #[inline]
    fn dup_x2(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.dup_x2();
    }

    #[inline]
    fn dup2(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.dup2();
    }

    #[inline]
    fn dup2_x1(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.dup2_x1();
    }

    #[inline]
    fn dup2_x2(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.dup2_x2();
    }

    #[inline]
    fn swap(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.swap();
    }

    #[inline]
    fn idiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_int(v1 / v2);
        }
    }

    #[inline]
    fn ldiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_long(v1 / v2);
        }
    }

    #[inline]
    fn fdiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_float();
        let v1 = stack.pop_float();
        if v2 == 0.0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_float(v1 / v2);
        }
    }

    #[inline]
    fn ddiv(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_double();
        let v1 = stack.pop_double();
        if v2 == 0.0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_double(v1 / v2);
        }
    }

    #[inline]
    fn irem(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_int(v1 - (v1 / v2) * v2);
        }
    }

    #[inline]
    fn lrem(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_long();
        let v1 = stack.pop_long();
        if v2 == 0 {
            drop(stack);
            exception::meet_ex(
                cls_const::J_ARITHMETIC_EX,
                Some("divide by zero".to_string()),
            );
        } else {
            stack.push_long(v1 - (v1 / v2) * v2);
        }
    }

    #[inline]
    fn frem(&self) {
        panic!("Use of deprecated instruction frem, please check your Java compiler");
    }

    #[inline]
    fn drem(&self) {
        panic!("Use of deprecated instruction drem, please check your Java compiler");
    }

    #[inline]
    fn ineg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        stack.push_int(-v);
    }

    #[inline]
    fn lneg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        stack.push_long(-v);
    }

    #[inline]
    fn fneg(&self) {
        panic!("Use of deprecated instruction fneg, please check your Java compiler");
    }

    #[inline]
    fn dneg(&self) {
        panic!("Use of deprecated instruction dneg, please check your Java compiler");
    }

    #[inline]
    fn ishl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        let s = v2 & 0x1F;
        stack.push_int(v1 << s);
    }

    #[inline]
    fn lshl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        stack.push_long(v1 << s);
    }

    #[inline]
    fn ishr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        let s = v2 & 0x1F;
        stack.push_int(v1 >> s);
    }

    #[inline]
    fn lshr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_long();
        let s = (v2 & 0x3F) as i64;
        stack.push_long(v1 >> s);
    }

    #[inline]
    fn iushr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int() as u32;
        let s = (v2 & 0x1F) as u32;
        stack.push_int((v1 >> s) as i32);
    }

    #[inline]
    fn lushr(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_long() as u64;
        let s = (v2 & 0x3F) as u64;
        stack.push_long((v1 >> s) as i64);
    }

    #[inline]
    fn iinc(&mut self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let op_widen = self.op_widen;
        let pos;
        let factor;
        if op_widen {
            self.op_widen = false;
            pos = read_u2!(pc, codes);
            factor = (read_u2!(pc, codes) as i16) as i32
        } else {
            pos = read_u1!(pc, codes);
            factor = (read_byte!(pc, codes) as i8) as i32
        };

        let v = self.local.get_int(pos);
        let v = v.wrapping_add(factor);
        self.local.set_int(pos, v);
    }

    #[inline]
    fn i2l(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        stack.push_long(v as i64);
    }

    #[inline]
    fn i2f(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        stack.push_float(v as f32);
    }

    #[inline]
    fn i2d(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        stack.push_double(v as f64);
    }

    #[inline]
    fn l2i(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        stack.push_int(v as i32);
    }

    #[inline]
    fn l2f(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        stack.push_float(v as f32);
    }

    #[inline]
    fn l2d(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        stack.push_double(v as f64);
    }

    #[inline]
    fn f2i(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        if v.is_nan() {
            stack.push_int(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                stack.push_int(i32::MAX);
            } else {
                stack.push_int(i32::MIN);
            }
        } else {
            stack.push_int(v as i32);
        }
    }

    #[inline]
    fn f2l(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        if v.is_nan() {
            stack.push_long(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                stack.push_long(i64::MAX);
            } else {
                stack.push_long(i64::MIN);
            }
        } else {
            stack.push_long(v as i64);
        }
    }

    #[inline]
    fn f2d(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        stack.push_double(v as f64);
    }

    #[inline]
    fn d2i(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        if v.is_nan() {
            stack.push_int(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                stack.push_int(i32::MAX);
            } else {
                stack.push_int(i32::MIN);
            }
        } else {
            stack.push_int(v as i32);
        }
    }

    #[inline]
    fn d2l(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        if v.is_nan() {
            stack.push_long(0);
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                stack.push_long(i64::MAX);
            } else {
                stack.push_long(i64::MIN);
            }
        } else {
            stack.push_long(v as i64);
        }
    }

    #[inline]
    fn d2f(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        stack.push_float(v as f32);
    }

    #[inline]
    fn i2b(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let v = v as i8;
        stack.push_int(v as i32);
    }

    #[inline]
    fn i2c(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let v = v as u16;
        stack.push_int(v as i32);
    }

    #[inline]
    fn i2s(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let v = v as i16;
        stack.push_int(v as i32);
    }

    #[inline]
    fn lcmp(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_long();
        let v2 = stack.pop_long();
        let v = match v1.cmp(&v2) {
            std::cmp::Ordering::Greater => -1,
            std::cmp::Ordering::Less => 1,
            std::cmp::Ordering::Equal => 0,
        };
        stack.push_int(v);
    }

    #[inline]
    fn fcmpl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_float();
        let v2 = stack.pop_float();
        let v = if v1.is_nan() || v2.is_nan() {
            -1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };
        stack.push_int(v);
    }

    #[inline]
    fn fcmpg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_float();
        let v2 = stack.pop_float();
        let v = if v1.is_nan() || v2.is_nan() {
            1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };
        stack.push_int(v);
    }

    #[inline]
    fn dcmpl(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_double();
        let v2 = stack.pop_double();
        let v = if v1.is_nan() || v2.is_nan() {
            -1
        } else if v1 > v2 {
            -1
        } else if v1 < v2 {
            1
        } else {
            0
        };
        stack.push_int(v);
    }

    #[inline]
    fn dcmpg(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v1 = stack.pop_double();
        let v2 = stack.pop_double();
        if v1.is_nan() || v2.is_nan() {
            stack.push_int(1);
        } else if v1 > v2 {
            stack.push_int(-1);
        } else if v1 < v2 {
            stack.push_int(1);
        } else {
            stack.push_int(0);
        }
    }

    #[inline]
    fn if_acmpeq(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_ref();
        let v1 = stack.pop_ref();
        if OopPtr::is_eq(&v1, &v2) {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    fn if_acmpne(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_ref();
        let v1 = stack.pop_ref();
        if !OopPtr::is_eq(&v1, &v2) {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }

    #[inline]
    fn goto(&self) {
        self.goto_by_offset_hardcoded(2);
    }

    #[inline]
    fn jsr(&self) {
        let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        panic!("Use of deprecated instruction jsr, please check your Java compiler");
    }

    #[inline]
    fn ret(&mut self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let op_widen = self.op_widen;
        let new_pc = if op_widen {
            self.op_widen = false;
            read_u2!(pc, codes)
        } else {
            read_u1!(pc, codes)
        };
        pc.store(new_pc as i32, Ordering::Relaxed);
    }

    #[inline]
    fn table_switch(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let mut bc = pc.load(Ordering::Relaxed) - 1;

        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += (4 - bc % 4);
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;

        let default_byte = [codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]];
        let default_byte = i32::from_be_bytes(default_byte);
        let low_byte = [
            codes[ptr + 4],
            codes[ptr + 5],
            codes[ptr + 6],
            codes[ptr + 7],
        ];
        let low_byte = i32::from_be_bytes(low_byte);
        let high_byte = [
            codes[ptr + 8],
            codes[ptr + 9],
            codes[ptr + 10],
            codes[ptr + 11],
        ];
        let high_byte = i32::from_be_bytes(high_byte);
        let num = high_byte - low_byte + 1;
        ptr += 12;

        // switch-case jump table
        let mut jump_table = Vec::with_capacity(num as usize);
        for pos in 0..num {
            let pos = [codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]];
            let pos = i32::from_be_bytes(pos);
            let jump_pos = pos + origin_bc;
            ptr += 4;
            jump_table.push(jump_pos);
        }
        // default
        jump_table.push(default_byte + origin_bc);

        let top_value = {
            let mut stack = self.frame.area.stack.borrow_mut();
            stack.pop_int()
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
    fn lookup_switch(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let mut bc = pc.load(Ordering::Relaxed) - 1;

        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += (4 - bc % 4);
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;

        let default_byte = [codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]];
        let default_byte = u32::from_be_bytes(default_byte);
        let count = [
            codes[ptr + 4],
            codes[ptr + 5],
            codes[ptr + 6],
            codes[ptr + 7],
        ];
        let count = u32::from_be_bytes(count);
        ptr += 8;

        let mut jump_table: HashMap<u32, u32> = HashMap::new();
        for i in 0..count {
            let value = [codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]];
            let value = u32::from_be_bytes(value);
            let position = [
                codes[ptr + 4],
                codes[ptr + 5],
                codes[ptr + 6],
                codes[ptr + 7],
            ];
            let position = u32::from_be_bytes(position) + origin_bc as u32;
            ptr += 8;
            jump_table.insert(value, position);
        }

        let top_value = {
            let mut stack = self.frame.area.stack.borrow_mut();
            stack.pop_int()
        };
        match jump_table.get(&(top_value as u32)) {
            Some(position) => self.goto_abs_with_occupied(*position as i32, 1),
            None => self.goto_abs_with_occupied(default_byte as i32 + origin_bc, 1),
        }
    }

    #[inline]
    fn ireturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        let v = Oop::new_int(v);
        drop(stack);

        self.set_return(Some(v));
    }

    #[inline]
    fn lreturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_long();
        let v = Oop::new_long(v);
        drop(stack);

        self.set_return(Some(v));
    }

    #[inline]
    fn freturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_float();
        let v = Oop::new_float(v);
        drop(stack);

        self.set_return(Some(v));
    }

    #[inline]
    fn dreturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_double();
        let v = Oop::new_double(v);
        drop(stack);

        self.set_return(Some(v));
    }

    #[inline]
    fn areturn(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        drop(stack);

        self.set_return(Some(v));
    }

    #[inline]
    fn return_void(&self) {
        self.set_return(None);
    }

    #[inline]
    fn get_static(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = read_u2!(pc, codes);
        self.get_field_helper(Oop::Null, cp_idx, true);
    }

    #[inline]
    fn put_static(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = read_u2!(pc, codes);
        self.put_field_helper(cp_idx, true);
    }

    #[inline]
    fn get_field(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = read_u2!(pc, codes);

        let mut stack = self.frame.area.stack.borrow_mut();
        let rf = stack.pop_ref();
        drop(stack);

        match rf {
            Oop::Null => {
                exception::meet_ex(cls_const::J_NPE, None);
            }
            _ => {
                self.get_field_helper(rf, idx, false);
            }
        }
    }

    #[inline]
    fn put_field(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = read_u2!(pc, codes);
        self.put_field_helper(idx, false);
    }

    #[inline]
    fn invoke_virtual(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = read_u2!(pc, codes);
        self.invoke_helper(false, idx, false);
    }

    #[inline]
    fn invoke_special(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = read_u2!(pc, codes);
        self.invoke_helper(false, idx, true);
    }

    #[inline]
    fn invoke_static(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = read_u2!(pc, codes);
        self.invoke_helper(true, idx, true);
    }

    #[inline]
    fn invoke_interface(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = read_u2!(pc, codes);
        let _count = read_u1!(pc, codes);
        let zero = read_u1!(pc, codes);
        if zero != 0 {
            warn!("interpreter: invalid invokeinterface: the value of the fourth operand byte must always be zero.");
        }

        self.invoke_helper(false, cp_idx, false);
    }

    #[inline]
    fn invoke_dynamic(&self) {
        //todo: impl
        unimplemented!()
    }

    #[inline]
    fn new_(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let idx = read_u2!(pc, codes);

        let class = {
            match runtime::require_class2(idx as u16, &self.cp) {
                Some(class) => {
                    oop::class::init_class(&class);
                    oop::class::init_class_fully(&class);

                    class
                }
                None => unreachable!("Cannot get class info from constant pool"),
            }
        };

        let v = oop::Oop::new_inst(class);
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.push_ref(v, false);
    }

    #[inline]
    fn new_array(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let ary_type = read_byte!(pc, codes);
        let mut stack = self.frame.area.stack.borrow_mut();
        let len = stack.pop_int();
        if len < 0 {
            drop(stack);
            exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
        } else {
            let len = len as usize;
            let ary = Oop::new_type_ary(ary_type, len);
            stack.push_ref(ary, false);
        }
    }

    #[inline]
    fn anew_array(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = read_i2!(pc, codes);

        let mut stack = self.frame.area.stack.borrow_mut();
        let length = stack.pop_int();
        drop(stack);

        //        info!("anew_array length={}", length);
        if length < 0 {
            exception::meet_ex(cls_const::J_NASE, Some("length < 0".to_string()));
        } else {
            let class = match runtime::require_class2(cp_idx as u16, &self.cp) {
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
                        let mut v = Vec::with_capacity(class.name.len() + 3);
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
            match runtime::require_class(cl, &name) {
                Some(ary_cls_obj) => {
                    oop::class::init_class(&ary_cls_obj);
                    oop::class::init_class_fully(&ary_cls_obj);

                    let ary = Oop::new_ref_ary(ary_cls_obj, length as usize);
                    let mut stack = self.frame.area.stack.borrow_mut();
                    stack.push_ref(ary, false);
                }
                None => unreachable!(),
            }
        }
    }

    #[inline]
    fn array_length(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();

        match v {
            Oop::Null => {
                drop(stack);
                exception::meet_ex(cls_const::J_NPE, None)
            }
            Oop::Ref(rf) => {
                let v = rf.get_raw_ptr();
                unsafe {
                    match &(*v).v {
                        oop::RefKind::Array(ary) => {
                            let len = ary.elements.len();
                            stack.push_int(len as i32);
                        }
                        oop::RefKind::TypeArray(ary) => {
                            let len = ary.len();
                            stack.push_int(len as i32);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn athrow(&self, jt: JavaThreadRef) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let ex = stack.pop_ref();
        drop(stack);

        jt.write().unwrap().set_ex(ex);
    }

    #[inline]
    fn check_cast(&self) {
        self.check_cast_helper(true);
    }

    #[inline]
    fn instance_of(&self) {
        self.check_cast_helper(false);
    }

    #[inline]
    fn monitor_enter(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        drop(stack);

        match v {
            Oop::Null => {
                exception::meet_ex(cls_const::J_NPE, None);
            }
            Oop::Ref(v) => v.monitor_enter(),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn monitor_exit(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let mut v = stack.pop_ref();
        drop(stack);

        match v {
            Oop::Null => {
                exception::meet_ex(cls_const::J_NPE, None);
            }
            Oop::Ref(v) => v.monitor_exit(),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn wide(&mut self) {
        self.op_widen = true;
    }

    #[inline]
    fn multi_anew_array(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let cp_idx = read_u2!(pc, codes);
        let dimension = read_u1!(pc, codes);

        let mut lens = Vec::new();
        let mut stack = self.frame.area.stack.borrow_mut();
        for _ in 0..dimension {
            let sub = stack.pop_int();
            //todo: check java/lang/NegativeArraySizeException
            lens.push(sub);
        }
        drop(stack);

        let cls = require_class2(cp_idx as u16, &self.cp).unwrap();
        let ary = new_multi_object_array_helper(cls, &lens, 0);

        let mut stack = self.frame.area.stack.borrow_mut();
        stack.push_ref(ary, false);
    }

    #[inline]
    fn if_null(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();

        match v {
            Oop::Null => {
                drop(stack);
                self.goto_by_offset_hardcoded(2)
            }
            _ => {
                let pc = &self.frame.pc;
                let _ = pc.fetch_add(2, Ordering::Relaxed);
            }
        }
    }

    #[inline]
    fn if_non_null(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();

        match v {
            Oop::Null => {
                let pc = &self.frame.pc;
                let _ = pc.fetch_add(2, Ordering::Relaxed);
            }
            _ => {
                drop(stack);
                self.goto_by_offset_hardcoded(2)
            }
        }
    }

    #[inline]
    fn goto_w(&self) {
        let pc = &self.frame.pc;
        let _ = pc.fetch_add(4, Ordering::Relaxed);
        panic!("Use of deprecated instruction goto_w, please check your Java compiler")
    }

    #[inline]
    fn jsr_w(&self) {
        let pc = &self.frame.pc;
        let _ = pc.fetch_add(4, Ordering::Relaxed);
        panic!("Use of deprecated instruction jsr_w, please check your Java compiler")
    }

    #[inline]
    fn other_wise(&self) {
        let pc = &self.frame.pc;
        let codes = &self.frame.code;
        let pc = pc.load(Ordering::Relaxed);
        let pc = pc - 1;
        panic!(
            "Use of undefined bytecode: {} at {}",
            codes[pc as usize], pc
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
