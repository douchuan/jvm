use std::sync::atomic::{AtomicI32, Ordering};

pub fn read_byte(pc: &AtomicI32, code: &[u8]) -> u8 {
    let pc = pc.fetch_add(1, Ordering::Relaxed);
    code[pc as usize]
}

pub fn read_i2(pc: &AtomicI32, code: &[u8]) -> i32 {
    let h = read_byte(pc, code) as i16;
    let l = read_byte(pc, code) as i16;
    (h << 8 | l) as i32
}

pub fn read_u1(pc: &AtomicI32, code: &[u8]) -> usize {
    let pc = pc.fetch_add(1, Ordering::Relaxed);
    code[pc as usize] as usize
}

pub fn read_u2(pc: &AtomicI32, code: &[u8]) -> usize {
    read_u1(pc, code) << 8 | read_u1(pc, code)
}
