use crate::sd::CodeSerde;
use crate::trans::SignatureTypeTranslator;
use crate::trans::{AccessFlagHelper, AccessFlagsTranslator, CodeTranslator};
use classfile::attributes::LocalVariable;
use classfile::{
    attributes::LineNumber, attributes::StackMapFrame, attributes::VerificationTypeInfo,
    constant_pool, BytesRef, ClassFile, MethodInfo, MethodSignature,
};
use handlebars::Handlebars;

pub struct MethodTranslation {
    pub desc: String,
    pub line_num_table: Vec<LineNumber>,
    pub code: CodeSerde,
    pub signature: String,
    pub flags: String,
    pub throws: String,
    pub ex_table: Vec<String>,
    pub stack_map_table: Vec<StackMapTableTranslation>,
    pub local_variable_table: Vec<String>,
    pub local_variable_type_table: Vec<String>,
}

pub struct StackMapTableTranslation {
    pub tag: u8,
    pub comment: &'static str,
    pub items: Vec<String>,
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
        let signature = self.signature(false);
        let signature = String::from_utf8_lossy(signature.as_slice()).to_string();
        let flags = AccessFlagsTranslator::new(self.method.acc_flags).access_flag_inner();
        let throws = self.throws().unwrap_or("".to_string());
        let ex_table = self.ex_table();
        let stack_map_table = self.stack_map_table();
        let local_variable_table = self.local_variable_table();
        let local_variable_type_table = self.local_variable_type_table();

