use jni_sys::{
	jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jclass, jdouble,
	jdoubleArray, jfieldID, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, jmethodID,
	jobject, jobjectArray, jobjectRefType, jshort, jshortArray, jsize, jstring, jthrowable, jvalue,
	jweak, JNIEnv, JNIInvokeInterface_, JNINativeInterface_, JNINativeMethod, JavaVM,
};
use libc::{c_char, c_void};
pub type va_list = *mut c_void;

pub unsafe extern "system" fn GetVersion(env: *mut JNIEnv) -> jint {
	todo!();
}
pub unsafe extern "system" fn DefineClass(
	env: *mut JNIEnv,
	name: *const c_char,
	loader: jobject,
	buf: *const jbyte,
	len: jsize,
) -> jclass {
	todo!();
}
pub unsafe extern "system" fn FindClass(env: *mut JNIEnv, name: *const c_char) -> jclass {
	todo!();
}
pub unsafe extern "system" fn FromReflectedMethod(env: *mut JNIEnv, method: jobject) -> jmethodID {
	todo!();
}
pub unsafe extern "system" fn FromReflectedField(env: *mut JNIEnv, field: jobject) -> jfieldID {
	todo!();
}
pub unsafe extern "system" fn ToReflectedMethod(
	env: *mut JNIEnv,
	cls: jclass,
	methodID: jmethodID,
	isStatic: jboolean,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn GetSuperclass(env: *mut JNIEnv, sub: jclass) -> jclass {
	todo!();
}
pub unsafe extern "system" fn IsAssignableFrom(
	env: *mut JNIEnv,
	sub: jclass,
	sup: jclass,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn ToReflectedField(
	env: *mut JNIEnv,
	cls: jclass,
	fieldID: jfieldID,
	isStatic: jboolean,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn Throw(env: *mut JNIEnv, obj: jthrowable) -> jint {
	todo!();
}
pub unsafe extern "system" fn ThrowNew(
	env: *mut JNIEnv,
	clazz: jclass,
	msg: *const c_char,
) -> jint {
	todo!();
}
pub unsafe extern "system" fn ExceptionOccurred(env: *mut JNIEnv) -> jthrowable {
	todo!();
}
pub unsafe extern "system" fn ExceptionDescribe(env: *mut JNIEnv) {
	todo!();
}
pub unsafe extern "system" fn ExceptionClear(env: *mut JNIEnv) {
	todo!();
}
pub unsafe extern "system" fn FatalError(env: *mut JNIEnv, msg: *const c_char) -> ! {
	todo!();
}
pub unsafe extern "system" fn PushLocalFrame(env: *mut JNIEnv, capacity: jint) -> jint {
	todo!();
}
pub unsafe extern "system" fn PopLocalFrame(env: *mut JNIEnv, result: jobject) -> jobject {
	todo!();
}
pub unsafe extern "system" fn NewGlobalRef(env: *mut JNIEnv, lobj: jobject) -> jobject {
	todo!();
}
pub unsafe extern "system" fn DeleteGlobalRef(env: *mut JNIEnv, gref: jobject) {
	todo!();
}
pub unsafe extern "system" fn DeleteLocalRef(env: *mut JNIEnv, obj: jobject) {
	todo!();
}
pub unsafe extern "system" fn IsSameObject(
	env: *mut JNIEnv,
	obj1: jobject,
	obj2: jobject,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn NewLocalRef(env: *mut JNIEnv, ref_: jobject) -> jobject {
	todo!();
}
pub unsafe extern "system" fn EnsureLocalCapacity(env: *mut JNIEnv, capacity: jint) -> jint {
	todo!();
}
pub unsafe extern "system" fn AllocObject(env: *mut JNIEnv, clazz: jclass) -> jobject {
	todo!();
}
pub unsafe extern "C" fn NewObject(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn NewObjectV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn NewObjectA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn GetObjectClass(env: *mut JNIEnv, obj: jobject) -> jclass {
	todo!();
}
pub unsafe extern "system" fn IsInstanceOf(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn GetMethodID(
	env: *mut JNIEnv,
	clazz: jclass,
	name: *const c_char,
	sig: *const c_char,
) -> jmethodID {
	todo!();
}
pub unsafe extern "C" fn CallObjectMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn CallObjectMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn CallObjectMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jobject {
	todo!();
}
pub unsafe extern "C" fn CallBooleanMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn CallBooleanMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn CallBooleanMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jboolean {
	todo!();
}
pub unsafe extern "C" fn CallByteMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn CallByteMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn CallByteMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jbyte {
	todo!();
}
pub unsafe extern "C" fn CallCharMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn CallCharMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn CallCharMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jchar {
	todo!();
}
pub unsafe extern "C" fn CallShortMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn CallShortMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn CallShortMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jshort {
	todo!();
}
pub unsafe extern "C" fn CallIntMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jint {
	todo!();
}
pub unsafe extern "system" fn CallIntMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jint {
	todo!();
}
pub unsafe extern "system" fn CallIntMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jint {
	todo!();
}
pub unsafe extern "C" fn CallLongMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn CallLongMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn CallLongMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jlong {
	todo!();
}
pub unsafe extern "C" fn CallFloatMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn CallFloatMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn CallFloatMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jfloat {
	todo!();
}
pub unsafe extern "C" fn CallDoubleMethod(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	...
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn CallDoubleMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn CallDoubleMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) -> jdouble {
	todo!();
}
pub unsafe extern "C" fn CallVoidMethod(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) {
	todo!();
}
pub unsafe extern "system" fn CallVoidMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: va_list,
) {
	todo!();
}
pub unsafe extern "system" fn CallVoidMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	methodID: jmethodID,
	args: *const jvalue,
) {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualObjectMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualObjectMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualObjectMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jobject {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualBooleanMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualBooleanMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualBooleanMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jboolean {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualByteMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualByteMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualByteMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jbyte {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualCharMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualCharMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualCharMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jchar {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualShortMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualShortMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualShortMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jshort {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualIntMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jint {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualIntMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jint {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualIntMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jint {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualLongMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualLongMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualLongMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jlong {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualFloatMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualFloatMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualFloatMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jfloat {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualDoubleMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualDoubleMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualDoubleMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jdouble {
	todo!();
}
pub unsafe extern "C" fn CallNonvirtualVoidMethod(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	...
) {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualVoidMethodV(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) {
	todo!();
}
pub unsafe extern "system" fn CallNonvirtualVoidMethodA(
	env: *mut JNIEnv,
	obj: jobject,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) {
	todo!();
}
pub unsafe extern "system" fn GetFieldID(
	env: *mut JNIEnv,
	clazz: jclass,
	name: *const c_char,
	sig: *const c_char,
) -> jfieldID {
	todo!();
}
pub unsafe extern "system" fn GetObjectField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn GetBooleanField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn GetByteField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn GetCharField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn GetShortField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn GetIntField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jint {
	todo!();
}
pub unsafe extern "system" fn GetLongField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn GetFloatField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn GetDoubleField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn SetObjectField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jobject,
) {
	todo!();
}
pub unsafe extern "system" fn SetBooleanField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jboolean,
) {
	todo!();
}
pub unsafe extern "system" fn SetByteField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jbyte,
) {
	todo!();
}
pub unsafe extern "system" fn SetCharField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jchar,
) {
	todo!();
}
pub unsafe extern "system" fn SetShortField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jshort,
) {
	todo!();
}
pub unsafe extern "system" fn SetIntField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jint,
) {
	todo!();
}
pub unsafe extern "system" fn SetLongField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jlong,
) {
	todo!();
}
pub unsafe extern "system" fn SetFloatField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jfloat,
) {
	todo!();
}
pub unsafe extern "system" fn SetDoubleField(
	env: *mut JNIEnv,
	obj: jobject,
	fieldID: jfieldID,
	val: jdouble,
) {
	todo!();
}
pub unsafe extern "system" fn GetStaticMethodID(
	env: *mut JNIEnv,
	clazz: jclass,
	name: *const c_char,
	sig: *const c_char,
) -> jmethodID {
	todo!();
}
pub unsafe extern "C" fn CallStaticObjectMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn CallStaticObjectMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn CallStaticObjectMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jobject {
	todo!();
}
pub unsafe extern "C" fn CallStaticBooleanMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn CallStaticBooleanMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn CallStaticBooleanMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jboolean {
	todo!();
}
pub unsafe extern "C" fn CallStaticByteMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn CallStaticByteMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn CallStaticByteMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jbyte {
	todo!();
}
pub unsafe extern "C" fn CallStaticCharMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn CallStaticCharMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn CallStaticCharMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jchar {
	todo!();
}
pub unsafe extern "C" fn CallStaticShortMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn CallStaticShortMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn CallStaticShortMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jshort {
	todo!();
}
pub unsafe extern "C" fn CallStaticIntMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jint {
	todo!();
}
pub unsafe extern "system" fn CallStaticIntMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jint {
	todo!();
}
pub unsafe extern "system" fn CallStaticIntMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jint {
	todo!();
}
pub unsafe extern "C" fn CallStaticLongMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn CallStaticLongMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn CallStaticLongMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jlong {
	todo!();
}
pub unsafe extern "C" fn CallStaticFloatMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn CallStaticFloatMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn CallStaticFloatMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jfloat {
	todo!();
}
pub unsafe extern "C" fn CallStaticDoubleMethod(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	...
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn CallStaticDoubleMethodV(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: va_list,
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn CallStaticDoubleMethodA(
	env: *mut JNIEnv,
	clazz: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) -> jdouble {
	todo!();
}
pub unsafe extern "C" fn CallStaticVoidMethod(
	env: *mut JNIEnv,
	cls: jclass,
	methodID: jmethodID,
	...
) {
	todo!();
}
pub unsafe extern "system" fn CallStaticVoidMethodV(
	env: *mut JNIEnv,
	cls: jclass,
	methodID: jmethodID,
	args: va_list,
) {
	todo!();
}
pub unsafe extern "system" fn CallStaticVoidMethodA(
	env: *mut JNIEnv,
	cls: jclass,
	methodID: jmethodID,
	args: *const jvalue,
) {
	todo!();
}
pub unsafe extern "system" fn GetStaticFieldID(
	env: *mut JNIEnv,
	clazz: jclass,
	name: *const c_char,
	sig: *const c_char,
) -> jfieldID {
	todo!();
}
pub unsafe extern "system" fn GetStaticObjectField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn GetStaticBooleanField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn GetStaticByteField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jbyte {
	todo!();
}
pub unsafe extern "system" fn GetStaticCharField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jchar {
	todo!();
}
pub unsafe extern "system" fn GetStaticShortField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jshort {
	todo!();
}
pub unsafe extern "system" fn GetStaticIntField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jint {
	todo!();
}
pub unsafe extern "system" fn GetStaticLongField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jlong {
	todo!();
}
pub unsafe extern "system" fn GetStaticFloatField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jfloat {
	todo!();
}
pub unsafe extern "system" fn GetStaticDoubleField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
) -> jdouble {
	todo!();
}
pub unsafe extern "system" fn SetStaticObjectField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jobject,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticBooleanField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jboolean,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticByteField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jbyte,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticCharField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jchar,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticShortField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jshort,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticIntField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jint,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticLongField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jlong,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticFloatField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jfloat,
) {
	todo!();
}
pub unsafe extern "system" fn SetStaticDoubleField(
	env: *mut JNIEnv,
	clazz: jclass,
	fieldID: jfieldID,
	value: jdouble,
) {
	todo!();
}
pub unsafe extern "system" fn NewString(
	env: *mut JNIEnv,
	unicode: *const jchar,
	len: jsize,
) -> jstring {
	todo!();
}
pub unsafe extern "system" fn GetStringLength(env: *mut JNIEnv, str: jstring) -> jsize {
	todo!();
}
pub unsafe extern "system" fn GetStringChars(
	env: *mut JNIEnv,
	str: jstring,
	isCopy: *mut jboolean,
) -> *const jchar {
	todo!();
}
pub unsafe extern "system" fn ReleaseStringChars(
	env: *mut JNIEnv,
	str: jstring,
	chars: *const jchar,
) {
	todo!();
}
pub unsafe extern "system" fn NewStringUTF(env: *mut JNIEnv, utf: *const c_char) -> jstring {
	todo!();
}
pub unsafe extern "system" fn GetStringUTFLength(env: *mut JNIEnv, str: jstring) -> jsize {
	todo!();
}
pub unsafe extern "system" fn GetStringUTFChars(
	env: *mut JNIEnv,
	str: jstring,
	isCopy: *mut jboolean,
) -> *const c_char {
	todo!();
}
pub unsafe extern "system" fn ReleaseStringUTFChars(
	env: *mut JNIEnv,
	str: jstring,
	chars: *const c_char,
) {
	todo!();
}
pub unsafe extern "system" fn GetArrayLength(env: *mut JNIEnv, array: jarray) -> jsize {
	todo!();
}
pub unsafe extern "system" fn NewObjectArray(
	env: *mut JNIEnv,
	len: jsize,
	clazz: jclass,
	init: jobject,
) -> jobjectArray {
	todo!();
}
pub unsafe extern "system" fn GetObjectArrayElement(
	env: *mut JNIEnv,
	array: jobjectArray,
	index: jsize,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn SetObjectArrayElement(
	env: *mut JNIEnv,
	array: jobjectArray,
	index: jsize,
	val: jobject,
) {
	todo!();
}
pub unsafe extern "system" fn NewBooleanArray(env: *mut JNIEnv, len: jsize) -> jbooleanArray {
	todo!();
}
pub unsafe extern "system" fn NewByteArray(env: *mut JNIEnv, len: jsize) -> jbyteArray {
	todo!();
}
pub unsafe extern "system" fn NewCharArray(env: *mut JNIEnv, len: jsize) -> jcharArray {
	todo!();
}
pub unsafe extern "system" fn NewShortArray(env: *mut JNIEnv, len: jsize) -> jshortArray {
	todo!();
}
pub unsafe extern "system" fn NewIntArray(env: *mut JNIEnv, len: jsize) -> jintArray {
	todo!();
}
pub unsafe extern "system" fn NewLongArray(env: *mut JNIEnv, len: jsize) -> jlongArray {
	todo!();
}
pub unsafe extern "system" fn NewFloatArray(env: *mut JNIEnv, len: jsize) -> jfloatArray {
	todo!();
}
pub unsafe extern "system" fn NewDoubleArray(env: *mut JNIEnv, len: jsize) -> jdoubleArray {
	todo!();
}
pub unsafe extern "system" fn GetBooleanArrayElements(
	env: *mut JNIEnv,
	array: jbooleanArray,
	isCopy: *mut jboolean,
) -> *mut jboolean {
	todo!();
}
pub unsafe extern "system" fn GetByteArrayElements(
	env: *mut JNIEnv,
	array: jbyteArray,
	isCopy: *mut jboolean,
) -> *mut jbyte {
	todo!();
}
pub unsafe extern "system" fn GetCharArrayElements(
	env: *mut JNIEnv,
	array: jcharArray,
	isCopy: *mut jboolean,
) -> *mut jchar {
	todo!();
}
pub unsafe extern "system" fn GetShortArrayElements(
	env: *mut JNIEnv,
	array: jshortArray,
	isCopy: *mut jboolean,
) -> *mut jshort {
	todo!();
}
pub unsafe extern "system" fn GetIntArrayElements(
	env: *mut JNIEnv,
	array: jintArray,
	isCopy: *mut jboolean,
) -> *mut jint {
	todo!();
}
pub unsafe extern "system" fn GetLongArrayElements(
	env: *mut JNIEnv,
	array: jlongArray,
	isCopy: *mut jboolean,
) -> *mut jlong {
	todo!();
}
pub unsafe extern "system" fn GetFloatArrayElements(
	env: *mut JNIEnv,
	array: jfloatArray,
	isCopy: *mut jboolean,
) -> *mut jfloat {
	todo!();
}
pub unsafe extern "system" fn GetDoubleArrayElements(
	env: *mut JNIEnv,
	array: jdoubleArray,
	isCopy: *mut jboolean,
) -> *mut jdouble {
	todo!();
}
pub unsafe extern "system" fn ReleaseBooleanArrayElements(
	env: *mut JNIEnv,
	array: jbooleanArray,
	elems: *mut jboolean,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn ReleaseByteArrayElements(
	env: *mut JNIEnv,
	array: jbyteArray,
	elems: *mut jbyte,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn ReleaseCharArrayElements(
	env: *mut JNIEnv,
	array: jcharArray,
	elems: *mut jchar,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn ReleaseShortArrayElements(
	env: *mut JNIEnv,
	array: jshortArray,
	elems: *mut jshort,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn ReleaseIntArrayElements(
	env: *mut JNIEnv,
	array: jintArray,
	elems: *mut jint,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn ReleaseLongArrayElements(
	env: *mut JNIEnv,
	array: jlongArray,
	elems: *mut jlong,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn ReleaseFloatArrayElements(
	env: *mut JNIEnv,
	array: jfloatArray,
	elems: *mut jfloat,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn ReleaseDoubleArrayElements(
	env: *mut JNIEnv,
	array: jdoubleArray,
	elems: *mut jdouble,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn GetBooleanArrayRegion(
	env: *mut JNIEnv,
	array: jbooleanArray,
	start: jsize,
	l: jsize,
	buf: *mut jboolean,
) {
	todo!();
}
pub unsafe extern "system" fn GetByteArrayRegion(
	env: *mut JNIEnv,
	array: jbyteArray,
	start: jsize,
	len: jsize,
	buf: *mut jbyte,
) {
	todo!();
}
pub unsafe extern "system" fn GetCharArrayRegion(
	env: *mut JNIEnv,
	array: jcharArray,
	start: jsize,
	len: jsize,
	buf: *mut jchar,
) {
	todo!();
}
pub unsafe extern "system" fn GetShortArrayRegion(
	env: *mut JNIEnv,
	array: jshortArray,
	start: jsize,
	len: jsize,
	buf: *mut jshort,
) {
	todo!();
}
pub unsafe extern "system" fn GetIntArrayRegion(
	env: *mut JNIEnv,
	array: jintArray,
	start: jsize,
	len: jsize,
	buf: *mut jint,
) {
	todo!();
}
pub unsafe extern "system" fn GetLongArrayRegion(
	env: *mut JNIEnv,
	array: jlongArray,
	start: jsize,
	len: jsize,
	buf: *mut jlong,
) {
	todo!();
}
pub unsafe extern "system" fn GetFloatArrayRegion(
	env: *mut JNIEnv,
	array: jfloatArray,
	start: jsize,
	len: jsize,
	buf: *mut jfloat,
) {
	todo!();
}
pub unsafe extern "system" fn GetDoubleArrayRegion(
	env: *mut JNIEnv,
	array: jdoubleArray,
	start: jsize,
	len: jsize,
	buf: *mut jdouble,
) {
	todo!();
}
pub unsafe extern "system" fn SetBooleanArrayRegion(
	env: *mut JNIEnv,
	array: jbooleanArray,
	start: jsize,
	l: jsize,
	buf: *const jboolean,
) {
	todo!();
}
pub unsafe extern "system" fn SetByteArrayRegion(
	env: *mut JNIEnv,
	array: jbyteArray,
	start: jsize,
	len: jsize,
	buf: *const jbyte,
) {
	todo!();
}
pub unsafe extern "system" fn SetCharArrayRegion(
	env: *mut JNIEnv,
	array: jcharArray,
	start: jsize,
	len: jsize,
	buf: *const jchar,
) {
	todo!();
}
pub unsafe extern "system" fn SetShortArrayRegion(
	env: *mut JNIEnv,
	array: jshortArray,
	start: jsize,
	len: jsize,
	buf: *const jshort,
) {
	todo!();
}
pub unsafe extern "system" fn SetIntArrayRegion(
	env: *mut JNIEnv,
	array: jintArray,
	start: jsize,
	len: jsize,
	buf: *const jint,
) {
	todo!();
}
pub unsafe extern "system" fn SetLongArrayRegion(
	env: *mut JNIEnv,
	array: jlongArray,
	start: jsize,
	len: jsize,
	buf: *const jlong,
) {
	todo!();
}
pub unsafe extern "system" fn SetFloatArrayRegion(
	env: *mut JNIEnv,
	array: jfloatArray,
	start: jsize,
	len: jsize,
	buf: *const jfloat,
) {
	todo!();
}
pub unsafe extern "system" fn SetDoubleArrayRegion(
	env: *mut JNIEnv,
	array: jdoubleArray,
	start: jsize,
	len: jsize,
	buf: *const jdouble,
) {
	todo!();
}
pub unsafe extern "system" fn RegisterNatives(
	env: *mut JNIEnv,
	clazz: jclass,
	methods: *const JNINativeMethod,
	nMethods: jint,
) -> jint {
	todo!();
}
pub unsafe extern "system" fn UnregisterNatives(env: *mut JNIEnv, clazz: jclass) -> jint {
	todo!();
}
pub unsafe extern "system" fn MonitorEnter(env: *mut JNIEnv, obj: jobject) -> jint {
	todo!();
}
pub unsafe extern "system" fn MonitorExit(env: *mut JNIEnv, obj: jobject) -> jint {
	todo!();
}
pub unsafe extern "system" fn GetJavaVM(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
	todo!();
}
pub unsafe extern "system" fn GetStringRegion(
	env: *mut JNIEnv,
	str: jstring,
	start: jsize,
	len: jsize,
	buf: *mut jchar,
) {
	todo!();
}
pub unsafe extern "system" fn GetStringUTFRegion(
	env: *mut JNIEnv,
	str: jstring,
	start: jsize,
	len: jsize,
	buf: *mut c_char,
) {
	todo!();
}
pub unsafe extern "system" fn GetPrimitiveArrayCritical(
	env: *mut JNIEnv,
	array: jarray,
	isCopy: *mut jboolean,
) -> *mut c_void {
	todo!();
}
pub unsafe extern "system" fn ReleasePrimitiveArrayCritical(
	env: *mut JNIEnv,
	array: jarray,
	carray: *mut c_void,
	mode: jint,
) {
	todo!();
}
pub unsafe extern "system" fn GetStringCritical(
	env: *mut JNIEnv,
	string: jstring,
	isCopy: *mut jboolean,
) -> *const jchar {
	todo!();
}
pub unsafe extern "system" fn ReleaseStringCritical(
	env: *mut JNIEnv,
	string: jstring,
	cstring: *const jchar,
) {
	todo!();
}
pub unsafe extern "system" fn NewWeakGlobalRef(env: *mut JNIEnv, obj: jobject) -> jweak {
	todo!();
}
pub unsafe extern "system" fn DeleteWeakGlobalRef(env: *mut JNIEnv, ref_: jweak) {
	todo!();
}
pub unsafe extern "system" fn ExceptionCheck(env: *mut JNIEnv) -> jboolean {
	todo!();
}
pub unsafe extern "system" fn NewDirectByteBuffer(
	env: *mut JNIEnv,
	address: *mut c_void,
	capacity: jlong,
) -> jobject {
	todo!();
}
pub unsafe extern "system" fn GetDirectBufferAddress(
	env: *mut JNIEnv,
	buf: jobject,
) -> *mut c_void {
	todo!();
}
pub unsafe extern "system" fn GetDirectBufferCapacity(env: *mut JNIEnv, buf: jobject) -> jlong {
	todo!();
}
pub unsafe extern "system" fn GetObjectRefType(env: *mut JNIEnv, obj: jobject) -> jobjectRefType {
	todo!();
}
