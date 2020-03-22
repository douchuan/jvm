use jni_sys::{jclass, JNIEnv};

/// Used by java launcher as entry point
#[no_mangle]
pub unsafe extern "system" fn JVM_FindClassFromBootLoader(
	env: *const JNIEnv,
	name: *const i8,
) -> jclass {
	let class_name = std::ffi::CStr::from_ptr(name);
	let bytes = class_name.to_bytes();
	let class = vm::runtime::require_class3(None, bytes);
	crate::util::class_ref_to_jclass(class)
}
