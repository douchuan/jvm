use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Daload;

impl Instruction for Daload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::daload,
            icp: 0,
        };

        (info, pc + 2)
    }
}
