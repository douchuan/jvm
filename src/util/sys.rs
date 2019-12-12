pub const PATH_DELIMITER: &str = platform::PATH_DELIMITER;
pub const PATH_SEP: &str = platform::PATH_SEP;

#[cfg(unix)]
mod platform {
    pub const PATH_DELIMITER: &str = ":";
    pub const PATH_SEP: &str = "/";
}

#[cfg(windows)]
mod platform {
    pub const PATH_DELIMITER: &str = ";";
    pub const PATH_SEP: &str = "\\";
}
