use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lshl;

impl Instruction for Lshl {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lshl,
            icp: 0,
        };

        (info, pc + 1)
    }
}
