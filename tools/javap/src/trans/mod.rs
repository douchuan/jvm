use classfile::ClassFile;

mod class_acc_translator;
mod class_file_translator;

use self::class_acc_translator::Translator as ClassAccessFlagsTranslator;
use self::class_file_translator::Translator as ClassFileTranslator;

pub fn class_source_file(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.source_file()
}

pub fn class_this_class(cf: &ClassFile) -> String {
    let x= ClassFileTranslator::new(cf);
    x.this_class()
}

pub fn class_super_class(cf: &ClassFile) -> String {
    let x= ClassFileTranslator::new(cf);
    x.super_class()
}

pub fn access_flags(cf: &ClassFile) -> String {
    let x = ClassAccessFlagsTranslator::new(cf);
    x.get()
}

