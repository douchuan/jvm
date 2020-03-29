use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fstore {
    pub wide: bool,
}

impl Instruction for Fstore {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::fstore,
            icp: 0,
        };

        if self.wide {
            (info, pc + 3)
        } else {
            (info, pc + 2)
        }
    }

    fn set_wide(&mut self, wide: bool) {
        self.wide = wide;
    }
}
