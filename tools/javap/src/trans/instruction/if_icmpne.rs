use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct If_Icmpne;

impl Instruction for If_Icmpne {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::if_icmpne.into(),
            code: codes[pc],
            icp: 0,
        };

        (info, pc + 3)
    }
}
