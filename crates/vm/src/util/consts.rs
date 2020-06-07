use classfile::BytesRef;
use crate::new_br;

lazy_static! {
    pub static ref S_INIT: BytesRef = new_br("<init>");

    pub static ref S_CLINIT: BytesRef = new_br("<clinit>");

    pub static ref S_CLINIT_SIG: BytesRef = new_br("()V");

    pub static ref S_JAVA_IO_FD: BytesRef = new_br("Ljava/io/FileDescriptor;");

    pub static ref S_FD: BytesRef = new_br("fd");

    pub static ref S_I: BytesRef = new_br("I");
}
