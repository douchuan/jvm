use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Instanceof;

impl Instruction for Instanceof {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::instanceof,
            icp: self.calc_cp_index_u16(codes, pc),
        };

        (info, pc + 3)
    }
}
