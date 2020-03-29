use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifne;

impl Instruction for Ifne {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ifne,
            icp: 0,
            wide: false,
        };

        (info, pc + 3)
    }
}
