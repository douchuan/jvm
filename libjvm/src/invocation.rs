#![allow(non_snake_case)]
#![allow(unused_imports)]

//https://docs.oracle.com/javase/7/docs/technotes/guides/jni/spec/invocation.html

use jni_sys::{jboolean, jint, jsize, JNIInvokeInterface_, JNINativeInterface_, JavaVM};
use lazy_static::lazy_static;
use libc::c_void;
use std::cell::RefCell;
use std::sync::Mutex;
use vm::runtime::thread::JavaMainThread;

use crate::native;

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
unsafe extern "system" fn GetEnv(
	_vm: *mut JavaVM,
	penv: *mut *mut core::ffi::c_void,
	_version: jint,
) -> jint {
	use std::ptr::null_mut;
	*penv = Box::into_raw(Box::new(JNINativeInterface_ {
		reserved0: null_mut(),
		reserved1: null_mut(),
		reserved2: null_mut(),
		reserved3: null_mut(),
		GetVersion: Some(native::GetVersion),
		DefineClass: Some(native::DefineClass),
		FindClass: Some(native::FindClass),
		FromReflectedMethod: Some(native::FromReflectedMethod),
		FromReflectedField: Some(native::FromReflectedField),
		ToReflectedMethod: Some(native::ToReflectedMethod),
		GetSuperclass: Some(native::GetSuperclass),
		IsAssignableFrom: Some(native::IsAssignableFrom),
		ToReflectedField: Some(native::ToReflectedField),
		Throw: Some(native::Throw),
		ThrowNew: Some(native::ThrowNew),
		ExceptionOccurred: Some(native::ExceptionOccurred),
		ExceptionDescribe: Some(native::ExceptionDescribe),
		ExceptionClear: Some(native::ExceptionClear),
		FatalError: Some(native::FatalError),
		PushLocalFrame: Some(native::PushLocalFrame),
		PopLocalFrame: Some(native::PopLocalFrame),
		NewGlobalRef: Some(native::NewGlobalRef),
		DeleteGlobalRef: Some(native::DeleteGlobalRef),
		DeleteLocalRef: Some(native::DeleteLocalRef),
		IsSameObject: Some(native::IsSameObject),
		NewLocalRef: Some(native::NewLocalRef),
		EnsureLocalCapacity: Some(native::EnsureLocalCapacity),
		AllocObject: Some(native::AllocObject),
		NewObject: Some(native::NewObject),
		NewObjectV: Some(native::NewObjectV),
		NewObjectA: Some(native::NewObjectA),
		GetObjectClass: Some(native::GetObjectClass),
		IsInstanceOf: Some(native::IsInstanceOf),
		GetMethodID: Some(native::GetMethodID),
		CallObjectMethod: Some(native::CallObjectMethod),
		CallObjectMethodV: Some(native::CallObjectMethodV),
		CallObjectMethodA: Some(native::CallObjectMethodA),
		CallBooleanMethod: Some(native::CallBooleanMethod),
		CallBooleanMethodV: Some(native::CallBooleanMethodV),
		CallBooleanMethodA: Some(native::CallBooleanMethodA),
		CallByteMethod: Some(native::CallByteMethod),
		CallByteMethodV: Some(native::CallByteMethodV),
		CallByteMethodA: Some(native::CallByteMethodA),
		CallCharMethod: Some(native::CallCharMethod),
		CallCharMethodV: Some(native::CallCharMethodV),
		CallCharMethodA: Some(native::CallCharMethodA),
		CallShortMethod: Some(native::CallShortMethod),
		CallShortMethodV: Some(native::CallShortMethodV),
		CallShortMethodA: Some(native::CallShortMethodA),
		CallIntMethod: Some(native::CallIntMethod),
		CallIntMethodV: Some(native::CallIntMethodV),
		CallIntMethodA: Some(native::CallIntMethodA),
		CallLongMethod: Some(native::CallLongMethod),
		CallLongMethodV: Some(native::CallLongMethodV),
		CallLongMethodA: Some(native::CallLongMethodA),
		CallFloatMethod: Some(native::CallFloatMethod),
		CallFloatMethodV: Some(native::CallFloatMethodV),
		CallFloatMethodA: Some(native::CallFloatMethodA),
		CallDoubleMethod: Some(native::CallDoubleMethod),
		CallDoubleMethodV: Some(native::CallDoubleMethodV),
		CallDoubleMethodA: Some(native::CallDoubleMethodA),
		CallVoidMethod: Some(native::CallVoidMethod),
		CallVoidMethodV: Some(native::CallVoidMethodV),
		CallVoidMethodA: Some(native::CallVoidMethodA),
		CallNonvirtualObjectMethod: Some(native::CallNonvirtualObjectMethod),
		CallNonvirtualObjectMethodV: Some(native::CallNonvirtualObjectMethodV),
		CallNonvirtualObjectMethodA: Some(native::CallNonvirtualObjectMethodA),
		CallNonvirtualBooleanMethod: Some(native::CallNonvirtualBooleanMethod),
		CallNonvirtualBooleanMethodV: Some(native::CallNonvirtualBooleanMethodV),
		CallNonvirtualBooleanMethodA: Some(native::CallNonvirtualBooleanMethodA),
		CallNonvirtualByteMethod: Some(native::CallNonvirtualByteMethod),
		CallNonvirtualByteMethodV: Some(native::CallNonvirtualByteMethodV),
		CallNonvirtualByteMethodA: Some(native::CallNonvirtualByteMethodA),
		CallNonvirtualCharMethod: Some(native::CallNonvirtualCharMethod),
		CallNonvirtualCharMethodV: Some(native::CallNonvirtualCharMethodV),
		CallNonvirtualCharMethodA: Some(native::CallNonvirtualCharMethodA),
		CallNonvirtualShortMethod: Some(native::CallNonvirtualShortMethod),
		CallNonvirtualShortMethodV: Some(native::CallNonvirtualShortMethodV),
		CallNonvirtualShortMethodA: Some(native::CallNonvirtualShortMethodA),
		CallNonvirtualIntMethod: Some(native::CallNonvirtualIntMethod),
		CallNonvirtualIntMethodV: Some(native::CallNonvirtualIntMethodV),
		CallNonvirtualIntMethodA: Some(native::CallNonvirtualIntMethodA),
		CallNonvirtualLongMethod: Some(native::CallNonvirtualLongMethod),
		CallNonvirtualLongMethodV: Some(native::CallNonvirtualLongMethodV),
		CallNonvirtualLongMethodA: Some(native::CallNonvirtualLongMethodA),
		CallNonvirtualFloatMethod: Some(native::CallNonvirtualFloatMethod),
		CallNonvirtualFloatMethodV: Some(native::CallNonvirtualFloatMethodV),
		CallNonvirtualFloatMethodA: Some(native::CallNonvirtualFloatMethodA),
		CallNonvirtualDoubleMethod: Some(native::CallNonvirtualDoubleMethod),
		CallNonvirtualDoubleMethodV: Some(native::CallNonvirtualDoubleMethodV),
		CallNonvirtualDoubleMethodA: Some(native::CallNonvirtualDoubleMethodA),
		CallNonvirtualVoidMethod: Some(native::CallNonvirtualVoidMethod),
		CallNonvirtualVoidMethodV: Some(native::CallNonvirtualVoidMethodV),
		CallNonvirtualVoidMethodA: Some(native::CallNonvirtualVoidMethodA),
		GetFieldID: Some(native::GetFieldID),
		GetObjectField: Some(native::GetObjectField),
		GetBooleanField: Some(native::GetBooleanField),
		GetByteField: Some(native::GetByteField),
		GetCharField: Some(native::GetCharField),
		GetShortField: Some(native::GetShortField),
		GetIntField: Some(native::GetIntField),
		GetLongField: Some(native::GetLongField),
		GetFloatField: Some(native::GetFloatField),
		GetDoubleField: Some(native::GetDoubleField),
		SetObjectField: Some(native::SetObjectField),
		SetBooleanField: Some(native::SetBooleanField),
		SetByteField: Some(native::SetByteField),
		SetCharField: Some(native::SetCharField),
		SetShortField: Some(native::SetShortField),
		SetIntField: Some(native::SetIntField),
		SetLongField: Some(native::SetLongField),
		SetFloatField: Some(native::SetFloatField),
		SetDoubleField: Some(native::SetDoubleField),
		GetStaticMethodID: Some(native::GetStaticMethodID),
		CallStaticObjectMethod: Some(native::CallStaticObjectMethod),
		CallStaticObjectMethodV: Some(native::CallStaticObjectMethodV),
		CallStaticObjectMethodA: Some(native::CallStaticObjectMethodA),
		CallStaticBooleanMethod: Some(native::CallStaticBooleanMethod),
		CallStaticBooleanMethodV: Some(native::CallStaticBooleanMethodV),
		CallStaticBooleanMethodA: Some(native::CallStaticBooleanMethodA),
		CallStaticByteMethod: Some(native::CallStaticByteMethod),
		CallStaticByteMethodV: Some(native::CallStaticByteMethodV),
		CallStaticByteMethodA: Some(native::CallStaticByteMethodA),
		CallStaticCharMethod: Some(native::CallStaticCharMethod),
		CallStaticCharMethodV: Some(native::CallStaticCharMethodV),
		CallStaticCharMethodA: Some(native::CallStaticCharMethodA),
		CallStaticShortMethod: Some(native::CallStaticShortMethod),
		CallStaticShortMethodV: Some(native::CallStaticShortMethodV),
		CallStaticShortMethodA: Some(native::CallStaticShortMethodA),
		CallStaticIntMethod: Some(native::CallStaticIntMethod),
		CallStaticIntMethodV: Some(native::CallStaticIntMethodV),
		CallStaticIntMethodA: Some(native::CallStaticIntMethodA),
		CallStaticLongMethod: Some(native::CallStaticLongMethod),
		CallStaticLongMethodV: Some(native::CallStaticLongMethodV),
		CallStaticLongMethodA: Some(native::CallStaticLongMethodA),
		CallStaticFloatMethod: Some(native::CallStaticFloatMethod),
		CallStaticFloatMethodV: Some(native::CallStaticFloatMethodV),
		CallStaticFloatMethodA: Some(native::CallStaticFloatMethodA),
		CallStaticDoubleMethod: Some(native::CallStaticDoubleMethod),
		CallStaticDoubleMethodV: Some(native::CallStaticDoubleMethodV),
		CallStaticDoubleMethodA: Some(native::CallStaticDoubleMethodA),
		CallStaticVoidMethod: Some(native::CallStaticVoidMethod),
		CallStaticVoidMethodV: Some(native::CallStaticVoidMethodV),
		CallStaticVoidMethodA: Some(native::CallStaticVoidMethodA),
		GetStaticFieldID: Some(native::GetStaticFieldID),
		GetStaticObjectField: Some(native::GetStaticObjectField),
		GetStaticBooleanField: Some(native::GetStaticBooleanField),
		GetStaticByteField: Some(native::GetStaticByteField),
		GetStaticCharField: Some(native::GetStaticCharField),
		GetStaticShortField: Some(native::GetStaticShortField),
		GetStaticIntField: Some(native::GetStaticIntField),
		GetStaticLongField: Some(native::GetStaticLongField),
		GetStaticFloatField: Some(native::GetStaticFloatField),
		GetStaticDoubleField: Some(native::GetStaticDoubleField),
		SetStaticObjectField: Some(native::SetStaticObjectField),
		SetStaticBooleanField: Some(native::SetStaticBooleanField),
		SetStaticByteField: Some(native::SetStaticByteField),
		SetStaticCharField: Some(native::SetStaticCharField),
		SetStaticShortField: Some(native::SetStaticShortField),
		SetStaticIntField: Some(native::SetStaticIntField),
		SetStaticLongField: Some(native::SetStaticLongField),
		SetStaticFloatField: Some(native::SetStaticFloatField),
		SetStaticDoubleField: Some(native::SetStaticDoubleField),
		NewString: Some(native::NewString),
		GetStringLength: Some(native::GetStringLength),
		GetStringChars: Some(native::GetStringChars),
		ReleaseStringChars: Some(native::ReleaseStringChars),
		NewStringUTF: Some(native::NewStringUTF),
		GetStringUTFLength: Some(native::GetStringUTFLength),
		GetStringUTFChars: Some(native::GetStringUTFChars),
		ReleaseStringUTFChars: Some(native::ReleaseStringUTFChars),
		GetArrayLength: Some(native::GetArrayLength),
		NewObjectArray: Some(native::NewObjectArray),
		GetObjectArrayElement: Some(native::GetObjectArrayElement),
		SetObjectArrayElement: Some(native::SetObjectArrayElement),
		NewBooleanArray: Some(native::NewBooleanArray),
		NewByteArray: Some(native::NewByteArray),
		NewCharArray: Some(native::NewCharArray),
		NewShortArray: Some(native::NewShortArray),
		NewIntArray: Some(native::NewIntArray),
		NewLongArray: Some(native::NewLongArray),
		NewFloatArray: Some(native::NewFloatArray),
		NewDoubleArray: Some(native::NewDoubleArray),
		GetBooleanArrayElements: Some(native::GetBooleanArrayElements),
		GetByteArrayElements: Some(native::GetByteArrayElements),
		GetCharArrayElements: Some(native::GetCharArrayElements),
		GetShortArrayElements: Some(native::GetShortArrayElements),
		GetIntArrayElements: Some(native::GetIntArrayElements),
		GetLongArrayElements: Some(native::GetLongArrayElements),
		GetFloatArrayElements: Some(native::GetFloatArrayElements),
		GetDoubleArrayElements: Some(native::GetDoubleArrayElements),
		ReleaseBooleanArrayElements: Some(native::ReleaseBooleanArrayElements),
		ReleaseByteArrayElements: Some(native::ReleaseByteArrayElements),
		ReleaseCharArrayElements: Some(native::ReleaseCharArrayElements),
		ReleaseShortArrayElements: Some(native::ReleaseShortArrayElements),
		ReleaseIntArrayElements: Some(native::ReleaseIntArrayElements),
		ReleaseLongArrayElements: Some(native::ReleaseLongArrayElements),
		ReleaseFloatArrayElements: Some(native::ReleaseFloatArrayElements),
		ReleaseDoubleArrayElements: Some(native::ReleaseDoubleArrayElements),
		GetBooleanArrayRegion: Some(native::GetBooleanArrayRegion),
		GetByteArrayRegion: Some(native::GetByteArrayRegion),
		GetCharArrayRegion: Some(native::GetCharArrayRegion),
		GetShortArrayRegion: Some(native::GetShortArrayRegion),
		GetIntArrayRegion: Some(native::GetIntArrayRegion),
		GetLongArrayRegion: Some(native::GetLongArrayRegion),
		GetFloatArrayRegion: Some(native::GetFloatArrayRegion),
		GetDoubleArrayRegion: Some(native::GetDoubleArrayRegion),
		SetBooleanArrayRegion: Some(native::SetBooleanArrayRegion),
		SetByteArrayRegion: Some(native::SetByteArrayRegion),
		SetCharArrayRegion: Some(native::SetCharArrayRegion),
		SetShortArrayRegion: Some(native::SetShortArrayRegion),
		SetIntArrayRegion: Some(native::SetIntArrayRegion),
		SetLongArrayRegion: Some(native::SetLongArrayRegion),
		SetFloatArrayRegion: Some(native::SetFloatArrayRegion),
		SetDoubleArrayRegion: Some(native::SetDoubleArrayRegion),
		RegisterNatives: Some(native::RegisterNatives),
		UnregisterNatives: Some(native::UnregisterNatives),
		MonitorEnter: Some(native::MonitorEnter),
		MonitorExit: Some(native::MonitorExit),
		GetJavaVM: Some(native::GetJavaVM),
		GetStringRegion: Some(native::GetStringRegion),
		GetStringUTFRegion: Some(native::GetStringUTFRegion),
		GetPrimitiveArrayCritical: Some(native::GetPrimitiveArrayCritical),
		ReleasePrimitiveArrayCritical: Some(native::ReleasePrimitiveArrayCritical),
		GetStringCritical: Some(native::GetStringCritical),
		ReleaseStringCritical: Some(native::ReleaseStringCritical),
		NewWeakGlobalRef: Some(native::NewWeakGlobalRef),
		DeleteWeakGlobalRef: Some(native::DeleteWeakGlobalRef),
		ExceptionCheck: Some(native::ExceptionCheck),
		NewDirectByteBuffer: Some(native::NewDirectByteBuffer),
		GetDirectBufferAddress: Some(native::GetDirectBufferAddress),
		GetDirectBufferCapacity: Some(native::GetDirectBufferCapacity),
		GetObjectRefType: Some(native::GetObjectRefType),
	})) as *mut core::ffi::c_void;
	0
}
unsafe extern "system" fn AttachCurrentThreadAsDaemon(
	_vm: *mut JavaVM,
	_penv: *mut *mut c_void,
	_args: *mut c_void,
) -> jint {
	todo!();
}

