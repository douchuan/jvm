use crate::trans::AccessFlagsTranslator;
use crate::trans::SignatureTypeTranslator;
use classfile::{constant_pool, ClassFile, FieldInfo, FieldSignature};
use handlebars::Handlebars;

pub struct FieldTranslation {
    pub desc: String,
    pub descriptor: String,
    pub flags: String,
}

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
    pub fn get(&self) -> FieldTranslation {
        let reg = Handlebars::new();
        let flags = self.access_flags();
        let desc = match flags.is_empty() {
            true => {
                let data = json!({
                    "type": self.field_type(),
                    "name": self.name(),
                });

                let tp = "{{type}} {{name}};";
                reg.render_template(tp, &data).unwrap()
            }
            false => {
                let data = json!({
                    "flags": flags,
                    "type": self.field_type(),
                    "name": self.name(),
                });

                let tp = "{{flags}} {{type}} {{name}};";
                reg.render_template(tp, &data).unwrap()
            }
        };

        let descriptor = self.descriptor();
        let flags = AccessFlagsTranslator::new(self.field.acc_flags).access_flag_inner();

        FieldTranslation {
            desc,
            descriptor,
            flags,
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

    fn descriptor(&self) -> String {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.field.desc_index as usize).unwrap();
        String::from_utf8_lossy(desc.as_slice()).to_string()
    }
}
