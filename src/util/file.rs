#![allow(unused)]

use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

pub fn read<P: AsRef<Path>>(path: P) -> Vec<u8> {
    fs::read(path).unwrap_or(vec![])
}

pub fn write<P: AsRef<Path>>(path: P, contents: &[u8]) -> bool {
    let f = |path: P, contents: &[u8]| -> std::io::Result<bool> {
        let mut file = File::create(path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        Ok(true)
    };
    let r = f(path, contents);
    r.unwrap_or(false)
}

pub fn rm<P: AsRef<Path>>(path: P) {
    let _r = fs::remove_file(path);
}
