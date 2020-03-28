use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dload_1;

impl Instruction for Dload_1 {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dload_1.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
