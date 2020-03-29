use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ishr;

impl Instruction for Ishr {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ishr,
            icp: 0,
        };

        (info, pc + 1)
    }
}
