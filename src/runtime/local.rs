
use bytes::{BigEndian, Bytes};

pub enum Slot {
    ConstM1,
    Const0,
    Const1,
    Const2,
    Const3,
    Const4,
    Const5,
    Primitive(Bytes),
    Ref,
    Null,
}

pub struct Local {
    locals: Vec<Slot>,
}

impl Local {
    pub fn new(size: usize) -> Self {
        Self {
            locals: Vec::with_capacity(size),
        }
    }

    pub fn set_int(&mut self, pos: usize, i: i32) {
        let v = i.to_be_bytes();
        self.set(pos, Bytes::from(vec![v[0], v[1], v[2], v[3]]));
    }

    pub fn set_long(&mut self, pos: usize, l: i64) {
        let v = l.to_be_bytes();
        self.set(
            pos,
            Bytes::from(vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]),
        );
    }

    pub fn set_float(&mut self, pos: usize, f: f32) {
        let v = f.to_bits().to_be_bytes();
        self.set(pos, Bytes::from(vec![v[0], v[1], v[2], v[3]]));
    }

    pub fn set_double(&mut self, pos: usize, d: f64) {
        let v = d.to_bits().to_be_bytes();
        self.set(
            pos,
            Bytes::from(vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]),
        );
    }

    pub fn set(&mut self, pos: usize, buf: Bytes) {
        self.locals[pos] = Slot::Primitive(buf);
    }

    pub fn get_int(&self, pos: usize) -> i32 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            i32::from_be_bytes([v[0], v[1], v[2], v[3]])
        } else {
            panic!("Illegal type");
        }
    }

    pub fn get_long(&self, pos: usize) -> i64 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]])
        } else {
            panic!("Illegal type");
        }
    }

    pub fn get_float(&self, pos: usize) -> f32 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            let v = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
            f32::from_bits(v)
        } else {
            panic!("Illegal type");
        }
    }

    pub fn get_double(&self, pos: usize) -> f64 {
        if let Slot::Primitive(v) = self.locals.get(pos).unwrap() {
            let v = u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
            f64::from_bits(v)
        } else {
            panic!("Illegal type");
        }
    }
}