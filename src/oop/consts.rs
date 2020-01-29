use crate::oop::{OopDesc, OopRef};
use std::sync::Arc;

lazy_static! {
    static ref NULL: OopRef = { OopDesc::new_null() };
    static ref INT0: OopRef = { OopDesc::new_int(0) };
    static ref LONG0: OopRef = { OopDesc::new_long(0) };
    static ref FLOAT0: OopRef = { OopDesc::new_float(0.0) };
    static ref DOUBLE0: OopRef = { OopDesc::new_double(0.0) };
}

pub fn get_null() -> OopRef {
    NULL.clone()
}

pub fn get_int0() -> OopRef {
    INT0.clone()
}

pub fn get_long0() -> OopRef {
    LONG0.clone()
}

pub fn get_float0() -> OopRef {
    FLOAT0.clone()
}

pub fn get_double0() -> OopRef {
    DOUBLE0.clone()
}

pub fn init() {
    lazy_static::initialize(&NULL);
    lazy_static::initialize(&INT0);
    lazy_static::initialize(&LONG0);
    lazy_static::initialize(&FLOAT0);
    lazy_static::initialize(&DOUBLE0);
}
