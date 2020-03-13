use crate::classfile::attributes::{AttrType, Code, LineNumber};
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

    pub fn get_line_number_table(&self) -> Vec<LineNumber> {
        let mut line_num_table = Vec::new();

        for it in self.attrs.iter() {
            match it {
                AttrType::Code(code) => {
                    for it in code.attrs.iter() {
                        match it {
                            AttrType::LineNumberTable { tables } => {
                                line_num_table.extend_from_slice(tables.as_slice());
                            }
                            _ => (),
                        }
                    }
                }

                _ => (),
            }
        }

        line_num_table
    }
}
