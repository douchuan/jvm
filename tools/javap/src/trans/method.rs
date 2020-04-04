use crate::sd::CodeSerde;
use crate::trans::SignatureTypeTranslator;
use crate::trans::{AccessFlagHelper, AccessFlagsTranslator, CodeTranslator};
use classfile::{attributes::LineNumber, constant_pool, ClassFile, MethodInfo, MethodSignature};
use handlebars::Handlebars;

pub struct MethodTranslation {
    pub desc: String,
    pub line_num_table: Vec<LineNumber>,
    pub code: CodeSerde,
    pub signature: String,
    pub flags: String,
    pub throws: String,
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
    pub fn get(&self, with_line_num: bool, with_code: bool) -> MethodTranslation {
        let desc = self.build_desc();
        let line_num_table = if with_line_num {
            self.method.get_line_number_table()
        } else {
            vec![]
        };
        let code = if with_code {
            self.code()
        } else {
            Default::default()
        };
        let signature = self.signature();
        let flags = AccessFlagsTranslator::new(self.method.acc_flags).access_flag_inner();
        let throws = self.throws().unwrap_or("".to_string());

        MethodTranslation {
            desc,
            line_num_table,
            code,
            signature,
            flags,
            throws,
        }
    }
}

impl<'a> Translator<'a> {
    fn build_desc(&self) -> String {
        let name = constant_pool::get_utf8(&self.cf.cp, self.method.name_index as usize).unwrap();
        let mut desc = match name.as_slice() {
            b"<init>" => {
                let access_flags = self.access_flags();
                let name = constant_pool::get_class_name(&self.cf.cp, self.cf.this_class as usize)
                    .unwrap();
                format!(
                    "{} {}()",
                    access_flags,
                    String::from_utf8_lossy(name.as_slice())
                )
            }
            b"<clinit>" => "static {}".to_string(),
            _ => {
                let reg = Handlebars::new();
                let flags = self.access_flags();
                match flags.is_empty() {
                    true => {
                        let data = json!({
                            "return_type": self.return_type(),
                            "name": String::from_utf8_lossy(name.as_slice()),
                            "args": self.args().join(", ")
                        });

                        let tp = "{{return_type}} {{name}}({{args}})";
                        reg.render_template(tp, &data).unwrap()
                    }
                    false => {
                        let data = json!({
                            "flags": self.access_flags(),
                            "return_type": self.return_type(),
                            "name": String::from_utf8_lossy(name.as_slice()),
                            "args": self.args().join(", ")
                        });

                        let tp = "{{flags}} {{return_type}} {{name}}({{args}})";
                        reg.render_template(tp, &data).unwrap()
                    }
                }
            }
        };

        match self.throws() {
            Some(ex) => {
                desc.push_str(" throws");
                desc.push_str(" ");
                desc.push_str(ex.as_str());
                desc.push_str(";")
            }
            _ => desc.push_str(";"),
        }

        desc
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

    fn args(&self) -> Vec<String> {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.method.desc_index as usize).unwrap();
        let signature = MethodSignature::new(desc.as_slice());
        signature.args.iter().map(|it| it.into_string()).collect()
    }

    fn signature(&self) -> String {
        let desc = constant_pool::get_utf8(&self.cf.cp, self.method.desc_index as usize).unwrap();
        String::from_utf8_lossy(desc.as_slice()).to_string()
    }

    fn code(&self) -> CodeSerde {
        match self.method.get_code() {
            Some(code) => {
                let t = CodeTranslator {
                    cf: self.cf,
                    code: &code,
                };
                let codes = t.get();
                let args_size = if self.method.acc_flags.is_static() {
                    self.args().len()
                } else {
                    self.args().len() + 1
                };
                CodeSerde {
                    max_stack: code.max_stack,
                    max_locals: code.max_locals,
                    args_size,
                    codes,
                    enable_verbose: false,
                }
            }
            None => Default::default(),
        }
    }

    fn throws(&self) -> Option<String> {
        self.method.get_exceptions().map(|v| {
            let exs: Vec<String> = v
                .iter()
                .map(|it| {
                    let name = constant_pool::get_class_name(&self.cf.cp, *it as usize).unwrap();
                    String::from_utf8_lossy(name.as_slice()).replace("/", ".")
                })
                .collect();

            exs.join(", ")
        })
    }
}
