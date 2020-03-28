mod aaload;
mod aastore;
mod aconst_null;
mod aload;
mod aload_0;
mod aload_1;
mod aload_2;
mod aload_3;
mod anewarray;
mod areturn;
mod arraylength;
mod astore;
mod astore_0;
mod astore_1;
mod astore_2;
mod astore_3;
mod athrow;
mod baload;
mod bastore;
mod bipush;
mod caload;
mod castore;
mod checkcast;
mod d2f;
mod d2i;
mod d2l;
mod dadd;
mod daload;
mod dastore;
mod dcmpg;
mod dcmpl;
mod dconst_0;
mod dconst_1;
mod ddiv;
mod dload;
mod dload_0;
mod dload_1;
mod dload_2;
mod dload_3;
mod dmul;
mod dneg;
mod drem;
mod dreturn;
mod dstore;
mod dstore_0;
mod dstore_1;
mod dstore_2;
mod dstore_3;
mod dsub;
mod dup;
mod dup2;
mod dup2_x1;
mod dup2_x2;
mod dup_x1;
mod dup_x2;
mod f2d;
mod f2i;
mod f2l;
mod fadd;
mod faload;
mod fastore;
mod fcmpg;
mod fcmpl;
mod fconst_0;
mod fconst_1;
mod fconst_2;
mod fdiv;
mod fload;
mod fload_0;
mod fload_1;
mod fload_2;
mod fload_3;
mod fmul;
mod fneg;
mod frem;
mod freturn;
mod fstore;
mod fstore_0;
mod fstore_1;
mod fstore_2;
mod fstore_3;
mod fsub;
mod getfield;
mod getstatic;
mod goto;
mod goto_w;
mod i2b;
mod i2c;
mod i2d;
mod i2f;
mod i2l;
mod i2s;
mod iadd;
mod iaload;
mod iand;
mod iastore;
mod iconst_0;
mod iconst_1;
mod iconst_2;
mod iconst_3;
mod iconst_4;
mod iconst_5;
mod iconst_m1;
mod idiv;
mod if_acmpeq;
mod if_acmpne;
mod if_icmpeq;
mod if_icmpge;
mod if_icmpgt;
mod if_icmple;
mod if_icmplt;
mod if_icmpne;
mod ifeq;
mod ifge;
mod ifgt;
mod ifle;
mod iflt;
mod ifne;
mod ifnonnull;
mod ifnull;
mod iinc;
mod iload;
mod iload_0;
mod iload_1;
mod iload_2;
mod iload_3;
mod imul;
mod ineg;
mod instanceof;
mod invokedynamic;
mod invokeinterface;
mod invokespecial;
mod invokestatic;
mod invokevirtual;
mod ior;
mod irem;
mod ireturn;
mod ishl;
mod ishr;
mod istore;
mod istore_0;
mod istore_1;
mod istore_2;
mod istore_3;
mod isub;
mod iushr;
mod ixor;
mod jsr;
mod jsr_w;
mod l2d;
mod l2f;
mod l2i;
mod ladd;
mod laload;
mod land;
mod lastore;
mod lcmp;
mod lconst_0;
mod lconst_1;
mod ldc;
mod ldc2_w;
mod ldc_w;
mod ldiv;
mod lload;
mod lload_0;
mod lload_1;
mod lload_2;
mod lload_3;
mod lmul;
mod lneg;
mod lookupswitch;
mod lor;
mod lrem;
mod lreturn;
mod lshl;
mod lshr;
mod lstore;
mod lstore_0;
mod lstore_1;
mod lstore_2;
mod lstore_3;
mod lsub;
mod lushr;
mod lxor;
mod monitorenter;
mod monitorexit;
mod multianewarray;
mod new;
mod newarray;
mod nop;
mod pop;
mod pop2;
mod putfield;
mod putstatic;
mod ret;
mod return_void;
mod saload;
mod sastore;
mod sipush;
mod swap;
mod tableswitch;
mod wide;

