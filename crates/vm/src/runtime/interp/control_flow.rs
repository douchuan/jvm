use super::Interp;
use crate::oop::Oop;
use std::sync::atomic::Ordering;

impl<'a> Interp<'a> {
    pub fn goto_abs(&self, pc: i32) {
        self.frame.pc.store(pc, Ordering::Relaxed);
    }

    pub fn goto_by_offset(&self, branch: i32) {
        let _ = self.frame.pc.fetch_add(branch, Ordering::Relaxed);
    }

    pub fn goto_by_offset_with_occupied(&self, branch: i32, occupied: i32) {
        self.goto_by_offset(branch);
        self.goto_by_offset(-(occupied - 1));
    }

    pub fn goto_by_offset_hardcoded(&self, occupied: i32) {
        let codes = &self.code;
        let pc = self.frame.pc.load(Ordering::Relaxed);
        let high = codes[pc as usize] as i16;
        let low = codes[(pc + 1) as usize] as i16;
        let branch = (high << 8) | low;
        self.goto_by_offset_with_occupied(branch as i32, occupied);
    }

    pub fn goto_abs_with_occupied(&self, pc: i32, occupied: i32) {
        self.goto_abs(pc);
        self.goto_by_offset(-(occupied - 1));
    }

    #[inline]
    pub fn ifeq(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        if v == 0 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn ifne(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        if v != 0 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn iflt(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        if v < 0 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn ifge(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        if v >= 0 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn ifgt(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        if v > 0 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn ifle(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_int();
        if v <= 0 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_icmpeq(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v1 == v2 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_icmpne(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v1 != v2 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_icmplt(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v1 < v2 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_icmpge(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v1 >= v2 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_icmpgt(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v1 > v2 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_icmple(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_int();
        let v1 = stack.pop_int();
        if v1 <= v2 {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_acmpeq(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_ref();
        let v1 = stack.pop_ref();
        if crate::oop::Oop::is_eq(v1.extract_ref(), v2.extract_ref()) {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn if_acmpne(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v2 = stack.pop_ref();
        let v1 = stack.pop_ref();
        if !crate::oop::Oop::is_eq(v1.extract_ref(), v2.extract_ref()) {
            drop(stack);
            self.goto_by_offset_hardcoded(2);
        } else {
            let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        }
    }
    #[inline]
    pub fn goto(&self) {
        self.goto_by_offset_hardcoded(2);
    }
    #[inline]
    pub fn jsr(&self) {
        let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
        panic!("Use of deprecated instruction jsr, please check your Java compiler");
    }
    pub fn ret(&mut self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let op_widen = self.op_widen;
        let slot_index = if op_widen {
            self.op_widen = false;
            super::read::read_u2(pc, codes)
        } else {
            super::read::read_u1(pc, codes)
        };
        let new_pc = self.local.borrow().get_int(slot_index) as i32;
        pc.store(new_pc, Ordering::Relaxed);
    }
    pub fn table_switch(&self) {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let mut bc = pc.load(Ordering::Relaxed) - 1;
        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += 4 - bc % 4;
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;
        let default_byte =
            i32::from_be_bytes([codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]]);
        let low_byte = i32::from_be_bytes([
            codes[ptr + 4],
            codes[ptr + 5],
            codes[ptr + 6],
            codes[ptr + 7],
        ]);
        let high_byte = i32::from_be_bytes([
            codes[ptr + 8],
            codes[ptr + 9],
            codes[ptr + 10],
            codes[ptr + 11],
        ]);
        let num = high_byte - low_byte + 1;
        ptr += 12;
        let mut jump_table = Vec::with_capacity(num as usize);
        for _ in 0..num {
            let pos =
                i32::from_be_bytes([codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]]);
            jump_table.push(pos + origin_bc);
            ptr += 4;
        }
        jump_table.push(default_byte + origin_bc);
        let top_value = { self.frame.area.stack.borrow_mut().pop_int() };
        if top_value > (jump_table.len() as i32 - 1 + low_byte) || top_value < low_byte {
            self.goto_abs_with_occupied(*jump_table.last().unwrap() as i32, 1);
        } else {
            self.goto_abs_with_occupied(jump_table[(top_value - low_byte) as usize] as i32, 1);
        }
    }
    pub fn lookup_switch(&self) {
        use std::collections::HashMap;
        let pc = &self.frame.pc;
        let codes = &self.code;
        let mut bc = pc.load(Ordering::Relaxed) - 1;
        let origin_bc = bc;
        if bc % 4 != 0 {
            bc += 4 - bc % 4;
        } else {
            bc += 4;
        }
        let mut ptr = bc as usize;
        let default_byte =
            u32::from_be_bytes([codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]]);
        let count = u32::from_be_bytes([
            codes[ptr + 4],
            codes[ptr + 5],
            codes[ptr + 6],
            codes[ptr + 7],
        ]);
        ptr += 8;
        let mut jump_table: HashMap<u32, u32> = HashMap::new();
        for _ in 0..count {
            let value =
                u32::from_be_bytes([codes[ptr], codes[ptr + 1], codes[ptr + 2], codes[ptr + 3]]);
            let position = u32::from_be_bytes([
                codes[ptr + 4],
                codes[ptr + 5],
                codes[ptr + 6],
                codes[ptr + 7],
            ]) + origin_bc as u32;
            ptr += 8;
            jump_table.insert(value, position);
        }
        let top_value = { self.frame.area.stack.borrow_mut().pop_int() };
        match jump_table.get(&(top_value as u32)) {
            Some(position) => self.goto_abs_with_occupied(*position as i32, 1),
            None => self.goto_abs_with_occupied(default_byte as i32 + origin_bc, 1),
        }
    }
    #[inline]
    pub fn if_null(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        match v {
            crate::oop::Oop::Null => {
                drop(stack);
                self.goto_by_offset_hardcoded(2);
            }
            _ => {
                let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
            }
        }
    }
    #[inline]
    pub fn if_non_null(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        let v = stack.pop_ref();
        match v {
            crate::oop::Oop::Null => {
                let _ = self.frame.pc.fetch_add(2, Ordering::Relaxed);
            }
            _ => {
                drop(stack);
                self.goto_by_offset_hardcoded(2);
            }
        }
    }
    #[inline]
    pub fn goto_w(&self) {
        let _ = self.frame.pc.fetch_add(4, Ordering::Relaxed);
        panic!("Use of deprecated instruction goto_w, please check your Java compiler");
    }
    #[inline]
    pub fn jsr_w(&self) {
        let _ = self.frame.pc.fetch_add(4, Ordering::Relaxed);
        panic!("Use of deprecated instruction jsr_w, please check your Java compiler");
    }
}
