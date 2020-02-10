#![allow(unused)]

mod class_loader;
pub mod cmp;
mod consts;
mod cp_manager;
mod exception;
mod frame;
mod init_vm;
pub mod java_call;
mod local;
pub mod reflect;
mod slot;
mod stack;
mod sys_dic;
pub mod thread;

pub use class_loader::{require_class, require_class2, require_class3, ClassLoader};

pub use consts::THREAD_MAX_STACK_FRAMES;
pub use cp_manager::{
    add_path as add_class_path, add_paths as add_class_paths,
    find_class as find_class_in_classpath, ClassPathResult,
};
pub use exception::Exception;
pub use frame::Frame;
pub use java_call::JavaCall;
pub use local::Local;
pub use slot::Slot;
pub use stack::Stack;
pub use sys_dic::{find as sys_dic_find, put as sys_dic_put};
pub use thread::JavaThread;

def_sync_ref!(FrameRef, Frame);

pub fn init() {
    sys_dic::init();
    cp_manager::init();
}
