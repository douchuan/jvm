use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iinc {
    pub wide: bool
}

impl Instruction for Iinc {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            name: OpCode::iinc.into(),
            code: codes[pc],
            icp: 0,
        };

        if self.wide {
            (info, pc + 5)
        } else {
            (info, pc + 3)
        }
    }

    fn set_wide(&mut self, wide: bool) {
        self.wide = wide;
    }
}
