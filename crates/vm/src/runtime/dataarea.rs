use crate::oop::Oop;
use crate::runtime::local::Local;
use crate::runtime::stack::Stack;
use crate::types::DataAreaRef;
use std::sync::{Arc, RwLock};

/*
The origin of DataArea

java method execution method:
Every time a method is called, a new Frame is constructed,
and the frame is pushed to the current thread.frames stack.
After the method is executed, the Frame is popped.
If an exception occurs, jvm_fillInStackTrace traverses the current thread frames:
extract the class name, method name, and pc (pc for LineNumberTable Attributes from each frame)
Locate the error line of code) and construct an exception stack.

The DataArea in the Frame is wrapped with RefCell, so that java_call::invoke_java can execute Java
Method, you can use the read-only frame to execute bytecode; when there is an exception, you can also let
jvm_fillInStackTrace traverse the frames to get the necessary information.
The nature of RefCell makes this possible.
In a read-only Frame context, to modify the DataArea, borrow_mut is fine.
*/
pub struct DataArea {
    pub local: Local,
    pub stack: Stack,
    pub pc: i32,
    pub return_v: Option<Oop>,
    pub ex_here: bool,

    pub op_widen: bool,
}

impl DataArea {
    pub fn new(max_locals: usize, max_stack: usize) -> DataAreaRef {
        let local = Local::new(max_locals);
        let stack = Stack::new(max_stack);

        Arc::new(RwLock::new(DataArea {
            local,
            stack,
            pc: 0,
            return_v: None,
            ex_here: false,
            op_widen: false,
        }))
    }
}
