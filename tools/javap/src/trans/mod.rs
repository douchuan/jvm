use classfile::ClassFile;

mod access_flag;
mod class_acc_translator;
mod class_file_translator;

pub use self::access_flag::{AccessFlag, AccessFlagHelper};
use self::class_acc_translator::Translator as ClassAccessFlagsTranslator;
use self::class_file_translator::Translator as ClassFileTranslator;

pub fn class_source_file(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.source_file()
}

pub fn class_this_class(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.this_class()
}

pub fn class_super_class(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.super_class()
}

pub fn class_access_flags(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.access_flags()
}

pub fn class_signature(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.signature()
}
