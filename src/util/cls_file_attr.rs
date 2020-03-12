use crate::classfile::attr_info::AttrType;
use crate::types::BytesRef;

pub fn assemble_annotation(attrs: &Vec<AttrType>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            AttrType::RuntimeVisibleAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            AttrType::RuntimeInvisibleAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_param_annotation(attrs: &Vec<AttrType>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            AttrType::RuntimeVisibleParameterAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            AttrType::RuntimeInvisibleParameterAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_type_annotation(attrs: &Vec<AttrType>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            AttrType::RuntimeVisibleTypeAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            AttrType::RuntimeInvisibleTypeAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_annotation_default(attrs: &Vec<AttrType>) -> Option<Vec<u8>> {
    let mut vis = None;

    for it in attrs.iter() {
        match it {
            AttrType::AnnotationDefault { raw, .. } => {
                vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, None)
}

fn do_assemble(vis: Option<BytesRef>, in_vis: Option<BytesRef>) -> Option<Vec<u8>> {
    let mut raw = None;

    match vis {
        Some(v) => {
            raw = Some(Vec::from(v.as_slice()));
        }
        None => (),
    }

    match in_vis {
        Some(v) => {
            let raw = raw.as_mut().unwrap();
            raw.extend_from_slice(v.as_slice());
        }
        None => (),
    }

    raw
}
