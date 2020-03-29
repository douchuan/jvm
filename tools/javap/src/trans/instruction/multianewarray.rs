use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Multianewarray;

impl Instruction for Multianewarray {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::multianewarray,
            icp: self.calc_cp_index_u16(codes, pc),
        };

        (info, pc + 4)
    }
}
