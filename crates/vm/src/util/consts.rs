use crate::new_br;
use classfile::BytesRef;

lazy_static! {
    pub static ref S_INIT: BytesRef = new_br("<init>");
    pub static ref S_CLINIT: BytesRef = new_br("<clinit>");
    pub static ref S_CLINIT_SIG: BytesRef = new_br("()V");
    pub static ref S_RUN_SIG: BytesRef = new_br("()Ljava/lang/Object;");
    pub static ref S_NEW_STRING_SIG: BytesRef = new_br("([C)V");
    pub static ref S_CLAZZ: BytesRef = new_br("clazz");
    pub static ref S_FD: BytesRef = new_br("fd");
    pub static ref S_I: BytesRef = new_br("I");
    pub static ref S_SLOT: BytesRef = new_br("slot");
    pub static ref S_MODIFIERS: BytesRef = new_br("modifiers");
    pub static ref S_NAME: BytesRef = new_br("name");
    pub static ref S_SIGNATURE: BytesRef = new_br("signature");
    pub static ref S_CONSTANT_POOL_OOP: BytesRef = new_br("constantPoolOop");
    pub static ref S_RUN: BytesRef = new_br("run");
    pub static ref S_ERR: BytesRef = new_br("err");
    pub static ref S_OUT: BytesRef = new_br("out");
    pub static ref S_IN: BytesRef = new_br("in");
    pub static ref S_JAVA_LANG_CLASS: BytesRef = new_br("Ljava/lang/Class;");
    pub static ref S_JAVA_LANG_OBJECT: BytesRef = new_br("Ljava/lang/Object;");
    pub static ref S_JAVA_LANG_STRING: BytesRef = new_br("Ljava/lang/String;");
    pub static ref S_JAVA_IO_FD: BytesRef = new_br("Ljava/io/FileDescriptor;");
    pub static ref S_JAVA_IO_PRINT_STREAM: BytesRef = new_br("Ljava/io/PrintStream;");
    pub static ref S_JAVA_IO_INPUT_STREAM: BytesRef = new_br("Ljava/io/InputStream;");
}
