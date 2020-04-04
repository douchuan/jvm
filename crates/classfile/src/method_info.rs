use crate::attributes::{Code, LineNumber, Type};
use crate::constant_pool;
use crate::types::{BytesRef, ConstantPool, U2};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub acc_flags: U2,
    pub name_index: U2,
    pub desc_index: U2,
    pub attrs: Vec<Type>,
}

impl MethodInfo {
    pub fn get_code(&self) -> Option<Code> {
        for it in self.attrs.iter() {
            match it {
                Type::Code(code) => return Some(code.clone()),
                _ => (),
            }
        }

        None
    }

    pub fn get_line_number_table(&self) -> Vec<LineNumber> {
        let mut line_num_table = Vec::new();

        for it in self.attrs.iter() {
            match it {
                Type::Code(code) => {
                    for it in code.attrs.iter() {
                        match it {
                            Type::LineNumberTable { tables } => {
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

    pub fn get_exceptions(&self) -> Option<Vec<U2>> {
        for it in self.attrs.iter() {
            match it {
                Type::Exceptions { exceptions } => return Some(exceptions.clone()),
                _ => (),
            }
        }

        None
    }
}
