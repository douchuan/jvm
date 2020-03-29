use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Invokevirtual;

impl Instruction for Invokevirtual {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            pc,
            op_code: OpCode::invokevirtual,
            icp: self.calc_cp_index_u16(codes, pc),
        };

        (info, pc + 3)
    }
}
