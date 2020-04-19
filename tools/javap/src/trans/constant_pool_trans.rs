use classfile::constant_pool::{self, Type};
use classfile::{ClassFile, ConstantPoolType};

pub struct Translator<'a> {
    pub cf: &'a ClassFile,
}

impl<'a> Translator<'a> {
    pub fn get(&self) -> Vec<String> {
        let mut pool = Vec::with_capacity(self.cf.cp.len());

        for (cp_idx, it) in self.cf.cp.iter().enumerate() {
            let pos = format!("#{}", cp_idx);

            match it {
                ConstantPoolType::Nop => (),
                Type::Class { name_index } => {
                    let index = format!("#{}", *name_index);
                    let name = constant_pool::get_utf8(&self.cf.cp, *name_index as usize).unwrap();
                    let v = format!(
                        "{:>6} = {:18} {:14} // {}",
                        pos,
                        "Class",
                        index,
                        String::from_utf8_lossy(name.as_slice())
                    );
                    pool.push(v);
                }
                Type::FieldRef {
                    class_index,
                    name_and_type_index,
                } => {
                    let index = format!("#{}.#{}", *class_index, *name_and_type_index);
                    let class_name =
                        constant_pool::get_class_name(&self.cf.cp, *class_index as usize).unwrap();
                    let (name, desc) = constant_pool::get_name_and_type(
                        &self.cf.cp,
                        *name_and_type_index as usize,
                    );
                    let name = name.unwrap();
                    let desc = desc.unwrap();

                    let class_name = String::from_utf8_lossy(class_name.as_slice());
                    let name = String::from_utf8_lossy(name.as_slice());
                    let desc = String::from_utf8_lossy(desc.as_slice());

                    let v = format!(
                        "{:>6} = {:18} {:14} // {}.{}:{}",
                        pos, "Fieldref", index, class_name, name, desc
                    );

                    pool.push(v);
                }
                Type::MethodRef {
                    class_index,
                    name_and_type_index,
                } => {
                    let index = format!("#{}.#{}", *class_index, *name_and_type_index);
                    let class_name =
                        constant_pool::get_class_name(&self.cf.cp, *class_index as usize).unwrap();
                    let (name, desc) = constant_pool::get_name_and_type(
                        &self.cf.cp,
                        *name_and_type_index as usize,
                    );
                    let name = name.unwrap();
                    let desc = desc.unwrap();

                    let class_name = String::from_utf8_lossy(class_name.as_slice());
                    let name = String::from_utf8_lossy(name.as_slice());
                    let desc = String::from_utf8_lossy(desc.as_slice());
                    let v = if name.as_bytes() == b"<init>" {
                        format!(
                            "{:>6} = {:18} {:14} // {}.\"<init>\":{}",
                            pos, "Methodref", index, class_name, desc
                        )
                    } else {
                        format!(
                            "{:>6} = {:18} {:14} // {}.{}:{}",
                            pos, "Methodref", index, class_name, name, desc
                        )
                    };

                    pool.push(v);
                }
                Type::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                } => {
                    let index = format!("#{}.#{}", *class_index, *name_and_type_index);
                    let class_name =
                        constant_pool::get_class_name(&self.cf.cp, *class_index as usize).unwrap();
                    let (name, desc) = constant_pool::get_name_and_type(
                        &self.cf.cp,
                        *name_and_type_index as usize,
                    );
                    let name = name.unwrap();
                    let desc = desc.unwrap();

                    let class_name = String::from_utf8_lossy(class_name.as_slice());
                    let name = String::from_utf8_lossy(name.as_slice());
                    let desc = String::from_utf8_lossy(desc.as_slice());
                    let v = format!(
                        "{:>6} = {:18} {:14} // {}.{}:{}",
                        pos, "InterfaceMethodref", index, class_name, name, desc
                    );

                    pool.push(v);
                }
                Type::String { string_index } => {
                    let index = format!("#{}", *string_index);
                    let constant_val = constant_pool::get_string(&self.cf.cp, cp_idx).unwrap();
                    let v = format!(
                        "{:>6} = {:18} {:14} // {}",
                        pos,
                        "String",
                        index,
                        constant_val.escape_default()
                    );

                    pool.push(v);
                }
                Type::Integer { v } => {
                    let value = i32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                    let v = format!("{:>6} = {:18} {}", pos, "Int", value);

                    pool.push(v);
                }
                Type::Float { v } => {
                    let value = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                    let value = f32::from_bits(value);
                    let v = format!("{:>6} = {:18} {}f", pos, "Float", value);

                    pool.push(v);
                }
                Type::Long { v } => {
                    let value =
                        i64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                    let v = format!("{:>6} = {:18} {}l", pos, "Long", value,);

                    pool.push(v);
                }
                Type::Double { v } => {
                    let value =
                        u64::from_be_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]]);
                    let value = f64::from_bits(value);

                    let v = format!("{:>6} = {:18} {}d", pos, "Double", value,);

                    pool.push(v);
                }
                Type::NameAndType {
                    name_index,
                    desc_index,
                } => {
                    let index = format!("#{}.#{}", *name_index, *desc_index);
                    let name = constant_pool::get_utf8(&self.cf.cp, *name_index as usize).unwrap();
                    let is_ctor = name.as_slice() == b"<init>";
                    let name = String::from_utf8_lossy(name.as_ref());
                    let desc = constant_pool::get_utf8(&self.cf.cp, *desc_index as usize).unwrap();
                    let desc = String::from_utf8_lossy(desc.as_ref());

                    let v = if is_ctor {
                        format!(
                            "{:>6} = {:18} {:14} // \"{}\":{}",
                            pos, "NameAndType", index, name, desc
                        )
                    } else {
                        format!(
                            "{:>6} = {:18} {:14} // {}:{}",
                            pos, "NameAndType", index, name, desc
                        )
                    };

                    pool.push(v);
                }
                Type::Utf8 { bytes } => {
                    let v = format!(
                        "{:>6} = {:18} {}",
                        pos,
                        "Utf8",
                        String::from_utf8_lossy(bytes.as_slice()).escape_default()
                    );
                    pool.push(v);
                }
                Type::MethodHandle {
                    ref_kind: _,
                    ref_index: _,
                } => {
                    pool.push("todo: MethodType".to_string());
                }
                Type::MethodType { desc_index: _ } => {
                    pool.push("todo: MethodType".to_string());
                }
                Type::InvokeDynamic {
                    bootstrap_method_attr_index: _,
                    name_and_type_index: _,
                } => {
                    pool.push("todo: InvokeDynamic".to_string());
                }
                Type::Unknown => (),
            }
        }

        pool
    }
}
