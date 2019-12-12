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

pub const JAVA_LANG_OBJECT: &str = "java/lang/Object";
