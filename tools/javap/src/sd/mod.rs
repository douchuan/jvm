mod code;

#[derive(Serialize)]
pub struct ClassInfoSerde {
    pub enable_sys_info: bool,
    pub sys_info: SysInfoSerde,
    pub source_file: String,
    pub class_head: String,
    pub fields: Vec<String>,
    pub methods: Vec<MethodInfoSerde>,
}

#[derive(Serialize)]
pub struct MethodInfoSerde {
    pub desc: String,
    pub line_number_table: Vec<LineNumberSerde>,
    pub codes: Vec<String>,

    pub enable_line_number: bool,
    pub enable_code: bool,
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
