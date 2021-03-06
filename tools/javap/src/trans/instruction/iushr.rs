use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iushr;

impl Instruction for Iushr {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::iushr,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
