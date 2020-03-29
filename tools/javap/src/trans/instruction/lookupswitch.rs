use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Lookupswitch;

impl Instruction for Lookupswitch {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::lookupswitch,
            icp: 0,
        };

        let mut bc = pc;
        if bc % 4 != 0 {
            bc += 4 - bc % 4;
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;

        let default_byte = [codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]];
        let _default_byte = u32::from_be_bytes(default_byte);
        let count = [
            codes[ptr + 4],
            codes[ptr + 5],
            codes[ptr + 6],
            codes[ptr + 7],
        ];
        let count = u32::from_be_bytes(count);
        ptr += 8;
        ptr += (8 * count) as usize;

        (info, ptr)
    }
}
