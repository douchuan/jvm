use classfile::MethodInfo;
use classfile::ConstantPool;

use crate::attributes;
use crate::reader::{Reader, Result};

pub fn parse_methods(r: &mut Reader, cp: &ConstantPool) -> Result<Vec<MethodInfo>> {
    let count = r.read_u16()?;
    let mut methods = Vec::with_capacity(count as usize);
    for _ in 0..count {
        methods.push(parse_method(r, cp)?);
    }
    Ok(methods)
}

fn parse_method(r: &mut Reader, cp: &ConstantPool) -> Result<MethodInfo> {
    let acc_flags = r.read_u16()?;
    let name_index = r.read_u16()?;
    let desc_index = r.read_u16()?;
    let attrs = attributes::parse_attributes(r, cp)?;
    Ok(MethodInfo {
        acc_flags,
        name_index,
        desc_index,
        attrs,
    })
}
