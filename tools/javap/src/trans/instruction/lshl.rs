use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lshl;

impl Instruction for Lshl {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::lshl,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
