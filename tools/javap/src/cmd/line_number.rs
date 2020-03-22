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
        } else if cf.acc_flags.is_enum() {
            self.render_enum(cf);
        } else {
            let methods = trans::class_methods(&cf);
            // trace!("{:?}", methods);
            unimplemented!()
        }
    }
}

impl LineNumber {
    fn render_interface(&self, cf: ClassFile) {
        let reg = Handlebars::new();
        const TP_INTERFACE: &str = "Compiled from \"{{source_file}}\"
{{access_flags}} {{this_class}} {
{{#each fields}}
    {{this}}
{{/each}}
{{#each methods}}
    {{this}}
{{/each}}
}";
        const TP_INTERFACE_WITH_SUPER: &str = "Compiled from \"{{source_file}}\"
{{access_flags}} {{this_class}} extends {{parent_interfaces}} {
{{#each fields}}
    {{this}}
{{/each}}
{{#each methods}}
    {{this}}
{{/each}}
}";

        let source_file = trans::class_source_file(&cf);
        let this_class = trans::class_this_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        let fields = trans::class_fields(&cf);
        let methods = trans::class_methods(&cf);

        if cf.interfaces.len() != 0 {
            let parent_interfaces = trans::class_parent_interfaces(&cf).join(", ");

            let data = json!({
                "source_file": source_file,
                "access_flags": access_flags,
                "this_class": this_class,
                "parent_interfaces": parent_interfaces,
                "fields": fields,
                "methods": methods,
            });

            println!(
                "{}",
                reg.render_template(TP_INTERFACE_WITH_SUPER, &data).unwrap()
            );
        } else {
            let data = json!({
                "source_file": source_file,
                "access_flags": access_flags,
                "this_class": this_class,
                "fields": fields,
                "methods": methods
            });

            println!("{}", reg.render_template(TP_INTERFACE, &data).unwrap());
        }
    }

    fn render_enum(&self, cf: ClassFile) {
        let reg = Handlebars::new();
        const TP_ENUM: &str = "Compiled from \"{{source_file}}\"
{{access_flags}} {{this_class}} extends {{super_class}}<{{this_class}}> {
{{#each fields}}
    {{this}}
{{/each}}
{{#each methods}}
    {{this}}
{{/each}}
}";
        let source_file = trans::class_source_file(&cf);
        let this_class = trans::class_this_class(&cf);
        let super_class = trans::class_super_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        let fields = trans::class_fields(&cf);
        let methods = trans::class_methods(&cf);
        let data = json!({
            "source_file": source_file,
            "access_flags": access_flags,
            "this_class": this_class,
            "super_class": super_class,
            "fields": fields,
            "methods": methods
        });

        println!("{}", reg.render_template(TP_ENUM, &data).unwrap());
    }
}
