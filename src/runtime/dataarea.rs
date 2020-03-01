use crate::oop::Oop;
use crate::runtime::local::Local;
use crate::runtime::stack::Stack;
use std::cell::RefCell;

/*
DataArea的由来

java method执行方式：
每调用一个方法，构造一个新的Frame，并把该frame推入至当前thread.frames栈，
方法执行完毕，出栈该Frame。如果出现异常，jvm_fillInStackTrace遍历当前thread
的frames：提取每个frame的类名，方法名，pc（pc是为了从LineNumberTable Attributes
中定位出错的代码行）并构造异常堆栈。

Frame中的DataArea用RefCell包装，这样就可以让java_call::invoke_java执行Java
方法时，可以用只读方式的frame执行字节码；当有异常时，也可以让jvm_fillInStackTrace
遍历frames获取必要信息。RefCell的性质让这种操作成为可能，在只读的Frame上下文中，需
要修改DataArea时，borrow_mut就可以了。
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
    pub fn new(max_locals: usize, max_stack: usize) -> RefCell<DataArea> {
        let local = Local::new(max_locals);
        let stack = Stack::new(max_stack);

        RefCell::new(DataArea {
            local,
            stack,
            pc: 0,
            return_v: None,
            ex_here: false,
            op_widen: false,
        })
    }
}
