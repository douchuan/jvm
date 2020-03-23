use crate::trans::AccessFlagsTranslator;
use crate::trans::SignatureTypeTranslator;
use classfile::{attributes::LineNumber, constant_pool, ClassFile, MethodInfo, MethodSignature};
use handlebars::Handlebars;

pub struct MethodTranslation {
    pub desc: String,
    pub line_num_table: Vec<LineNumber>,
}

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
    pub fn get(&self, with_line_num: bool) -> MethodTranslation {
        let desc = self.build_desc();
        let line_num_table = if with_line_num {
            self.method.get_line_number_table()
        } else {
            vec![]
        };

        MethodTranslation {
            desc,
            line_num_table,
        }
    }
}

impl<'a> Translator<'a> {
    fn build_desc(&self) -> String {
        let name = self.name();

        if name.as_bytes() == b"<init>" {
            let access_flags = self.access_flags();
            let name =
                constant_pool::get_class_name(&self.cf.cp, self.cf.this_class as usize).unwrap();
            format!(
                "{} {}();",
                access_flags,
                String::from_utf8_lossy(name.as_slice())
            )
        } else if name.as_bytes() == b"<clinit>" {
            "static {};".to_string()
        } else {
            let tp_method = "{{flags}} {{return}} {{name}}({{args}});";
            let reg = Handlebars::new();

            let data = json!({
                "flags": self.access_flags(),
                "return": self.return_type(),
                "name": name,
                "args": self.args().join(", ")
            });
            reg.render_template(tp_method, &data).unwrap()
        }
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

        if name.as_slice() == b"<init>" {
            "<init>".to_string()
        } else if name.as_slice() == b"<clinit>" {
            "<clinit>".to_string()
        } else {
            String::from_utf8_lossy(name.as_slice()).to_string()
        }
    }

    fn args(&self) -> Vec<String> {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.method.desc_index as usize).unwrap();
        let signature = MethodSignature::new(desc.as_slice());
        signature.args.iter().map(|it| it.into_string()).collect()
    }
}
