#![allow(unused)]

pub const FILE_SEP: &str = platform::FILE_SEP;
pub const PATH_SEP: &str = platform::PATH_SEP;

#[cfg(unix)]
mod platform {
    // pub const PATH_DELIMITER: &[u8] = b":";
    pub const PATH_SEP: &str = ":";
    // pub const PATH_SEP: &[u8] = b"/";
    pub const FILE_SEP: &str = "/";
}

#[cfg(windows)]
mod platform {
    // pub const PATH_DELIMITER: &[u8] = b";";
    pub const PATH_SEP: &str = ";";
    // pub const PATH_SEP: &[u8] = b"\\";
    pub const FILE_SEP: &str = "\\";
}
