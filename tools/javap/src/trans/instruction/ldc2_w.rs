use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Ldc2_W;

impl Instruction for Ldc2_W {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::ldc2_w.into(),
           code: codes[pc],
           icp: self.calc_cp_index_u16(codes, pc)
       };

       (info, pc + 3)
   }
}