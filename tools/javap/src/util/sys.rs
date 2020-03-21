#![allow(unused)]

pub const FILE_SEP: &str = platform::FILE_SEP;
pub const PATH_SEP: &str = platform::PATH_SEP;
pub const LINE_SEP: &str = "\n";

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
