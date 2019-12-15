#![allow(unused)]

mod class_loader;
mod cp_manager;
mod execution;
mod frame;
mod local;
mod stack;
mod sys_dic;
mod thread;

pub use class_loader::{require_class, require_class2, ClassLoader};
pub use cp_manager::{find_class as find_class_in_classpath, ClassPathResult, ClassSource};
pub use execution::instance_of;
pub use frame::Frame;
pub use local::Local;
pub use local::Slot;
pub use stack::Stack;
pub use sys_dic::{find as sys_dic_find, put as sys_dic_put};
pub use thread::{JavaMainThread, JavaThread};

pub fn init() {
    sys_dic::init();
    cp_manager::init();
}