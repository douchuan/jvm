use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Tableswitch;

impl Instruction for Tableswitch {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::tableswitch.into(),
           code: codes[pc],
           icp: 0
       };

unimplemented!()   }
}