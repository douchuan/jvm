use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Arraylength;

impl Instruction for Arraylength {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::arraylength,
            icp: 0,
            wide: false,
        };

        (info, pc + 1)
    }
}
