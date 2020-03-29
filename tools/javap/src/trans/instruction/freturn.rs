use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Freturn;

impl Instruction for Freturn {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::freturn,
            icp: 0,
        };

        (info, pc + 1)
    }
}
