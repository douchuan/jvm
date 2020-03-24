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
            // let methods = trans::class_methods(&cf);
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
        let methods: Vec<String> = {
            let methods = trans::class_methods(&cf, false);
            methods.iter().map(|it| it.desc.clone()).collect()
        };

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
        // reg.register_escape_fn(handlebars::no_escape);
        const TP_ENUM: &str = "Compiled from \"{{source_file}}\"
{{access_flags}} {{this_class}} extends {{super_class}}<{{this_class}}> {
    {{#each fields}}
    {{this}}
    {{/each}}

    {{#each methods as |method| ~}}
        {{method.desc}}
      LineNumberTable:\
        {{#each method.line_number_table}}
          line {{this.line_number}}: {{this.start_pc}}\
        {{/each}}\n
    {{/each}}
}";
        let source_file = trans::class_source_file(&cf);
        let this_class = trans::class_this_class(&cf);
        let super_class = trans::class_super_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        let fields = trans::class_fields(&cf);
        let methods: Vec<MethodInfoSerde> = {
            let methods = trans::class_methods(&cf, true);
            methods
                .iter()
                .map(|it| {
                    let line_number_table: Vec<LineNumberSerde> = it
                        .line_num_table
                        .iter()
                        .map(|it| LineNumberSerde {
                            start_pc: it.start_pc,
                            line_number: it.number,
                        })
                        .collect();

                    MethodInfoSerde {
                        desc: it.desc.clone(),
                        line_number_table,
                    }
                })
                .collect()
        };

        let data = ClassInfoSerde {
            source_file,
            access_flags,
            this_class,
            super_class,
            fields,
            methods,
        };

        println!("{}", reg.render_template(TP_ENUM, &data).unwrap());
    }
}

#[derive(Serialize)]
struct ClassInfoSerde {
    source_file: String,
    access_flags: String,
    this_class: String,
    super_class: String,
    fields: Vec<String>,
    methods: Vec<MethodInfoSerde>,
}

#[derive(Serialize)]
struct MethodInfoSerde {
    desc: String,
    line_number_table: Vec<LineNumberSerde>,
}

#[derive(Serialize)]
struct LineNumberSerde {
    start_pc: u16,
    line_number: u16,
}
