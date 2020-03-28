use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dcmpl;

impl Instruction for Dcmpl {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dcmpl.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 1)
    }
}