#[repr(C)]
struct JavaVMOption {
	option_string: *const libc::c_void,
	extra_info: *const libc::c_void,
}
impl JavaVMOption {
	fn string(&self) -> &std::ffi::CStr {
		unsafe { std::ffi::CStr::from_ptr(self.option_string as *const _) }
	}
}
impl std::fmt::Debug for JavaVMOption {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		fmt.debug_struct("JavaVMOption")
			.field("option_string", &self.string())
			.finish()
	}
}

#[repr(C)]
struct JavaVMInitArgs {
	version: jint,

	n_options: jint,
	options: *const JavaVMOption,
	ignore_unrecognized: jboolean,
}
impl JavaVMInitArgs {
	fn options(&self) -> &[JavaVMOption] {
		unsafe { std::slice::from_raw_parts(self.options, self.n_options as usize) }
	}
}

impl std::fmt::Debug for JavaVMInitArgs {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		fmt.debug_struct("JavaVMInitArgs")
			.field("version", &self.version)
			.field("ignore_unrecognized", &self.ignore_unrecognized)
			.field("options", &self.options())
			.finish()
	}
}

#[no_mangle]
extern "C" fn JNI_CreateJavaVM(
	pvm: *mut *mut JavaVM,
	penv: *mut *mut c_void,
	args: *const JavaVMInitArgs,
) -> jint {
	let mut lock = JVM.lock().expect("jvm lock");
	// Can't have multiple VMs per process
	if lock.is_some() {
		-1
	} else {
		use std::ptr::null_mut;
		vm::native::init();
		vm::oop::init();
		vm::runtime::init();
		// To be run from jre dir
		vm::runtime::add_class_path("./lib/rt.jar");
		vm::runtime::add_class_path("./lib/jsse.jar");
		let args = unsafe { &*args };
		// TODO: Pass to jvm
		let mut properties: std::collections::HashMap<String, String> =
			std::collections::HashMap::new();
		for option in args.options() {
			let option: String = option.string().to_string_lossy().into();
			if option.starts_with("-D") {
				let idx = option.find("=").expect("bad property argument format");
				properties.insert(option[2..idx].to_owned(), option[idx..].to_owned());
			} else if args.ignore_unrecognized == 0 {
				panic!("unknown option: {}", option);
			}
		}

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
			*pvm = Box::into_raw(Box::new(holder.inner()));
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
				*vm_buf = Box::into_raw(Box::new(holder.jvm.as_ref()));
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
