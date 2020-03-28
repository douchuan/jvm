use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Invokestatic;

impl Instruction for Invokestatic {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::invokestatic.into(),
           code: codes[pc],
           icp: self.calc_cp_index_u16(codes, pc)
       };

       (info, pc + 3)
   }
}