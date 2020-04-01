use crate::cmd::Cmd;
use crate::misc::SysInfo;
use crate::sd::{ClassInfoSerde, FieldInfoSerde, LineNumberSerde, MethodInfoSerde, SysInfoSerde};
use crate::template;
use crate::trans::{self, AccessFlagHelper};
use clap::ArgMatches;
use classfile::flags as access_flags;
use classfile::ClassFile;

pub struct Disassemble {
    enable_line_number: bool,
    enable_code: bool,
    acc_flags: u16,
    enable_sys_info: bool,
    enable_inner_signature: bool,
}

impl Disassemble {
    pub fn new(m: &ArgMatches) -> Option<Self> {
        let enable_line_number = m.is_present("line_number");
        let enable_code = m.is_present("disassemble");
        let acc_flags = Self::build_acc_flags(m);
        let enable_sys_info = m.is_present("sysinfo");
        let enable_inner_signature = m.is_present("signatures");

        Some(Self {
            enable_line_number,
            enable_code,
            acc_flags,
            enable_sys_info,
            enable_inner_signature,
        })
    }
}

impl Cmd for Disassemble {
    fn run(&self, si: &SysInfo, cf: ClassFile) {
        if cf.acc_flags.is_interface() {
            self.render_interface(si, cf);
        } else if cf.acc_flags.is_enum() {
            self.render_enum(si, cf);
        } else {
            self.render_class(si, cf);
        }
    }
}

impl Disassemble {
    fn render_interface(&self, si: &SysInfo, cf: ClassFile) {
        let reg = template::get_engine();

        let sys_info = self.get_sys_info(si, &cf);
        //build class_head
        let mut head_parts = vec![];
        let this_class = trans::class_this_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        head_parts.push(access_flags.as_str());
        head_parts.push(this_class.as_str());

        let class_head = if cf.interfaces.len() != 0 {
            let parent_interfaces = trans::class_parent_interfaces(&cf).join(", ");
            head_parts.push("extends");
            head_parts.push(parent_interfaces.as_str());

            head_parts.join(" ")
        } else {
            head_parts.join(" ")
        };

        let source_file = trans::class_source_file(&cf);
        let fields: Vec<FieldInfoSerde> = self.class_fields(&cf);
        let methods: Vec<MethodInfoSerde> = {
            let methods = trans::class_methods(&cf, false, false, self.acc_flags);
            methods
                .iter()
                .map(|it| MethodInfoSerde {
                    desc: it.desc.clone(),
                    line_number_table: vec![],
                    codes: vec![],
                    signature: it.signature.clone(),
                    enable_line_number: false,
                    enable_code: false,
                    enable_inner_signature: self.enable_inner_signature,
                })
                .collect()
        };

        let data = ClassInfoSerde {
            enable_sys_info: self.enable_sys_info,
            sys_info,
            source_file,
            class_head,
            fields,
            methods,
        };

        println!("{}", reg.render_template(template::CLASS, &data).unwrap());
    }

