#[derive(Debug, Clone)]
pub struct SysInfo {
    pub class_file: String,
    pub last_modified: String,
    pub size: usize,
    pub checksum: String,
}
