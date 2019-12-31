use crate::oop::Oop;
use std::sync::Arc;

lazy_static! {
    static ref NULL: Arc<Oop> = { Arc::new(Oop::Null) };
    static ref INT0: Arc<Oop> = { Arc::new(Oop::Int(0)) };
    static ref LONG0: Arc<Oop> = { Arc::new(Oop::Long(0)) };
    static ref FLOAT0: Arc<Oop> = { Arc::new(Oop::Float(0.0)) };
    static ref DOUBLE0: Arc<Oop> = { Arc::new(Oop::Double(0.0)) };
}

pub fn get_null() -> Arc<Oop> {
    NULL.clone()
}

pub fn get_int0() -> Arc<Oop> {
    INT0.clone()
}

pub fn get_long0() -> Arc<Oop> {
    LONG0.clone()
}

pub fn get_float0() -> Arc<Oop> {
    FLOAT0.clone()
}

pub fn get_double0() -> Arc<Oop> {
    DOUBLE0.clone()
}

pub fn init() {
    lazy_static::initialize(&NULL);
    lazy_static::initialize(&INT0);
    lazy_static::initialize(&LONG0);
    lazy_static::initialize(&FLOAT0);
    lazy_static::initialize(&DOUBLE0);
}
