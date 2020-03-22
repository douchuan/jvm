use crate::cmd::Cmd;
use crate::trans::{self, AccessFlagHelper};
use classfile::ClassFile;
use handlebars::Handlebars;

pub struct LineNumber;

const TP_INTERFACE: &str = "Compiled from \"{{source_file}}\"\n\
{{access_flags}} {{this_class}} {\n\
{{#each methods}}
    {{this}}
{{/each}}
}";

impl Cmd for LineNumber {
    fn run(&self, cf: ClassFile) {
        let reg = Handlebars::new();

        let source_file = trans::class_source_file(&cf);
        let this_class = trans::class_this_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        let methods = trans::class_methods(&cf);

        // let methods: Vec<String> = vec![];
        if cf.acc_flags.is_interface() {
            let data = json!({
                "source_file": source_file,
                "access_flags": access_flags,
                "this_class": this_class,
                "methods": methods
            });
            println!("{}", reg.render_template(TP_INTERFACE, &data).unwrap());
        } else {
        }

        let super_class = trans::class_super_class(&cf);
        let signature = trans::class_signature(&cf);
        // let access_flags = trans::class_access_flags(&cf);
        // trace!("source file = {}", source_file);

        // trace!("this class  = {}", this_class);
        // trace!("super class  = {}", super_class);
        // trace!("class access flags = {}", access_flags);
        // trace!("signature= {}", signature);
    }
}
