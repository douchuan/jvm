use std::cell::RefCell;

use rustc_hash::FxHashMap;

use classfile::ConstantPool;

use crate::oop::field;
use crate::types::{FieldIdRef, MethodIdRef};
use crate::{oop, runtime};

enum CacheType {
    Field(FieldIdRef),
    Method(MethodIdRef),
}

impl CacheType {
    fn extract_field(&self) -> FieldIdRef {
        match self {
            CacheType::Field(fid) => fid.clone(),
            _ => unreachable!(),
        }
    }

    fn extract_method(&self) -> MethodIdRef {
        match self {
            CacheType::Method(mid) => mid.clone(),
            _ => unreachable!(),
        }
    }
}

pub struct ConstantPoolCache {
    cp: ConstantPool,
    cache: RefCell<FxHashMap<usize, CacheType>>,
}

impl ConstantPoolCache {
    pub fn new(cp: ConstantPool) -> Self {
        Self {
            cp,
            cache: RefCell::new(FxHashMap::default()),
        }
    }

    pub fn get_field(&self, idx: usize, is_static: bool) -> FieldIdRef {
        let cache = self.cache.borrow();
        let it = cache.get(&idx);
        match it {
            Some(it) => it.extract_field(),
            None => {
                drop(cache);
                let fid = field::get_field_ref(&self.cp, idx, is_static);
                self.cache_field(idx, fid.clone());
                fid
            }
        }
    }

    fn cache_field(&self, k: usize, v: FieldIdRef) {
        let mut cache = self.cache.borrow_mut();
        let v = CacheType::Field(v);
        cache.insert(k, v);
    }

    pub fn get_method(&self, idx: usize) -> MethodIdRef {
        let cache = self.cache.borrow();
        let it = cache.get(&idx);
        match it {
            Some(it) => it.extract_method(),
            None => {
                drop(cache);
                let m = runtime::method::get_method_ref(&self.cp, idx).unwrap();
                self.cache_method(idx, m.clone());
                m
            }
        }
    }

    fn cache_method(&self, k: usize, v: MethodIdRef) {
        let mut cache = self.cache.borrow_mut();
        let v = CacheType::Method(v);
        cache.insert(k, v);
    }
}
