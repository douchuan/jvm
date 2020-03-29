use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lastore;

impl Instruction for Lastore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lastore,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
