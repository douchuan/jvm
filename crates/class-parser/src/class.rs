use classfile::ClassFile;

use crate::constant_pool;
use crate::fields;
use crate::methods;
use crate::reader::{Error, Reader, Result};
use crate::attributes;

/// Parse a Java class file from raw bytes.
///
/// Follows JVM spec section 4.1:
/// ```text
/// ClassFile {
///     u4             magic;
///     u2             minor_version;
///     u2             major_version;
///     u2             constant_pool_count;
///     cp_info        constant_pool[constant_pool_count-1];
///     u2             access_flags;
///     u2             this_class;
///     u2             super_class;
///     u2             interfaces_count;
///     u2             interfaces[interfaces_count];
///     u2             fields_count;
///     field_info     fields[fields_count];
///     u2             methods_count;
///     method_info    methods[methods_count];
///     u2             attributes_count;
///     attribute_info attributes[attributes_count];
/// }
/// ```
pub fn parse_class_file(r: &mut Reader) -> Result<ClassFile> {
    // magic
    let magic = r.read_u32()?;
    if magic != 0xCAFEBABE {
        return Err(Error::BadMagic(magic));
    }

    let version = parse_version(r)?;
    let cp = constant_pool::parse_constant_pool(r)?;
    let cp_arc = std::sync::Arc::new(cp);

    let access_flags = r.read_u16()?;
    let this_class = r.read_u16()?;
    let super_class = r.read_u16()?;

    let interfaces = parse_interfaces(r)?;
    let fields = fields::parse_fields(r, &cp_arc)?;
    let methods = methods::parse_methods(r, &cp_arc)?;
    let attrs = attributes::parse_attributes(r, &cp_arc)?;

    Ok(ClassFile {
        version,
        cp: cp_arc,
        acc_flags: access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attrs,
    })
}

fn parse_version(r: &mut Reader) -> Result<classfile::Version> {
    let minor = r.read_u16()?;
    let major = r.read_u16()?;
    Ok(classfile::Version { minor, major })
}

fn parse_interfaces(r: &mut Reader) -> Result<Vec<u16>> {
    let count = r.read_u16()?;
    let mut interfaces = Vec::with_capacity(count as usize);
    for _ in 0..count {
        interfaces.push(r.read_u16()?);
    }
    Ok(interfaces)
}
