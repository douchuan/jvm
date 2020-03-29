use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lshr;

impl Instruction for Lshr {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lshr,
            icp: 0,
        };

        (info, pc + 1)
    }
}
