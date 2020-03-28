use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Fcmpl;

impl Instruction for Fcmpl {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::fcmpl.into(),
           code: codes[pc],
           icp: 0
       };

       (info, pc + 1)
   }
}