use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Tableswitch;

impl Instruction for Tableswitch {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::tableswitch.into(),
            code: codes[pc],
            icp: 0,
        };

        unimplemented!()
    }
}
