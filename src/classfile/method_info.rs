use crate::classfile::attr_info::{AttrType, Code};
use crate::classfile::constant_pool;
use crate::types::{BytesRef, ConstantPool, U2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub acc_flags: U2,
    pub name_index: U2,
    pub desc_index: U2,
    pub attrs: Vec<AttrType>,
}

impl MethodInfo {
    pub fn get_code(&self) -> Option<Code> {
        for it in self.attrs.iter() {
            match it {
                AttrType::Code(code) => return Some(code.clone()),
                _ => (),
            }
        }

        None
    }

    pub fn get_line_number_table(&self) -> HashMap<U2, U2> {
        let mut hm = HashMap::new();

        for it in self.attrs.iter() {
            match it {
                AttrType::LineNumberTable { tables } => {
                    tables.iter().for_each(|ln| {
                        hm.insert(ln.start_pc, ln.number);
                    });
                }
                _ => (),
            }
        }

        hm
    }

    pub fn get_src_file(&self, cp: &ConstantPool) -> Option<BytesRef> {
        for it in self.attrs.iter() {
            match it {
                AttrType::SourceFile { source_file_index } => {
                    return constant_pool::get_utf8(cp, *source_file_index as usize);
                }
                _ => (),
            }
        }

        None
    }
}
