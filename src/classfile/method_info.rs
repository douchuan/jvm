use crate::classfile::attr_info;
use crate::types::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub acc_flags: U2,
    pub name_index: U2,
    pub desc_index: U2,
    pub attrs: Vec<attr_info::AttrType>,
}

impl MethodInfo {
    pub fn get_code(&self) -> Option<attr_info::Code> {
        for it in self.attrs.iter() {
            match it {
                attr_info::AttrType::Code(code) => return Some(code.clone()),
                _ => (),
            }
        }

        None
    }

    pub fn get_line_number_table(&self) -> HashMap<U2, U2> {
        let mut hm = HashMap::new();

        for it in self.attrs.iter() {
            match it {
                attr_info::AttrType::LineNumberTable { tables } => {
                    tables.iter().for_each(|ln| {
                        hm.insert(ln.start_pc, ln.number);
                    });
                }
                _ => (),
            }
        }

        hm
    }
}
