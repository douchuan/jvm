use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Wide;

impl Instruction for Wide {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::wide.into(),
            code: codes[pc],
            icp: 0,
        };

        unimplemented!()
    }
}
