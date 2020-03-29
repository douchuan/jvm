use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ret {
    pub wide: bool,
}

impl Instruction for Ret {
    fn run(&self, _codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let mut info = InstructionInfo {
            pc,
            op_code: OpCode::ret,
            icp: 0,
            wide: false,
        };

        if self.wide {
            info.wide = self.wide;
            (info, pc + 3)
        } else {
            (info, pc + 2)
        }
    }

    fn set_wide(&mut self, wide: bool) {
        self.wide = wide;
    }
}
