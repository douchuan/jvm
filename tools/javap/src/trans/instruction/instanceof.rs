use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Instanceof;

impl Instruction for Instanceof {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::instanceof.into(),
           code: codes[pc],
           icp: self.calc_cp_index_u16(codes, pc)
       };

       (info, pc + 3)
   }
}