use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fcmpl;

impl Instruction for Fcmpl {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::fcmpl,
            icp: 0,
        };

        (info, pc + 1)
    }
}
