use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Daload;

impl Instruction for Daload {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::daload,
            icp: 0,
            wide: false,
        };

        (info, pc + 2)
    }
}
