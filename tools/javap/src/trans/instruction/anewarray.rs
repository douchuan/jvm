use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Anewarray;

impl Instruction for Anewarray {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::anewarray,
            icp: self.calc_cp_index_u16(codes, pc),
        };

        (info, pc + 3)
    }
}
