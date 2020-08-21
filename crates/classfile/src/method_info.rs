use crate::attributes::{Code, CodeException, LineNumber, LocalVariable, StackMapFrame, Type};
use crate::constant_pool;
use crate::{BytesRef, ConstantPool, U2};
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
            if let Type::Code(code) = it {
                return Some(code.clone());
            }
        }

        None
    }

    pub fn get_line_number_table(&self) -> Vec<LineNumber> {
        let mut line_num_table = Vec::new();

        self.attrs.iter().for_each(|attr| {
            if let Type::Code(code) = attr {
                code.attrs.iter().for_each(|it| {
                    if let Type::LineNumberTable {tables} = it {
                        line_num_table.extend_from_slice(tables.as_slice());
                    }
                });
            }
        });

        line_num_table
    }

    pub fn get_throws(&self) -> Option<Vec<U2>> {
        for it in self.attrs.iter() {
            if let Type::Exceptions { exceptions } = it {
                return Some(exceptions.clone());
            }
        }

        None
    }

    pub fn get_ex_table(&self) -> Option<Vec<CodeException>> {
        for it in self.attrs.iter() {
            if let Type::Code(code) = it {
                if !code.exceptions.is_empty() {
                    return Some(code.exceptions.clone())
                }
            }
        }

        None
    }

    pub fn get_stack_map_table(&self) -> Option<Vec<StackMapFrame>> {
        if let Some(code) = self.get_code() {
            for it in code.attrs.iter() {
                if let Type::StackMapTable { entries} = it {
                    return Some(entries.clone());
                }
            }
        }

        None
    }

    pub fn get_local_variable_table(&self) -> Option<Vec<LocalVariable>> {
        if let Some(code) = self.get_code() {
            for it in code.attrs.iter() {
                if let Type::LocalVariableTable { tables } = it {
                    return Some(tables.clone());
                }
            }
        }

        None
    }

    pub fn get_local_variable_type_table(&self) -> Option<Vec<LocalVariable>> {
        if let Some(code) = self.get_code() {
            for it in code.attrs.iter() {
                if let Type::LocalVariableTypeTable { tables } = it {
                    return Some(tables.clone());
                }
            }
        }

        None
    }
}
