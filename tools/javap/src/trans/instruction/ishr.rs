use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ishr;

impl Instruction for Ishr {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::ishr,
            icp: 0,
        };

        (info, pc + 1)
    }
}