use aaload::Aaload;
use aastore::Aastore;
use aconst_null::Aconst_Null;
use aload::Aload;
use aload_0::Aload_0;
use aload_1::Aload_1;
use aload_2::Aload_2;
use aload_3::Aload_3;
use anewarray::Anewarray;
use areturn::Areturn;
use arraylength::Arraylength;
use astore::Astore;
use astore_0::Astore_0;
use astore_1::Astore_1;
use astore_2::Astore_2;
use astore_3::Astore_3;
use athrow::Athrow;
use baload::Baload;
use bastore::Bastore;
use bipush::Bipush;
use caload::Caload;
use castore::Castore;
use checkcast::Checkcast;
use d2f::D2F;
use d2i::D2I;
use d2l::D2L;
use dadd::Dadd;
use daload::Daload;
use dastore::Dastore;
use dcmpg::Dcmpg;
use dcmpl::Dcmpl;
use dconst_0::Dconst_0;
use dconst_1::Dconst_1;
use ddiv::Ddiv;
use dload::Dload;
use dload_0::Dload_0;
use dload_1::Dload_1;
use dload_2::Dload_2;
use dload_3::Dload_3;
use dmul::Dmul;
use dneg::Dneg;
use drem::Drem;
use dreturn::Dreturn;
use dstore::Dstore;
use dstore_0::Dstore_0;
use dstore_1::Dstore_1;
use dstore_2::Dstore_2;
use dstore_3::Dstore_3;
use dsub::Dsub;
use dup::Dup;
use dup2::Dup2;
use dup2_x1::Dup2_X1;
use dup2_x2::Dup2_X2;
use dup_x1::Dup_X1;
use dup_x2::Dup_X2;
use f2d::F2D;
use f2i::F2I;
use f2l::F2L;
use fadd::Fadd;
use faload::Faload;
use fastore::Fastore;
use fcmpg::Fcmpg;
use fcmpl::Fcmpl;
use fconst_0::Fconst_0;
use fconst_1::Fconst_1;
use fconst_2::Fconst_2;
use fdiv::Fdiv;
use fload::Fload;
use fload_0::Fload_0;
use fload_1::Fload_1;
use fload_2::Fload_2;
use fload_3::Fload_3;
use fmul::Fmul;
use fneg::Fneg;
use frem::Frem;
use freturn::Freturn;
use fstore::Fstore;
use fstore_0::Fstore_0;
use fstore_1::Fstore_1;
use fstore_2::Fstore_2;
use fstore_3::Fstore_3;
use fsub::Fsub;
use getfield::Getfield;
use getstatic::Getstatic;
use goto::Goto;
use goto_w::Goto_W;
use i2b::I2B;
use i2c::I2C;
use i2d::I2D;
use i2f::I2F;
use i2l::I2L;
use i2s::I2S;
use iadd::Iadd;
use iaload::Iaload;
use iand::Iand;
use iastore::Iastore;
use iconst_0::Iconst_0;
use iconst_1::Iconst_1;
use iconst_2::Iconst_2;
use iconst_3::Iconst_3;
use iconst_4::Iconst_4;
use iconst_5::Iconst_5;
use iconst_m1::Iconst_M1;
use idiv::Idiv;
use if_acmpeq::If_Acmpeq;
use if_acmpne::If_Acmpne;
use if_icmpeq::If_Icmpeq;
use if_icmpge::If_Icmpge;
use if_icmpgt::If_Icmpgt;
use if_icmple::If_Icmple;
use if_icmplt::If_Icmplt;
use if_icmpne::If_Icmpne;
use ifeq::Ifeq;
use ifge::Ifge;
use ifgt::Ifgt;
use ifle::Ifle;
use iflt::Iflt;
use ifne::Ifne;
use ifnonnull::Ifnonnull;
use ifnull::Ifnull;
use iinc::Iinc;
use iload::Iload;
use iload_0::Iload_0;
use iload_1::Iload_1;
use iload_2::Iload_2;
use iload_3::Iload_3;
use imul::Imul;
use ineg::Ineg;
use instanceof::Instanceof;
use invokedynamic::Invokedynamic;
use invokeinterface::Invokeinterface;
use invokespecial::Invokespecial;
use invokestatic::Invokestatic;
use invokevirtual::Invokevirtual;
use ior::Ior;
use irem::Irem;
use ireturn::Ireturn;
use ishl::Ishl;
use ishr::Ishr;
use istore::Istore;
use istore_0::Istore_0;
use istore_1::Istore_1;
use istore_2::Istore_2;
use istore_3::Istore_3;
use isub::Isub;
use iushr::Iushr;
use ixor::Ixor;
use jsr::Jsr;
use jsr_w::Jsr_W;
use l2d::L2D;
use l2f::L2F;
use l2i::L2I;
use ladd::Ladd;
use laload::Laload;
use land::Land;
use lastore::Lastore;
use lcmp::Lcmp;
use lconst_0::Lconst_0;
use lconst_1::Lconst_1;
use ldc::Ldc;
use ldc2_w::Ldc2_W;
use ldc_w::Ldc_W;
use ldiv::Ldiv;
use lload::Lload;
use lload_0::Lload_0;
use lload_1::Lload_1;
use lload_2::Lload_2;
use lload_3::Lload_3;
use lmul::Lmul;
use lneg::Lneg;
use lookupswitch::Lookupswitch;
use lor::Lor;
use lrem::Lrem;
use lreturn::Lreturn;
use lshl::Lshl;
use lshr::Lshr;
use lstore::Lstore;
use lstore_0::Lstore_0;
use lstore_1::Lstore_1;
use lstore_2::Lstore_2;
use lstore_3::Lstore_3;
use lsub::Lsub;
use lushr::Lushr;
use lxor::Lxor;
use monitorenter::Monitorenter;
use monitorexit::Monitorexit;
use multianewarray::Multianewarray;
use new::New;
use newarray::Newarray;
use nop::Nop;
use pop::Pop;
use pop2::Pop2;
use putfield::Putfield;
use putstatic::Putstatic;
use ret::Ret;
use return_void::Return_Void;
use saload::Saload;
use sastore::Sastore;
use sipush::Sipush;
use swap::Swap;
use tableswitch::Tableswitch;
use wide::Wide;

