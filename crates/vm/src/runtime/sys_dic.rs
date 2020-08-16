use crate::types::ClassRef;
use crate::util;

use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

type SystemDictionary = Mutex<FxHashMap<String, ClassRef>>;

lazy_static! {
    static ref SYS_DIC: SystemDictionary = { Mutex::new(FxHashMap::default()) };
}

pub fn put(key: &[u8], klass: ClassRef) {
    debug_assert!(!key.contains(&b'.'));

    let key = Vec::from(key);
    let key = unsafe { String::from_utf8_unchecked(key) };
    let mut dict = SYS_DIC.lock().unwrap();
    dict.insert(key, klass);
}

//key style: "sun/security/provider/Sun"
pub fn find(key: &[u8]) -> Option<ClassRef> {
    debug_assert!(!key.contains(&b'.'));
    let key = unsafe { std::str::from_utf8_unchecked(key) };
    let dict = SYS_DIC.lock().unwrap();
    dict.get(key).cloned()
}

pub fn init() {
    lazy_static::initialize(&SYS_DIC);
}
