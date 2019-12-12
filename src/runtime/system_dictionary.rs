use crate::oop::ClassRef;
use crate::util;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

type SystemDictionary = Mutex<HashMap<String, ClassRef>>;

lazy_static! {
    static ref SYS_DIC: SystemDictionary = { Mutex::new(HashMap::new()) };
}

pub fn put(key: &str, klass: ClassRef) {
    util::sync_call_ctx(&SYS_DIC, |dic| {
        dic.insert(key.into(), klass);
    })
}

pub fn find(key: &str) -> Option<ClassRef> {
    util::sync_call_ctx(&SYS_DIC, |dic| dic.get(key).map(|it| it.clone()))
}

pub fn init() {
    lazy_static::initialize(&SYS_DIC);
}
