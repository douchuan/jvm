use jni_sys::{jclass, JNIEnv};

/// Used by java launcher as entry point
#[no_mangle]
pub unsafe extern "system" fn JVM_FindClassFromBootLoader(
	env: *const JNIEnv,
	name: *const i8,
) -> jclass {
	let class_name = std::ffi::CStr::from_ptr(name);
	println!("JVM_FindClassFromBootLoader({:?})", class_name);
	todo!();
}
