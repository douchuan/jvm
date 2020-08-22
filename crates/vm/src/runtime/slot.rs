use crate::oop::Oop;

#[derive(Debug, Clone)]
pub enum Slot {
    ConstM1,
    Const0,
    Const1,
    Const2,
    Const3,
    Const4,
    Const5,
    I32(i32),
    F32(f32),
    F64(f64),
    I64(i64),
    Ref(Oop),
    Nop, //for Stack long, double
}
