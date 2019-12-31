use crate::oop::OopDesc;
use std::sync::Arc;

lazy_static! {
    static ref NULL: Arc<OopDesc> = { OopDesc::new_null() };
    static ref INT0: Arc<OopDesc> = { OopDesc::new_int(0) };
    static ref LONG0: Arc<OopDesc> = { OopDesc::new_long(0) };
    static ref FLOAT0: Arc<OopDesc> = { OopDesc::new_float(0.0) };
    static ref DOUBLE0: Arc<OopDesc> = { OopDesc::new_double(0.0) };
}

pub fn get_null() -> Arc<OopDesc> {
    NULL.clone()
}

pub fn get_int0() -> Arc<OopDesc> {
    INT0.clone()
}

pub fn get_long0() -> Arc<OopDesc> {
    LONG0.clone()
}

pub fn get_float0() -> Arc<OopDesc> {
    FLOAT0.clone()
}

pub fn get_double0() -> Arc<OopDesc> {
    DOUBLE0.clone()
}

pub fn init() {
    lazy_static::initialize(&NULL);
    lazy_static::initialize(&INT0);
    lazy_static::initialize(&LONG0);
    lazy_static::initialize(&FLOAT0);
    lazy_static::initialize(&DOUBLE0);
}
