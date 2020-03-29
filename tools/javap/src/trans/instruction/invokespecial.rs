use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Invokespecial;

impl Instruction for Invokespecial {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::invokespecial,
            icp: self.calc_cp_index_u16(codes, pc),
        };

        (info, pc + 3)
    }
}
