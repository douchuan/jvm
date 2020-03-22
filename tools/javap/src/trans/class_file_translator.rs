use super::ClassAccessFlagsTranslator;
use super::MethodTranslator;
use classfile::constant_pool;
use classfile::AttributeType;
use classfile::ClassFile;

const S_UNKNOWN: &str = "unknown";

pub struct Translator<'a> {
    cf: &'a ClassFile,
}

impl<'a> Translator<'a> {
    pub fn new(cf: &'a ClassFile) -> Self {
        Self { cf }
    }
}

impl<'a> Translator<'a> {
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

    pub fn super_class(&self) -> String {
        assert_ne!(self.cf.super_class, 0);

        constant_pool::get_class_name(&self.cf.cp, self.cf.super_class as usize).map_or_else(
            || S_UNKNOWN.into(),
            |v| String::from_utf8_lossy(v.as_slice()).into(),
        )
    }

    pub fn access_flags(&self) -> String {
        let x = ClassAccessFlagsTranslator::new(self.cf);
        x.get()
    }

    pub fn signature(&self) -> String {
        for it in &self.cf.attrs {
            match it {
                AttributeType::Signature { signature_index } => {
                    return constant_pool::get_utf8(&self.cf.cp, *signature_index as usize)
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

    pub fn methods(&self) -> Vec<String> {
        let mut methods = Vec::with_capacity(self.cf.methods.len());
        for it in self.cf.methods.iter() {
            let t = MethodTranslator::new(self.cf, it);
            methods.push(t.get());
        }

        methods
    }
}
