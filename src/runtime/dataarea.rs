use crate::runtime::local::Local;
use crate::runtime::stack::Stack;
use crate::oop::Oop;
use std::cell::RefCell;

/*
Frame的可变部分放在这里
*/
pub struct DataArea {
    pub local: Local,
    pub stack: Stack,
    pub pc: i32,
    pub return_v: Option<Oop>,

    pub op_widen: bool,
}

impl DataArea {
    pub fn new(max_locals: usize, max_stack: usize) -> RefCell<DataArea> {
        let local = Local::new(max_locals);
        let stack = Stack::new(max_stack);

        RefCell::new(DataArea {
            local,
            stack,
            pc: 0,
            return_v: None,
            op_widen: false,
        })
    }
}