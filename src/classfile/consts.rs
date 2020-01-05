use crate::classfile::types::U4;

pub const MAGIC: U4 = 0xCAFEBABE;

pub const METHOD_NAME_INIT: &[u8] = b"<init>";
pub const METHOD_NAME_CLINIT: &[u8] = b"<clinit>";

pub const MAX_CP_ENTRIES: u16 = 65535;
pub const MAX_FIELDS_NUM: u16 = 65535;
pub const MAX_METHODS_NUM: u16 = 65535;
pub const MAX_DIRECT_SUPER_INTERFACES_NUM: u16 = 65535;
pub const MAX_LOCAL_VARS_NUM: u16 = 65535;
pub const MAX_OP_STACKS_SIZE: u16 = 65535;
pub const MAX_METHOD_PARAMS_NUM: u16 = 255;
pub const MAX_CONST_STR_LEN: u16 = 65535;
pub const MAX_ARRAY_DIMENSIONS: u16 = 255;

pub const J_OBJECT: &[u8] = b"java/lang/Object";
pub const J_CLONEABLE: &[u8] = b"java/lang/Cloneable";
pub const J_SERIALIZABLE: &[u8] = b"java/io/Serializable";
pub const J_CLASS: &[u8] = b"java/lang/Class";
pub const J_STRING: &[u8] = b"java/lang/String";
pub const J_THREAD: &[u8] = b"java/lang/Thread";
pub const J_THREAD_GROUP: &[u8] = b"java/lang/ThreadGroup";
pub const J_SYSTEM: &[u8] = b"java/lang/System";

pub const J_INPUT_STREAM: &[u8] = b"java/io/InputStream";
pub const J_PRINT_STREAM: &[u8] = b"java/io/PrintStream";
pub const J_SECURITY_MANAGER: &[u8] = b"java/lang/SecurityManager";

pub const J_FIELD: &[u8] = b"java/lang/reflect/Field";
pub const J_METHOD: &[u8] = b"java/lang/reflect/Method";
pub const J_CTOR: &[u8] = b"java/lang/reflect/Constructor";
pub const J_ACCESSIBLE_OBJECT: &[u8] = b"java/lang/reflect/AccessibleObject";
pub const J_METHODHANDLE: &[u8] = b"java/lang/invoke/MethodHandle";
pub const J_METHODTYPE: &[u8] = b"java/lang/invoke/MethodType";

pub const J_INTERNAL_ERROR: &[u8] = b"java/lang/InternalError";
pub const J_NPE: &[u8] = b"java/lang/NullPointerException";
pub const J_IOEXCEPTION: &[u8] = b"java/io/IOException";
pub const J_ARRAY_INDEX_OUT_OF_BOUNDS: &[u8] = b"java/lang/ArrayIndexOutOfBoundsException";
pub const J_CLASS_NOT_FOUND: &[u8] = b"java/lang/ClassNotFoundException";
pub const J_ARITHMETIC_EX: &[u8] = b"java/lang/ArithmeticException";

pub const CONSTANT_METHOD_REF_TAG: u8 = 10;
pub const CONSTANT_INTERFACE_METHOD_REF_TAG: u8 = 11;
