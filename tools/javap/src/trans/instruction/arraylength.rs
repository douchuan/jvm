use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Arraylength;

impl Instruction for Arraylength {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::arraylength,
            icp: 0,
        };

        (info, pc + 1)
    }
}
