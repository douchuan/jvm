#[derive(Serialize)]
pub struct ClassInfoSerde {
    pub sys_info: SysInfoSerde,
    pub version: ClassVersionSerde,
    pub flags: String,
    pub source_file: String,
    pub class_head: String,
    pub fields: Vec<FieldInfoSerde>,
    pub methods: Vec<MethodInfoSerde>,
    pub cp: Vec<String>,
    pub inner_classes: Vec<String>,
    pub signature: String,

    pub enable_verbose: bool,
    pub enable_sys_info: bool,

    pub has_inner_classes: bool,
    pub has_signature: bool,
}

#[derive(Serialize)]
pub struct MethodInfoSerde {
    pub desc: String,

    pub line_number_table: Vec<LineNumberSerde>,
    pub descriptor: String,
    pub signature: String,
    pub code: CodeSerde,
    pub flags: String,
    pub throws: String,
    pub ex_table: Vec<String>,
    pub stack_map_table: StackMapTableSerde,
    pub local_var_table: Vec<String>,
    pub local_var_type_table: Vec<String>,

    pub enable_line_number: bool,
    pub enable_code: bool,
    pub enable_descriptor: bool,
    pub enable_flags: bool,
    pub enable_throws: bool,
    pub enable_stack_map: bool,
    pub enable_local_var_table: bool,
    pub enable_local_var_type_table: bool,
    pub enable_attr_signature: bool,

    pub has_ex_table: bool,
}

#[derive(Serialize, Clone)]
pub struct CodeSerde {
    pub max_stack: u16,
    pub max_locals: u16,
    pub args_size: usize,
    pub codes: Vec<String>,

    pub enable_verbose: bool,
}

#[derive(Serialize)]
pub struct FieldInfoSerde {
    pub desc: String,

    pub descriptor: String, //constant pool descriptor
    pub signature: String,  //Attribute Signature
    pub flags: String,

    pub enable_descriptor: bool,
    pub enable_flags: bool,
    pub enable_attr_signature: bool,
}

#[derive(Serialize)]
pub struct LineNumberSerde {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Serialize)]
pub struct SysInfoSerde {
    pub class_file: String,
    pub last_modified: String,
    pub size: usize,
    pub checksum: String,
    pub compiled_from: String,
}

#[derive(Serialize)]
pub struct ClassVersionSerde {
    pub minor: u16,
    pub major: u16,
}

#[derive(Serialize)]
pub struct StackMapTableSerde {
    pub number_of_entries: usize,
    pub frames: Vec<StackMapFrameSerde>,
}

#[derive(Serialize)]
pub struct StackMapFrameSerde {
    pub desc: String,
    pub items: Vec<String>,
}

impl Default for SysInfoSerde {
    fn default() -> Self {
        Self {
            class_file: "".to_string(),
            last_modified: "".to_string(),
            size: 0,
            checksum: "".to_string(),
            compiled_from: "".to_string(),
        }
    }
}

impl Default for CodeSerde {
    fn default() -> Self {
        Self {
            max_stack: 0,
            max_locals: 0,
            args_size: 0,
            codes: vec![],

            enable_verbose: false,
        }
    }
}

impl Default for StackMapTableSerde {
    fn default() -> Self {
        Self {
            number_of_entries: 0,
            frames: vec![],
        }
    }
}

impl Default for StackMapFrameSerde {
    fn default() -> Self {
        Self {
            desc: "".to_string(),
            items: vec![],
        }
    }
}
