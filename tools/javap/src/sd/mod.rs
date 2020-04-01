mod code;

#[derive(Serialize)]
pub struct ClassInfoSerde {
    pub enable_sys_info: bool,
    pub sys_info: SysInfoSerde,
    pub source_file: String,
    pub class_head: String,
    pub fields: Vec<FieldInfoSerde>,
    pub methods: Vec<MethodInfoSerde>,
}

#[derive(Serialize)]
pub struct MethodInfoSerde {
    pub desc: String,
    pub line_number_table: Vec<LineNumberSerde>,
    pub codes: Vec<String>,
    pub signature: String,

    pub enable_line_number: bool,
    pub enable_code: bool,
    pub enable_inner_signature: bool,
}

#[derive(Serialize)]
pub struct FieldInfoSerde {
    pub desc: String,
    pub signature: String,

    pub enable_inner_signature: bool,
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
