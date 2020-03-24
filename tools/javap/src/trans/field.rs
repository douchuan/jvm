use crate::trans::AccessFlagsTranslator;
use crate::trans::SignatureTypeTranslator;
use classfile::{constant_pool, ClassFile, FieldInfo, FieldSignature};
use handlebars::Handlebars;

pub struct Translator<'a> {
    cf: &'a ClassFile,
    field: &'a FieldInfo,
}

impl<'a> Translator<'a> {
    pub fn new(cf: &'a ClassFile, field: &'a FieldInfo) -> Self {
        Self { cf, field }
    }
}

impl<'a> Translator<'a> {
    pub fn get(&self) -> String {
        let reg = Handlebars::new();

        let tp_method = "{{flags}} {{type}} {{name}};";
        let tp_method_no_flags = "{{type}} {{name}}";

        let flags = self.access_flags();
        if flags.is_empty() {
            let data = json!({
                "type": self.field_type(),
                "name": self.name(),
            });

            reg.render_template(tp_method_no_flags, &data).unwrap()
        } else {
            let data = json!({
                "flags": flags,
                "type": self.field_type(),
                "name": self.name(),
            });

            reg.render_template(tp_method, &data).unwrap()
        }
    }
}

impl<'a> Translator<'a> {
    fn access_flags(&self) -> String {
        let flags = self.field.acc_flags;
        let t = AccessFlagsTranslator::new(flags);
        t.field_access_flags()
    }

    fn field_type(&self) -> String {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.field.desc_index as usize).unwrap();
        let signature = FieldSignature::new(desc.as_slice());
        signature.field_type.into_string()
    }

    fn name(&self) -> String {
        let name = constant_pool::get_utf8(&self.cf.cp, self.field.name_index as usize).unwrap();
        String::from_utf8_lossy(name.as_slice()).to_string()
    }
}
