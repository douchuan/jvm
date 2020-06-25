use crate::oop;
use crate::oop::field;
use crate::types::{FieldIdRef, MethodIdRef};
use classfile::ConstantPool;
use std::cell::RefCell;
use std::collections::HashMap;

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
    cache: RefCell<HashMap<usize, CacheType>>,
}

impl ConstantPoolCache {
    pub fn new(cp: ConstantPool) -> Self {
        Self {
            cp,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_field(&self, idx: usize, is_static: bool) -> FieldIdRef {
        let cache = self.cache.borrow();
        let it = cache.get(&idx);
        match it {
            Some(it) => it.extract_field(),
            None => {
                drop(cache);
                self.cache_field(idx, is_static)
            }
        }
    }

    fn cache_field(&self, idx: usize, is_static: bool) -> FieldIdRef {
        let fid = field::get_field_ref(&self.cp, idx, is_static);

        let mut cache = self.cache.borrow_mut();
        let v = CacheType::Field(fid.clone());
        cache.insert(idx, v);

        fid
    }

    pub fn get_method(&self, idx: usize) -> MethodIdRef {
        let cache = self.cache.borrow();
        let it = cache.get(&idx);
        match it {
            Some(it) => it.extract_method(),
            None => {
                drop(cache);
                self.cache_method(idx)
            }
        }
    }

    fn cache_method(&self, idx: usize) -> MethodIdRef {
        let m = oop::method::get_method_ref(&self.cp, idx).unwrap();

        let mut cache = self.cache.borrow_mut();
        let v = CacheType::Method(m.clone());
        cache.insert(idx, v);

        m
    }
}
