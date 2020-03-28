use classfile::OpCode;
use super::{Instruction, InstructionInfo};

pub struct Lookupswitch;

impl Instruction for Lookupswitch {
   fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
       let info = InstructionInfo {
           name: OpCode::lookupswitch.into(),
           code: codes[pc],
           icp: 0
       };

        unimplemented!()
   }
}