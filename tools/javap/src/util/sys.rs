#![allow(unused)]

use std::time::SystemTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

pub const FILE_SEP: &str = platform::FILE_SEP;
pub const PATH_SEP: &str = platform::PATH_SEP;
pub const LINE_SEP: &str = "\n";
const LAST_MODIFIED_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[month repr:short] [day padding:space], [year]");

#[cfg(unix)]
mod platform {
    pub const FILE_SEP: &str = "/";
    pub const PATH_SEP: &str = ":";
}

#[cfg(windows)]
mod platform {
    pub const FILE_SEP: &str = "\\";
    pub const PATH_SEP: &str = ";";
}

pub fn to_abs_path(src: &str) -> String {
    let src = std::path::PathBuf::from(src);
    match std::fs::canonicalize(&src) {
        Ok(pb) => pb.to_string_lossy().to_string(),
        Err(_) => String::new(),
    }
}

pub fn md5_checksum(data: &[u8]) -> String {
    let digest = md5::compute(data);
    format!("{:x}", digest)
}

pub fn format_time1(t: SystemTime) -> String {
    match t.duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(t) => {
            let odt = OffsetDateTime::from_unix_timestamp(t.as_secs() as i64);
            match odt {
                Ok(odt) => odt.format(&LAST_MODIFIED_FORMAT).unwrap_or_default(),
                Err(_) => String::new(),
            }
        }
        Err(_) => String::new(),
    }
}

pub fn format_time2(sec: i64) -> String {
    match OffsetDateTime::from_unix_timestamp(sec) {
        Ok(odt) => odt.format(&LAST_MODIFIED_FORMAT).unwrap_or_default(),
        Err(_) => String::new(),
    }
}
