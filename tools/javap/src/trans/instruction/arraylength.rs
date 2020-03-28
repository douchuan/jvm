use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Arraylength;

impl Instruction for Arraylength {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::arraylength.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
