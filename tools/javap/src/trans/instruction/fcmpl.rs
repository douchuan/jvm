use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fcmpl;

impl Instruction for Fcmpl {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::fcmpl.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
