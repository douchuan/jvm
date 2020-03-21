use classfile::ClassFile;

mod class_file_translator;

use self::class_file_translator::ClassFileTranslator;

pub fn extract_source_file(cf: &ClassFile) -> String {
    let extractor = ClassFileTranslator::new(cf);
    extractor.source_file()
}

pub fn extract_access_flags(cf: &ClassFile) {

}

pub fn extract_this_class_name(cf: &ClassFile) -> String {
    let extractor = ClassFileTranslator::new(cf);
    extractor.this_class()
}
