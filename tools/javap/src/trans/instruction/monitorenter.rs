use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Monitorenter;

impl Instruction for Monitorenter {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::monitorenter.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}