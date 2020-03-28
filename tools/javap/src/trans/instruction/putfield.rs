use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Putfield;

impl Instruction for Putfield {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::putfield.into(),
           code: codes[pc],
           icp: self.calc_cp_index_u16(codes, pc)
       };

       (info, pc + 3)
   }
}