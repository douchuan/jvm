use crate::runtime::thread::Threads;
use std::ptr;

static mut VM_GLOBAL: *const u8 = ptr::null();

pub fn get_vm() -> &'static VM {
    unsafe { &*(VM_GLOBAL as *const VM) }
}

pub fn set_vm(vm: &VM) {
    let ptr = vm as *const _ as *const u8;

    unsafe {
        VM_GLOBAL = ptr;
    }
}

pub struct VM {
    pub threads: Threads,
}

impl VM {
    pub fn new(thread_pool_count: usize) -> Box<VM> {
        let vm = Box::new(VM {
            threads: Threads::new(thread_pool_count),
        });

        set_vm(&vm);

        vm
    }
}
