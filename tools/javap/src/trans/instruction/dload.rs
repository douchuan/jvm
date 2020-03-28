use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Dload {
    pub wide: bool,
}

impl Instruction for Dload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::dload.into(),
            code: codes[pc],
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
