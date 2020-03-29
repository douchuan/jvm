use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fastore;

impl Instruction for Fastore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fastore,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
