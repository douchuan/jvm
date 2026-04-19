use classfile::FieldInfo;
use classfile::ConstantPool;

use crate::attributes;
use crate::reader::{Reader, Result};

pub fn parse_fields(r: &mut Reader, cp: &ConstantPool) -> Result<Vec<FieldInfo>> {
    let count = r.read_u16()?;
    let mut fields = Vec::with_capacity(count as usize);
    for _ in 0..count {
        fields.push(parse_field(r, cp)?);
    }
    Ok(fields)
}

fn parse_field(r: &mut Reader, cp: &ConstantPool) -> Result<FieldInfo> {
    let acc_flags = r.read_u16()?;
    let name_index = r.read_u16()?;
    let desc_index = r.read_u16()?;
    let attrs = attributes::parse_attributes(r, cp)?;
    Ok(FieldInfo {
        acc_flags,
        name_index,
        desc_index,
        attrs,
    })
}
