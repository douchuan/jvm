mod file;
mod net;
mod sync;
mod sys;

pub use self::file::*;
pub use self::net::*;
pub use self::sync::*;
pub use self::sys::*;

pub fn make_id(v: Vec<&str>) -> String {
    v.join(":")
}
