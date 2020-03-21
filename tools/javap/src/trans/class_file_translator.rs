use classfile::constant_pool;
use classfile::AttributeType;
use classfile::ClassFile;

const S_UNKNOWN: &str = "unknown";

pub struct ClassFileTranslator<'a> {
    cf: &'a ClassFile,
}

impl<'a> ClassFileTranslator<'a> {
    pub fn new(cf: &'a ClassFile) -> Self {
        Self {
            cf
        }
    }
}

impl<'a> ClassFileTranslator<'a> {
    pub fn source_file(&self) -> String {
        for it in &self.cf.attrs {
            match it {
                AttributeType::SourceFile { source_file_index } => {
                    return constant_pool::get_utf8(&self.cf.cp, *source_file_index as usize)
                        .map_or_else(
                            || S_UNKNOWN.into(),
                            |v| String::from_utf8_lossy(v.as_slice()).into(),
                        );
                }
                _ => (),
            }
        }

        String::from(S_UNKNOWN)
    }

    pub fn this_class(&self) -> String {
        constant_pool::get_class_name(&self.cf.cp, self.cf.this_class as usize).map_or_else(
            || S_UNKNOWN.into(),
            |v| String::from_utf8_lossy(v.as_slice()).into(),
        )
    }

    pub fn access_flags(&self) {

    }
}