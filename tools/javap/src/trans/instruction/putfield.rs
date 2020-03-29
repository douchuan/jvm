use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Putfield;

impl Instruction for Putfield {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::putfield,
            icp: self.calc_cp_index_u16(codes, pc),
        };

        (info, pc + 3)
    }
}
