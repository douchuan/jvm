use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Castore;

impl Instruction for Castore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::castore,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
