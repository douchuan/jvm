use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Monitorenter;

impl Instruction for Monitorenter {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::monitorenter,
            icp: 0,
        };

        (info, pc + 1)
    }
}
