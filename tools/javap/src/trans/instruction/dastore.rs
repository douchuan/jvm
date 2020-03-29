use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dastore;

impl Instruction for Dastore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::dastore,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
