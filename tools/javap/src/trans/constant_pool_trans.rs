use classfile::constant_pool::{self, Type};
use classfile::{ClassFile, ConstantPoolType};

pub struct Translator<'a> {
    pub cf: &'a ClassFile,
}

impl<'a> Translator<'a> {
    pub fn get(&self) -> Vec<String> {
        let mut pool = Vec::with_capacity(self.cf.cp.len());

        for (pos, it) in self.cf.cp.iter().enumerate() {
            let pos = format!("#{}", pos);

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
                    class_index: _,
                    name_and_type_index: _,
                } => {}
                Type::String { string_index: _ } => {}
                Type::Integer { v: _ } => {}
                Type::Float { v: _ } => {}
                Type::Long { v: _ } => {}
                Type::Double { v: _ } => {}
                Type::NameAndType {
                    name_index: _,
                    desc_index: _,
                } => {}
                Type::Utf8 { bytes } => {
                    let v = format!(
                        "{:>6} = {:18} {}",
                        pos,
                        "Utf8",
                        String::from_utf8_lossy(bytes.as_slice())
                    );
                    pool.push(v);
                }
                Type::MethodHandle {
                    ref_kind: _,
                    ref_index: _,
                } => {}
                Type::MethodType { desc_index: _ } => {}
                Type::InvokeDynamic {
                    bootstrap_method_attr_index: _,
                    name_and_type_index: _,
                } => {}
                Type::Unknown => (),
            }
        }

        pool
    }
}
