use crate::trans::AccessFlagsTranslator;
use crate::trans::SignatureTypeTranslator;
use classfile::{
    constant_pool, constant_pool::Type as ConstantPoolType, BytesRef, ClassFile, FieldInfo,
    FieldSignature,
};
use handlebars::Handlebars;

pub struct FieldTranslation {
    pub desc: String,
    pub descriptor: String,
    pub signature: String,
    pub flags: String,
    pub constant: String,
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
        let signature = self.signature();
        let flags = AccessFlagsTranslator::new(self.field.acc_flags).access_flag_inner();
        let constant = self.attr_constant_value().unwrap_or("".to_string());

        FieldTranslation {
            desc,
            descriptor,
            signature,
            flags,
            constant,
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

    fn signature(&self) -> String {
        self.attr_signature().map_or("".to_string(), |(idx, v)| {
            format!("#{:<28} // {}", idx, String::from_utf8_lossy(v.as_slice()))
        })
    }

    fn attr_signature(&self) -> Option<(usize, BytesRef)> {
        self.field.attrs.iter().find_map(|v| {
            if let classfile::attributes::Type::Signature { signature_index } = v {
                let signature_index = *signature_index as usize;
                let v = constant_pool::get_utf8(&self.cf.cp, signature_index).unwrap();
                Some((signature_index, v))
            } else {
                None
            }
        })
    }

    fn attr_constant_value(&self) -> Option<String> {
        let idx = self.field.attrs.iter().find_map(|v| {
            if let classfile::attributes::Type::ConstantValue {
                constant_value_index,
            } = v
            {
                Some(*constant_value_index as usize)
            } else {
                None
            }
        });

        idx.map(|idx| match self.cf.cp.get(idx) {
            Some(ConstantPoolType::Long { v }) => {
                let v = i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                format!("long {}", v)
            }
            Some(ConstantPoolType::Float { v }) => {
                let v = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                let v = f32::from_bits(v);
                format!("float {}", v)
            }
            Some(ConstantPoolType::Double { v }) => {
                let v = u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                let v = f64::from_bits(v);
                format!("double {}", v)
            }
            Some(ConstantPoolType::Integer { v }) => {
                let v = i32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                format!("int {}", v)
            }
            Some(ConstantPoolType::String { string_index: _ }) => {
                let v = constant_pool::get_string(&self.cf.cp, idx).unwrap();
                format!("String {}", v.escape_default())
            }
            _ => format!("todo"),
        })
    }
}
