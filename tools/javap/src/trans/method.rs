use crate::trans::AccessFlagsTranslator;
use crate::trans::SignatureTypeTranslator;
use classfile::{constant_pool, ClassFile, MethodInfo, MethodSignature};
use handlebars::Handlebars;

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
    pub fn get(&self) -> String {
        let tp_method = "{{flags}} {{return}} {{name}}({{args}});";

        let reg = Handlebars::new();
        let data = json!({
            "flags": self.access_flags(),
            "return": self.return_type(),
            "name": self.name(),
            "args": self.args().join(", ")
        });
        reg.render_template(tp_method, &data).unwrap()
    }
}

impl<'a> Translator<'a> {
    fn access_flags(&self) -> String {
        let flags = self.method.acc_flags;
        let t = AccessFlagsTranslator::new(flags);
        t.method_access_flags()
    }

    fn return_type(&self) -> String {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.method.desc_index as usize).unwrap();
        let signature = MethodSignature::new(desc.as_slice());

        signature.retype.into_string()
    }

    fn name(&self) -> String {
        let name = constant_pool::get_utf8(&self.cf.cp, self.method.name_index as usize).unwrap();

        String::from_utf8_lossy(name.as_slice()).to_string()
    }

    fn args(&self) -> Vec<String> {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.method.desc_index as usize).unwrap();
        let signature = MethodSignature::new(desc.as_slice());
        signature.args.iter().map(|it| it.into_string()).collect()
    }
}
