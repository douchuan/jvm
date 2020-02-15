use crate::classfile::attr_info::{AttrType, Code};
use crate::types::U2;
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
}
