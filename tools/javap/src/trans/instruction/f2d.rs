use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct F2D;

impl Instruction for F2D {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::f2d.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}