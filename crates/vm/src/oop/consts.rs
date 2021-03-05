use crate::oop::Oop;
use std::sync::Arc;

lazy_static! {
    static ref INT0: Oop = { Oop::new_int(0) };
    static ref LONG0: Oop = { Oop::new_long(0) };
    static ref FLOAT0: Oop = { Oop::new_float(0.0) };
    static ref DOUBLE0: Oop = { Oop::new_double(0.0) };
}

pub fn get_int0() -> Oop {
    INT0.clone()
}

pub fn get_long0() -> Oop {
    LONG0.clone()
}

pub fn get_float0() -> Oop {
    FLOAT0.clone()
}

pub fn get_double0() -> Oop {
    DOUBLE0.clone()
}

pub fn init() {
    lazy_static::initialize(&INT0);
    lazy_static::initialize(&LONG0);
    lazy_static::initialize(&FLOAT0);
    lazy_static::initialize(&DOUBLE0);
}
