use crate::trans::AccessFlagHelper;
use classfile::{constant_pool, ClassFile, MethodInfo, MethodSignature};

pub struct Translator<'a> {
    cf: &'a ClassFile,
    method: &'a MethodInfo,
}

impl<'a> Translator<'a> {
    pub fn new(cf: &'a ClassFile, method: &'a MethodInfo) -> Self {
        Self { cf, method }
    }
}

impl<'a> Translator<'a> {
    pub fn get(&self) {
        let mut name = String::new();
        self.build_access_flags(&mut name);
        self.build_signature(&mut name);
    }
}

impl<'a> Translator<'a> {
    fn build_access_flags(&self, name: &mut String) {
        let flags = self.method.acc_flags;

        if flags.is_public() {
            name.push_str("public");
        } else if flags.is_protected() {
            name.push_str("protected");
        } else if flags.is_private() {
            name.push_str("private");
        }

        if flags.is_final() {
            name.push_str(" final");
        } else if flags.is_abstract() {
            name.push_str(" abstract");
        }
    }

    fn build_signature(&self, name: &mut String) {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.method.desc_index as usize).unwrap();
        let signature = MethodSignature::new(desc.as_slice());

        let method_name =
            constant_pool::get_utf8(&self.cf.cp, self.method.name_index as usize).unwrap();
    }
}
