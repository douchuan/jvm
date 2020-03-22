use jni_sys::{jint, jsize, JNIInvokeInterface_, JavaVM};
use lazy_static::lazy_static;
use libc::c_void;
use std::cell::RefCell;
use std::sync::Mutex;

// TODO: We can't safely panic over FFI boundaries, so we need another panic handler

//https://docs.oracle.com/javase/7/docs/technotes/guides/jni/spec/invocation.html
#[no_mangle]
extern "C" fn JNI_GetDefaultJavaVMInitArgs(args: *mut c_void) -> jint {
	0
}

#[derive(Clone)]
struct VMHolder {
	jvm: Box<JNIInvokeInterface_>,
}
unsafe impl Send for VMHolder {}
unsafe impl Sync for VMHolder {}

impl VMHolder {
	fn inner(&self) -> JavaVM {
		self.jvm.as_ref() as *const _
	}
}

// TODO: Hotspot doesn't supports creation of multiple JVMs per process, should we?
lazy_static! {
	static ref JVM: Mutex<Option<VMHolder>> = Mutex::new(None);
}

unsafe extern "system" fn DestroyJavaVM(_vm: *mut JavaVM) -> jint {
	todo!();
}
unsafe extern "system" fn AttachCurrentThread(
	_vm: *mut JavaVM,
	_penv: *mut *mut c_void,
	_args: *mut c_void,
) -> jint {
	todo!();
}
unsafe extern "system" fn DetachCurrentThread(_vm: *mut JavaVM) -> jint {
	todo!();
}
unsafe extern "system" fn GetEnv(_vm: *mut JavaVM, _penv: *mut *mut c_void, _version: jint) -> jint {
	todo!();
}
unsafe extern "system" fn AttachCurrentThreadAsDaemon(
	_vm: *mut JavaVM,
	_penv: *mut *mut c_void,
	_args: *mut c_void,
) -> jint {
	todo!();
}

#[no_mangle]
extern "C" fn JNI_CreateJavaVM(
	pvm: *mut *mut JavaVM,
	penv: *mut *mut c_void,
	_args: *mut c_void,
) -> jint {
	let mut lock = JVM.lock().expect("jvm lock");
	// Can't have multiple VMs per process
	if lock.is_some() {
		-1
	} else {
		use std::ptr::null_mut;
		let holder = VMHolder {
			jvm: Box::new(JNIInvokeInterface_ {
				// We can use reserved fields for implementation details
				reserved0: null_mut(),
				reserved1: null_mut(),
				reserved2: null_mut(),
				DestroyJavaVM: Some(DestroyJavaVM),
				AttachCurrentThread: Some(AttachCurrentThread),
				DetachCurrentThread: Some(DetachCurrentThread),
				GetEnv: Some(GetEnv),
				AttachCurrentThreadAsDaemon: Some(AttachCurrentThreadAsDaemon),
			}),
		};
		unsafe {
			**pvm = holder.inner();
			holder.jvm.GetEnv.unwrap()(&mut holder.inner(), penv, 0);
		}
		lock.replace(holder);
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
		if let Some(ref holder) = lock.as_ref() {
			unsafe {
				**vm_buf = holder.jvm.as_ref();
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
