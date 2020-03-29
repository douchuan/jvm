use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iastore;

impl Instruction for Iastore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iastore,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
