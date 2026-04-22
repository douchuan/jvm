use crate::types::ClassRef;
use std::sync::{Mutex, OnceLock};

use rustc_hash::FxHashMap;

type SystemDictionary = Mutex<FxHashMap<String, ClassRef>>;

static SYS_DIC: OnceLock<SystemDictionary> = OnceLock::new();

pub fn put(key: &[u8], klass: ClassRef) {
    debug_assert!(!key.contains(&b'.'));

    let key = Vec::from(key);
    let key = unsafe { String::from_utf8_unchecked(key) };
    let mut dict = SYS_DIC.get().unwrap().lock().unwrap();
    dict.insert(key, klass);
}

//key style: "sun/security/provider/Sun"
pub fn find(key: &[u8]) -> Option<ClassRef> {
    debug_assert!(!key.contains(&b'.'));
    let key = unsafe { std::str::from_utf8_unchecked(key) };
    let dict = SYS_DIC.get().unwrap().lock().unwrap();
    dict.get(key).cloned()
}

pub fn init() {
    SYS_DIC.get_or_init(|| Mutex::new(FxHashMap::default()));
}
