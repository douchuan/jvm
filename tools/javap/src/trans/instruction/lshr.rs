use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lshr;

impl Instruction for Lshr {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lshr,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
