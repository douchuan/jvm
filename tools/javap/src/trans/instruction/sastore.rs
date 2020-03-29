use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Sastore;

impl Instruction for Sastore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::sastore,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
