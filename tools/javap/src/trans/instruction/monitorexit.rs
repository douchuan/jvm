use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Monitorexit;

impl Instruction for Monitorexit {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::monitorexit,
            icp: 0,
        };

        (info, pc + 1)
    }
}
