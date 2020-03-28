use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Invokevirtual;

impl Instruction for Invokevirtual {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::invokevirtual.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 3)
   }
}