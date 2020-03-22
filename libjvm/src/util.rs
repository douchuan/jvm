/// Conversions between jvm and jni api
use jni_sys::{jclass, JNIEnv};

pub fn class_ref_to_jclass(class_ref: Option<vm::types::ClassRef>) -> jclass {
	if let Some(class_ref) = class_ref {
		class_ref.as_ref() as *const _ as jclass
	} else {
		std::ptr::null_mut()
	}
}
