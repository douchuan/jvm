use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lookupswitch;

impl Instruction for Lookupswitch {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::lookupswitch.into(),
            code: codes[pc],
            icp: 0,
        };

        unimplemented!()
    }
}
