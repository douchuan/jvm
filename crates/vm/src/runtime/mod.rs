#![allow(unused)]

mod class_loader;
mod class_path_manager;
pub mod cmp;
mod constant_pool;
mod consts;
mod dataarea;
pub mod exception;
mod frame;
mod init_vm;
pub mod interp;
pub mod invoke;
mod local;
mod slot;
mod stack;
mod sys_dic;
pub mod thread;
pub mod vm;

pub use class_loader::{require_class, require_class2, require_class3, ClassLoader};

pub use class_path_manager::{
    add_path as add_class_path, add_paths as add_class_paths,
    find_class as find_class_in_classpath, ClassPathResult,
};
pub use constant_pool::ConstantPoolCache;
pub use consts::THREAD_MAX_STACK_FRAMES;
pub use dataarea::DataArea;
pub use frame::Frame;
pub use interp::Interp;
pub use invoke::JavaCall;
pub use slot::Slot;
pub use sys_dic::{find as sys_dic_find, put as sys_dic_put};
pub use thread::JavaThread;

pub fn init() {
    sys_dic::init();
    class_path_manager::init();
}
