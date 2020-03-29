use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct New;

impl Instruction for New {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::new,
            icp: self.calc_cp_index_u16(codes, pc),
        };

        (info, pc + 3)
    }
}