        MethodTranslation {
            desc,
            line_num_table,
            code,
            signature,
            flags,
            throws,
            ex_table,
            stack_map_table,
            local_variable_table,
            local_variable_type_table,
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
                    String::from_utf8_lossy(name.as_slice()).replace("/", ".")
                )
            }
            b"<clinit>" => "static {}".to_string(),
            _ => {
                let mut reg = Handlebars::new();
                reg.register_escape_fn(handlebars::no_escape);

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
        let signature = self.method_signature();
        return signature.retype.into_string();
    }

    fn args(&self) -> Vec<String> {
        let signature = self.method_signature();
        signature.args.iter().map(|it| it.into_string()).collect()
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
        self.method.get_throws().map(|v| {
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

    fn ex_table(&self) -> Vec<String> {
        let ext = self.method.get_ex_table();
        match ext {
            Some(ext) => {
                let mut table = Vec::with_capacity(1 + ext.len());
                let v = format!("{:5} {:>5} {:6} {:4}", "from", "to ", "target", "type");
                table.push(v);

                for ex in ext.iter() {
                    let t = if ex.is_finally() {
                        "any".to_string()
                    } else {
                        let name =
                            constant_pool::get_class_name(&self.cf.cp, ex.catch_type as usize)
                                .unwrap();
                        format!("Class {}", String::from_utf8_lossy(name.as_slice()))
                    };
                    let v = format!(
                        "{:>5} {:>5} {:>5}   {}",
                        ex.start_pc, ex.end_pc, ex.handler_pc, t
                    );
                    table.push(v)
                }

                table
            }
            _ => vec![],
        }
    }

    fn stack_map_table(&self) -> Vec<StackMapTableTranslation> {
        match self.method.get_stack_map_table() {
            Some(t) => {
                let mut table = Vec::with_capacity(t.len());

                for it in t.iter() {
                    match it {
                        StackMapFrame::Append {
                            tag,
                            offset_delta,
                            locals,
                        } => {
                            let infos = self.build_verification_type_infos(locals);
                            let items = vec![
                                format!("offset_delta = {}", *offset_delta),
                                format!("locals = [ {} ]", infos.join(", ")),
                            ];

                            table.push(StackMapTableTranslation {
                                tag: *tag,
                                comment: "/* append */",
                                items,
                            });
                        }
                        StackMapFrame::Same {
                            tag,
                            offset_delta: _,
                        } => {
                            table.push(StackMapTableTranslation {
                                tag: *tag,
                                comment: "/* same */",
                                items: vec![],
                            });
                        }
                        StackMapFrame::SameLocals1StackItem {
                            tag,
                            offset_delta: _,
                            stack,
                        } => {
                            let stack = stack.to_vec();
                            let infos = self.build_verification_type_infos(&stack);
                            table.push(StackMapTableTranslation {
                                tag: *tag,
                                comment: "/* same_locals_1_stack_item */",
                                items: vec![format!("stack = [ {} ]", infos.join(", "))],
                            });
                        }
                        StackMapFrame::SameLocals1StackItemExtended {
                            tag,
                            offset_delta: _,
                            stack: _,
                        } => {
                            trace!("todo: StackMapFrame::SameLocals1StackItemExtended");
                            table.push(StackMapTableTranslation {
                                tag: *tag,
                                comment: "/* todo: SameLocals1StackItemExtended */",
                                items: vec![],
                            });
                        }
                        StackMapFrame::Chop { tag, offset_delta } => {
                            table.push(StackMapTableTranslation {
                                tag: *tag,
                                comment: "/* chop */",
                                items: vec![format!("offset_delta = {}", *offset_delta)],
                            });
                        }
                        StackMapFrame::SameExtended {
                            tag,
                            offset_delta: _,
                        } => {
                            trace!("todo: StackMapFrame::SameExtended");
                            table.push(StackMapTableTranslation {
                                tag: *tag,
                                comment: "/* todo: SameExtended */",
                                items: vec![],
                            });
                        }
                        StackMapFrame::Full {
                            tag,
                            offset_delta,
                            locals,
                            stack,
                        } => {
                            let locals = self.build_verification_type_infos(locals);
                            let stack = self.build_verification_type_infos(stack);
                            let mut items = Vec::with_capacity(3);
                            items.push(format!("offset_delta = {}", *offset_delta));
                            items.push(format!("locals = [ {} ]", locals.join(", ")));
                            items.push(format!("stack = [ {} ]", stack.join(", ")));
                            table.push(StackMapTableTranslation {
                                tag: *tag,
                                comment: "/* full_frame */",
                                items,
                            });
                        }
                        StackMapFrame::Reserved(_) => {}
                    }
                }

                table
            }
            None => vec![],
        }
    }

    fn local_variable_table(&self) -> Vec<String> {
        match self.method.get_local_variable_table() {
            Some(vars) => self.build_variable_table(&vars),
            None => vec![],
        }
    }

    fn local_variable_type_table(&self) -> Vec<String> {
        match self.method.get_local_variable_type_table() {
            Some(vars) => self.build_variable_table(&vars),
            None => vec![],
        }
    }
}

impl<'a> Translator<'a> {
    fn build_verification_type_infos(&self, locals: &Vec<VerificationTypeInfo>) -> Vec<String> {
        let mut infos = Vec::with_capacity(locals.len());

        for it in locals.iter() {
            match it {
                VerificationTypeInfo::Float => {
                    infos.push("float".to_string());
                }
                VerificationTypeInfo::Top => {
                    infos.push("top".to_string());
                }
                VerificationTypeInfo::Integer => {
                    infos.push("int".to_string());
                }
                VerificationTypeInfo::Long => {
                    infos.push("long".to_string());
                }
                VerificationTypeInfo::Double => {
                    infos.push("double".to_string());
                }
                VerificationTypeInfo::Null => {
                    infos.push("null".to_string());
                }
                VerificationTypeInfo::UninitializedThis => {
                    infos.push("UninitializedThis".to_string());
                }
                VerificationTypeInfo::Object { cpool_index } => {
                    let name =
                        constant_pool::get_class_name(&self.cf.cp, *cpool_index as usize).unwrap();
                    let name = String::from_utf8_lossy(name.as_slice());
                    let v = if name.starts_with("[") {
                        format!("class \"{}\"", name)
                    } else {
                        format!("class {}", name)
                    };
                    infos.push(v);
                }
                VerificationTypeInfo::Uninitialized { offset: _ } => {
                    infos.push("Uninitialized".to_string());
                }
            }
        }

        infos
    }

    fn build_variable_table(&self, local_vars: &Vec<LocalVariable>) -> Vec<String> {
        let mut table = Vec::with_capacity(1 + local_vars.len());
        table.push(format!(
            "{:5}  {:6}  {:4}  {:>5}  {}",
            "Start", "Length", "Slot", "Name", "Signature"
        ));
        for it in local_vars.iter() {
            let name = constant_pool::get_utf8(&self.cf.cp, it.name_index as usize).unwrap();
            let name = String::from_utf8_lossy(name.as_slice());
            let signature =
                constant_pool::get_utf8(&self.cf.cp, it.signature_index as usize).unwrap();
            let signature = String::from_utf8_lossy(signature.as_slice());
            let v = format!(
                "{:>5}  {:>6}  {:>4}  {:>5}  {}",
                it.start_pc, it.length, it.index, name, signature
            );
            table.push(v);
        }

        table
    }

    //Attribute::Signature contains signature details
    //such as, generics info; but constant pool only contains container info
    //
    // Attribute::Signature
    //()Ljava/util/List<Lorg/testng/ITestNGListener;>;
    //
    //constant pool
    //()Ljava/util/List;
    fn signature(&self, try_fetch_attribute: bool) -> BytesRef {
        let mut desc = None;

        if try_fetch_attribute {
            for it in self.method.attrs.iter() {
                match it {
                    classfile::attributes::Type::Signature { signature_index } => {
                        let v = constant_pool::get_utf8(&self.cf.cp, *signature_index as usize)
                            .unwrap();
                        desc = Some(v);
                    }
                    _ => (),
                }
            }
        }

        desc.unwrap_or_else(|| {
            constant_pool::get_utf8(&self.cf.cp, self.method.desc_index as usize).unwrap()
        })
    }

    fn method_signature(&self) -> MethodSignature {
        let desc = self.signature(true);
        MethodSignature::new(desc.as_slice())
    }
}
