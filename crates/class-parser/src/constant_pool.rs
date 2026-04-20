use std::sync::Arc;

use crate::reader::{Reader, Result};
use classfile::constant_pool::Tag;
use classfile::constant_pool::Type;

pub fn parse_constant_pool(r: &mut Reader) -> Result<Vec<Type>> {
    let count = r.read_u16()?;
    let mut pool = Vec::with_capacity(count as usize);

    // JVM spec 4.4: constant pool index 0 is unused
    pool.push(Type::Nop);

    let mut i = 1u16;
    while i < count {
        let entry = parse_cp_entry(r)?;

        let is_wide = matches!(entry, Type::Long { .. } | Type::Double { .. });

        pool.push(entry);

        if is_wide {
            // Long/Double occupy two constant pool entries (spec 4.4.5)
            pool.push(Type::Nop);
            i += 2;
        } else {
            i += 1;
        }
    }

    Ok(pool)
}

fn parse_cp_entry(r: &mut Reader) -> Result<Type> {
    let tag_byte = r.read_u8()?;
    let tag = Tag::from(tag_byte);

    match tag {
        Tag::Class => {
            let name_index = r.read_u16()?;
            Ok(Type::Class { name_index })
        }
        Tag::FieldRef => {
            let class_index = r.read_u16()?;
            let name_and_type_index = r.read_u16()?;
            Ok(Type::FieldRef {
                class_index,
                name_and_type_index,
            })
        }
        Tag::MethodRef => {
            let class_index = r.read_u16()?;
            let name_and_type_index = r.read_u16()?;
            Ok(Type::MethodRef {
                class_index,
                name_and_type_index,
            })
        }
        Tag::InterfaceMethodRef => {
            let class_index = r.read_u16()?;
            let name_and_type_index = r.read_u16()?;
            Ok(Type::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            })
        }
        Tag::String => {
            let string_index = r.read_u16()?;
            Ok(Type::String { string_index })
        }
        Tag::Integer => {
            let v = read_exact_4(r)?;
            Ok(Type::Integer { v })
        }
        Tag::Float => {
            let v = read_exact_4(r)?;
            Ok(Type::Float { v })
        }
        Tag::Long => {
            let v = read_exact_8(r)?;
            Ok(Type::Long { v })
        }
        Tag::Double => {
            let v = read_exact_8(r)?;
            Ok(Type::Double { v })
        }
        Tag::NameAndType => {
            let name_index = r.read_u16()?;
            let desc_index = r.read_u16()?;
            Ok(Type::NameAndType {
                name_index,
                desc_index,
            })
        }
        Tag::Utf8 => {
            let bytes = r.read_utf8()?;
            Ok(Type::Utf8 {
                bytes: Arc::new(bytes),
            })
        }
        Tag::MethodHandle => {
            let ref_kind = r.read_u8()?;
            let ref_index = r.read_u16()?;
            Ok(Type::MethodHandle {
                ref_kind,
                ref_index,
            })
        }
        Tag::MethodType => {
            let desc_index = r.read_u16()?;
            Ok(Type::MethodType { desc_index })
        }
        Tag::InvokeDynamic => {
            let bootstrap_method_attr_index = r.read_u16()?;
            let name_and_type_index = r.read_u16()?;
            Ok(Type::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            })
        }
    }
}

fn read_exact_4(r: &mut Reader) -> Result<[u8; 4]> {
    let mut buf = [0; 4];
    for b in &mut buf {
        *b = r.read_u8()?;
    }
    Ok(buf)
}

fn read_exact_8(r: &mut Reader) -> Result<[u8; 8]> {
    let mut buf = [0; 8];
    for b in &mut buf {
        *b = r.read_u8()?;
    }
    Ok(buf)
}
