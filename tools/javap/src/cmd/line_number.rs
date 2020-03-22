use crate::cmd::Cmd;
use crate::trans::{self, AccessFlagHelper};
use classfile::ClassFile;
use handlebars::Handlebars;

pub struct LineNumber;

impl Cmd for LineNumber {
    fn run(&self, cf: ClassFile) {
        // let methods: Vec<String> = vec![];
        if cf.acc_flags.is_interface() {
            self.render_interface(cf);
        } else {
            unimplemented!()
        }
    }
}

impl LineNumber {
    fn render_interface(&self, cf: ClassFile) {
        let reg = Handlebars::new();
        const TP_INTERFACE: &str = "Compiled from \"{{source_file}}\"\n\
{{access_flags}} {{this_class}} {\n\
{{#each methods}}
    {{this}}
{{/each}}
}";

        let source_file = trans::class_source_file(&cf);
        let this_class = trans::class_this_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        let methods = trans::class_methods(&cf);

        let data = json!({
            "source_file": source_file,
            "access_flags": access_flags,
            "this_class": this_class,
            "methods": methods
        });

        println!("{}", reg.render_template(TP_INTERFACE, &data).unwrap());
    }
}