    fn render_enum(&self, si: &SysInfo, cf: ClassFile) {
        let reg = template::get_engine();

        let sys_info = self.get_sys_info(si, &cf);
        let source_file = trans::class_source_file(&cf);

        //build class_head
        let mut head_parts = vec![];
        let this_class = trans::class_this_class(&cf);
        let super_class = trans::class_super_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        head_parts.push(access_flags.as_str());
        head_parts.push(this_class.as_str());
        head_parts.push("extends");
        let class_head = {
            let mut s = String::new();
            s.push_str(super_class.as_str());
            s.push_str("<");
            s.push_str(this_class.as_str());
            s.push_str(">");

            head_parts.push(s.as_str());

            head_parts.join(" ")
        };

        let fields: Vec<FieldInfoSerde> = self.class_fields(&cf);
        let methods: Vec<MethodInfoSerde> = {
            let methods = trans::class_methods(
                &cf,
                self.enable_line_number,
                self.enable_code,
                self.acc_flags,
            );
            methods
                .iter()
                .map(|it| {
                    let enable_line_number = self.enable_line_number;
                    let enable_code = self.enable_code;

                    let line_number_table: Vec<LineNumberSerde> = if enable_line_number {
                        it.line_num_table
                            .iter()
                            .map(|it| LineNumberSerde {
                                start_pc: it.start_pc,
                                line_number: it.number,
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    let codes = if enable_code {
                        it.codes.clone()
                    } else {
                        vec![]
                    };

                    MethodInfoSerde {
                        desc: it.desc.clone(),
                        line_number_table,
                        codes,
                        signature: it.signature.clone(),
                        enable_line_number,
                        enable_code,
                        enable_inner_signature: self.enable_inner_signature,
                    }
                })
                .collect()
        };

        let data = ClassInfoSerde {
            enable_sys_info: self.enable_sys_info,
            sys_info,
            source_file,
            class_head,
            fields,
            methods,
        };

        println!("{}", reg.render_template(template::CLASS, &data).unwrap());
    }

    fn render_class(&self, si: &SysInfo, cf: ClassFile) {
        let reg = template::get_engine();

        let sys_info = self.get_sys_info(si, &cf);
        let source_file = trans::class_source_file(&cf);

        //build class_head
        let mut head_parts = vec![];
        let this_class = trans::class_this_class(&cf);
        let access_flags = trans::class_access_flags(&cf);
        head_parts.push(access_flags.as_str());
        head_parts.push(this_class.as_str());
        let super_class = trans::class_super_class(&cf);
        let class_head = if super_class != "java.lang.Object" {
            head_parts.push("extends");
            head_parts.push(super_class.as_str());
            head_parts.join(" ")
        } else {
            head_parts.join(" ")
        };

        let fields: Vec<FieldInfoSerde> = self.class_fields(&cf);
        let methods: Vec<MethodInfoSerde> = {
            let methods = trans::class_methods(
                &cf,
                self.enable_line_number,
                self.enable_code,
                self.acc_flags,
            );
            methods
                .iter()
                .map(|it| {
                    let enable_line_number = self.enable_line_number;
                    let enable_code = self.enable_code;

                    let line_number_table: Vec<LineNumberSerde> = if enable_line_number {
                        it.line_num_table
                            .iter()
                            .map(|it| LineNumberSerde {
                                start_pc: it.start_pc,
                                line_number: it.number,
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    let codes = if enable_code {
                        it.codes.clone()
                    } else {
                        vec![]
                    };

                    MethodInfoSerde {
                        desc: it.desc.clone(),
                        line_number_table,
                        codes,
                        signature: it.signature.clone(),
                        enable_line_number,
                        enable_code,
                        enable_inner_signature: self.enable_inner_signature,
                    }
                })
                .collect()
        };

        let data = ClassInfoSerde {
            enable_sys_info: self.enable_sys_info,
            sys_info,
            source_file,
            class_head,
            fields,
            methods,
        };

        println!("{}", reg.render_template(template::CLASS, &data).unwrap());
    }

    fn get_sys_info(&self, si: &SysInfo, cf: &ClassFile) -> SysInfoSerde {
        if self.enable_sys_info {
            let source_file = trans::class_source_file(&cf);
            SysInfoSerde {
                class_file: si.class_file.clone(),
                last_modified: si.last_modified.clone(),
                size: si.size,
                checksum: si.checksum.clone(),
                compiled_from: source_file,
            }
        } else {
            SysInfoSerde::default()
        }
    }

    fn class_fields(&self, cf: &ClassFile) -> Vec<FieldInfoSerde> {
        let fields = trans::class_fields(&cf, self.acc_flags);
        fields
            .iter()
            .map(|it| FieldInfoSerde {
                desc: it.desc.clone(),
                signature: it.signature.clone(),
                enable_inner_signature: self.enable_inner_signature,
            })
            .collect()
    }
}

impl Disassemble {
    fn build_acc_flags(m: &ArgMatches) -> u16 {
        let mut flags = 0;

        if m.is_present("public") {
            flags = access_flags::ACC_PUBLIC;
        }

        if m.is_present("protected") {
            flags = access_flags::ACC_PROTECTED;
        }

        if m.is_present("private") {
            flags = access_flags::ACC_PRIVATE;
        }

        flags
    }
}
