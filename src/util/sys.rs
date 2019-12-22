#![allow(unused)]

pub const PATH_DELIMITER: &[u8] = platform::PATH_DELIMITER;
pub const PATH_DELIMITER_STR: &str = platform::PATH_DELIMITER_STR;
pub const PATH_SEP: &[u8] = platform::PATH_SEP;
pub const PATH_SEP_STR: &str = platform::PATH_SEP_STR;

#[cfg(unix)]
mod platform {
    pub const PATH_DELIMITER: &[u8] = b":";
    pub const PATH_DELIMITER_STR: &str = ":";
    pub const PATH_SEP: &[u8] = b"/";
    pub const PATH_SEP_STR: &str = "/";
}

#[cfg(windows)]
mod platform {
    pub const PATH_DELIMITER: &[u8] = b";";
    pub const PATH_DELIMITER_STR: &str = ";";
    pub const PATH_SEP: &[u8] = b"\\";
    pub const PATH_SEP_STR: &str = "\\";
}