pub struct InstructionInfo {
    pub name: &'static str,
    pub code: u8,
    pub icp: usize,
}

pub trait Instruction {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize);
    fn calc_cp_index_u16(&self, codes: &[u8], pc: usize) -> usize {
        let indexbyte1 = codes[pc + 1] as i16;
        let indexbyte2 = codes[pc + 2] as i16;
        ((indexbyte1 << 8 | indexbyte2) as i32) as usize
    }
    fn set_wide(&mut self, _wide: bool) {
        unimplemented!()
    }
}

pub fn get_instructions() -> Vec<Box<dyn Instruction>> {
    vec![
        Box::new(Nop),
        Box::new(Aconst_Null),
        Box::new(Iconst_M1),
        Box::new(Iconst_0),
        Box::new(Iconst_1),
        Box::new(Iconst_2),
        Box::new(Iconst_3),
        Box::new(Iconst_4),
        Box::new(Iconst_5),
        Box::new(Lconst_0),
        Box::new(Lconst_1),
        Box::new(Fconst_0),
        Box::new(Fconst_1),
        Box::new(Fconst_2),
        Box::new(Dconst_0),
        Box::new(Dconst_1),
        Box::new(Bipush),
        Box::new(Sipush),
        Box::new(Ldc),
        Box::new(Ldc_W),
        Box::new(Ldc2_W),
        Box::new(Iload { wide: false }),
        Box::new(Lload { wide: false }),
        Box::new(Fload { wide: false }),
        Box::new(Dload { wide: false }),
        Box::new(Aload { wide: false }),
        Box::new(Iload_0),
        Box::new(Iload_1),
        Box::new(Iload_2),
        Box::new(Iload_3),
        Box::new(Lload_0),
        Box::new(Lload_1),
        Box::new(Lload_2),
        Box::new(Lload_3),
        Box::new(Fload_0),
        Box::new(Fload_1),
        Box::new(Fload_2),
        Box::new(Fload_3),
        Box::new(Dload_0),
        Box::new(Dload_1),
        Box::new(Dload_2),
        Box::new(Dload_3),
        Box::new(Aload_0),
        Box::new(Aload_1),
        Box::new(Aload_2),
        Box::new(Aload_3),
        Box::new(Iaload),
        Box::new(Laload),
        Box::new(Faload),
        Box::new(Daload),
        Box::new(Aaload),
        Box::new(Baload),
        Box::new(Caload),
        Box::new(Saload),
        Box::new(Istore { wide: false }),
        Box::new(Lstore { wide: false }),
        Box::new(Fstore { wide: false }),
        Box::new(Dstore { wide: false }),
        Box::new(Astore { wide: false }),
        Box::new(Istore_0),
        Box::new(Istore_1),
        Box::new(Istore_2),
        Box::new(Istore_3),
        Box::new(Lstore_0),
        Box::new(Lstore_1),
        Box::new(Lstore_2),
        Box::new(Lstore_3),
        Box::new(Fstore_0),
        Box::new(Fstore_1),
        Box::new(Fstore_2),
        Box::new(Fstore_3),
        Box::new(Dstore_0),
        Box::new(Dstore_1),
        Box::new(Dstore_2),
        Box::new(Dstore_3),
        Box::new(Astore_0),
        Box::new(Astore_1),
        Box::new(Astore_2),
        Box::new(Astore_3),
        Box::new(Iastore),
        Box::new(Lastore),
        Box::new(Fastore),
        Box::new(Dastore),
        Box::new(Aastore),
        Box::new(Bastore),
        Box::new(Castore),
        Box::new(Sastore),
        Box::new(Pop),
        Box::new(Pop2),
        Box::new(Dup),
        Box::new(Dup_X1),
        Box::new(Dup_X2),
        Box::new(Dup2),
        Box::new(Dup2_X1),
        Box::new(Dup2_X2),
        Box::new(Swap),
        Box::new(Iadd),
        Box::new(Ladd),
        Box::new(Fadd),
        Box::new(Dadd),
        Box::new(Isub),
        Box::new(Lsub),
        Box::new(Fsub),
        Box::new(Dsub),
        Box::new(Imul),
        Box::new(Lmul),
        Box::new(Fmul),
        Box::new(Dmul),
        Box::new(Idiv),
        Box::new(Ldiv),
        Box::new(Fdiv),
        Box::new(Ddiv),
        Box::new(Irem),
        Box::new(Lrem),
        Box::new(Frem),
        Box::new(Drem),
        Box::new(Ineg),
        Box::new(Lneg),
        Box::new(Fneg),
        Box::new(Dneg),
        Box::new(Ishl),
        Box::new(Lshl),
        Box::new(Ishr),
        Box::new(Lshr),
        Box::new(Iushr),
        Box::new(Lushr),
        Box::new(Iand),
        Box::new(Land),
        Box::new(Ior),
        Box::new(Lor),
        Box::new(Ixor),
        Box::new(Lxor),
        Box::new(Iinc { wide: false }),
        Box::new(I2L),
        Box::new(I2F),
        Box::new(I2D),
        Box::new(L2I),
        Box::new(L2F),
        Box::new(L2D),
        Box::new(F2I),
        Box::new(F2L),
        Box::new(F2D),
        Box::new(D2I),
        Box::new(D2L),
        Box::new(D2F),
        Box::new(I2B),
        Box::new(I2C),
        Box::new(I2S),
        Box::new(Lcmp),
        Box::new(Fcmpl),
        Box::new(Fcmpg),
        Box::new(Dcmpl),
        Box::new(Dcmpg),
        Box::new(Ifeq),
        Box::new(Ifne),
        Box::new(Iflt),
        Box::new(Ifge),
        Box::new(Ifgt),
        Box::new(Ifle),
        Box::new(If_Icmpeq),
        Box::new(If_Icmpne),
        Box::new(If_Icmplt),
        Box::new(If_Icmpge),
        Box::new(If_Icmpgt),
        Box::new(If_Icmple),
        Box::new(If_Acmpeq),
        Box::new(If_Acmpne),
        Box::new(Goto),
        Box::new(Jsr),
        Box::new(Ret { wide: false }),
        Box::new(Tableswitch),
        Box::new(Lookupswitch),
        Box::new(Ireturn),
        Box::new(Lreturn),
        Box::new(Freturn),
        Box::new(Dreturn),
        Box::new(Areturn),
        Box::new(Return_Void),
        Box::new(Getstatic),
        Box::new(Putstatic),
        Box::new(Getfield),
        Box::new(Putfield),
        Box::new(Invokevirtual),
        Box::new(Invokespecial),
        Box::new(Invokestatic),
        Box::new(Invokeinterface),
        Box::new(Invokedynamic),
        Box::new(New),
        Box::new(Newarray),
        Box::new(Anewarray),
        Box::new(Arraylength),
        Box::new(Athrow),
        Box::new(Checkcast),
        Box::new(Instanceof),
        Box::new(Monitorenter),
        Box::new(Monitorexit),
        Box::new(Wide),
        Box::new(Multianewarray),
        Box::new(Ifnull),
        Box::new(Ifnonnull),
        Box::new(Goto_W),
        Box::new(Jsr_W),
    ]
}
