use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dcmpl;

impl Instruction for Dcmpl {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::dcmpl,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
