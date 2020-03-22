use classfile::ClassFile;

mod access_flag;
mod class_file;
mod method;
mod signature_type;

pub use self::access_flag::AccessFlagHelper;
pub use self::access_flag::Translator as AccessFlagsTranslator;
pub use self::class_file::Translator as ClassFileTranslator;
pub use self::method::Translator as MethodTranslator;
pub use self::signature_type::Translator as SignatureTypeTranslator;

pub fn class_source_file(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.source_file()
}

pub fn class_this_class(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.this_class()
}

#[allow(unused)]
pub fn class_super_class(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.super_class()
}

pub fn class_access_flags(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.access_flags()
}

#[allow(unused)]
pub fn class_signature(cf: &ClassFile) -> String {
    let x = ClassFileTranslator::new(cf);
    x.signature()
}

pub fn class_methods(cf: &ClassFile) -> Vec<String> {
    let x = ClassFileTranslator::new(cf);
    x.methods()
}
