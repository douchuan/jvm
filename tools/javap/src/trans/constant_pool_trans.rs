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
                } => {
                    pool.push("todo: InterfaceMethodRef".to_string());
                }
                Type::String { string_index } => {
                    let index = format!("#{}", *string_index);
                    let bytes =
                        constant_pool::get_utf8(&self.cf.cp, *string_index as usize).unwrap();
                    let string = build_constant_string(bytes.as_slice());
                    let v = format!(
                        "{:>6} = {:18} {:14} // {}",
                        pos,
                        "String",
                        index,
                        String::from_utf16_lossy(string.as_slice())
                    );

                    pool.push(v);
                }
                Type::Integer { v } => {
                    let value = i32::from_be_bytes([v[0], v[1], v[2], v[3]]);
                    let v = format!("{:>6} = {:18} {}f", pos, "Int", value);

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
                    let v = format!("{:>6} = {:18} {}f", pos, "Long", value,);

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
                    let name = String::from_utf8_lossy(name.as_ref());
                    let desc = constant_pool::get_utf8(&self.cf.cp, *desc_index as usize).unwrap();
                    let desc = String::from_utf8_lossy(desc.as_ref());

                    let v = format!(
                        "{:>6} = {:18} {:14} // {}:{}",
                        pos, "NameAndType", index, name, desc
                    );

                    pool.push(v);
                }
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

pub fn build_constant_string(bs: &[u8]) -> Vec<u16> {
    let length = bs.len();
    let mut buffer: Vec<u16> = Vec::with_capacity(length);
    let mut pos = 0;
    while pos < length {
        if bs[pos] & 0x80 == 0 {
            let v = bs[pos] as u16;
            buffer.push(v);
            pos += 1;
        } else if bs[pos] & 0xE0 == 0xC0 && (bs[pos + 1] & 0xC0) == 0x80 {
            let x = bs[pos] as u16;
            let y = bs[pos + 1] as u16;
            let v = ((x & 0x1f) << 6) + (y & 0x3f);
            buffer.push(v);
            pos += 2;
        } else if bs[pos] & 0xF0 == 0xE0
            && (bs[pos + 1] & 0xC0) == 0x80
            && (bs[pos + 2] & 0xC0) == 0x80
        {
            let x = bs[pos] as u16;
            let y = bs[pos + 1] as u16;
            let z = bs[pos + 2] as u16;
            let v = ((x & 0xf) << 12) + ((y & 0x3f) << 6) + (z & 0x3f);
            buffer.push(v);
            pos += 3;
        } else if bs[pos] == 0xED
            && (bs[pos + 1] & 0xF0 == 0xA0)
            && (bs[pos + 2] & 0xC0 == 0x80)
            && (bs[pos + 3] == 0xED)
            && (bs[pos + 4] & 0xF0 == 0xB0)
            && (bs[pos + 5] & 0xC0 == 0x80)
        {
            let v = bs[pos + 1] as u32;
            let w = bs[pos + 2] as u32;
            let y = bs[pos + 4] as u32;
            let z = bs[pos + 5] as u32;
            let vv =
                0x10000 + ((v & 0x0f) << 16) + ((w & 0x3f) << 10) + ((y & 0x0f) << 6) + (z & 0x3f);
            buffer.push(vv as u16);

            pos += 6;
        } else {
            unreachable!()
        }
    }

    buffer
}
