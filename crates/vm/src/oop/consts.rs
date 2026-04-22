use crate::oop::Oop;
use std::sync::OnceLock;

static INT0: OnceLock<Oop> = OnceLock::new();
static LONG0: OnceLock<Oop> = OnceLock::new();
static FLOAT0: OnceLock<Oop> = OnceLock::new();
static DOUBLE0: OnceLock<Oop> = OnceLock::new();

pub fn get_int0() -> Oop {
    INT0.get().unwrap().clone()
}

pub fn get_long0() -> Oop {
    LONG0.get().unwrap().clone()
}

pub fn get_float0() -> Oop {
    FLOAT0.get().unwrap().clone()
}

pub fn get_double0() -> Oop {
    DOUBLE0.get().unwrap().clone()
}

pub fn init() {
    INT0.get_or_init(|| Oop::new_int(0));
    LONG0.get_or_init(|| Oop::new_long(0));
    FLOAT0.get_or_init(|| Oop::new_float(0.0));
    DOUBLE0.get_or_init(|| Oop::new_double(0.0));
}
