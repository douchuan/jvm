use crate::cmd::Cmd;
use crate::trans::{self, AccFlagHelper};
use classfile::ClassFile;
use handlebars::Handlebars;

pub struct LineNumber;

const TP_INTERFACE: &str = "Compiled from \"{{source_file}}\"\n\
{{access_flags}} {{this_class}} {\n\
}";

impl Cmd for LineNumber {
    fn run(&self, cf: ClassFile) {
        let mut reg = Handlebars::new();

        let source_file = trans::class_source_file(&cf);
        let this_class = trans::class_this_class(&cf);
        let access_flags = trans::class_access_flags(&cf);

        if cf.acc_flags.is_interface() {
            println!(
                "{}",
                reg.render_template(
                    TP_INTERFACE,
                    &json!({
                    "source_file": source_file,
                    "access_flags": access_flags,
                    "this_class": this_class,
                    })
                )
                .unwrap()
            );
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
