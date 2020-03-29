use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lushr;

impl Instruction for Lushr {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lushr,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
