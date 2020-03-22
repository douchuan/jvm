use jni_sys::{jint, jsize, JavaVM};
use lazy_static::lazy_static;
use libc::c_void;
use std::cell::RefCell;
use std::sync::Mutex;

//https://docs.oracle.com/javase/7/docs/technotes/guides/jni/spec/invocation.html
#[no_mangle]
extern "C" fn JNI_GetDefaultJavaVMInitArgs(args: *mut c_void) -> jint {
	0
}

#[derive(Clone, Copy)]
struct VMHolder {
	jvm: *mut JavaVM,
}
unsafe impl Send for VMHolder {}
unsafe impl Sync for VMHolder {}

// TODO: Hotspot doesn't supports creation of multiple JVMs per process, should we?
lazy_static! {
	static ref JVM: Mutex<Option<VMHolder>> = Mutex::new(None);
}

#[no_mangle]
extern "C" fn JNI_CreateJavaVM(
	pvm: *mut *mut JavaVM,
	penv: *mut *mut c_void,
	args: *mut c_void,
) -> jint {
	let lock = JVM.lock().expect("jvm lock");
	// Can't have multiple VMs per process
	if lock.is_some() {
		-1
	} else {
		0
	}
}

#[no_mangle]
extern "C" fn JNI_GetCreatedJavaVMs(
	vm_buf: *mut *mut JavaVM,
	buf_len: jsize,
	n_vms: *mut jsize,
) -> jint {
	if buf_len >= 1 {
		let lock = JVM.lock().expect("jvm lock");
		if let Some(holder) = *lock {
			unsafe {
				*vm_buf = &mut *holder.jvm;
				*n_vms = 1;
			}
		} else {
			unsafe {
				*n_vms = 0;
			}
		}
		1
	} else {
		unsafe {
			*n_vms = 0;
		};
		0
	}
}
