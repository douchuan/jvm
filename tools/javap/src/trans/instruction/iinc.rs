use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iinc {
    pub wide: bool,
}

impl Instruction for Iinc {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let mut info = InstructionInfo {
            pc,
            op_code: OpCode::iinc,
            icp: 0,
            wide: false,
        };

        if self.wide {
            info.wide = self.wide;
            (info, pc + 5)
        } else {
            (info, pc + 3)
        }
    }

    fn set_wide(&mut self, wide: bool) {
        self.wide = wide;
    }
}
