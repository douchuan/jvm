use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Tableswitch;

impl Instruction for Tableswitch {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::tableswitch,
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
        let _default_byte = i32::from_be_bytes(default_byte);
        let low_byte = [
            codes[ptr + 4],
            codes[ptr + 5],
            codes[ptr + 6],
            codes[ptr + 7],
        ];
        let low_byte = i32::from_be_bytes(low_byte);
        let high_byte = [
            codes[ptr + 8],
            codes[ptr + 9],
            codes[ptr + 10],
            codes[ptr + 11],
        ];
        let high_byte = i32::from_be_bytes(high_byte);
        let num = high_byte - low_byte + 1;
        ptr += 12;
        ptr += (4 * num) as usize;

        (info, ptr)
    }
}
