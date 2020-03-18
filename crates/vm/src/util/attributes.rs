use class_parser::{types::BytesRef, AttributeType};

pub fn assemble_annotation(attrs: &Vec<AttributeType>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            AttributeType::RuntimeVisibleAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            AttributeType::RuntimeInvisibleAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_param_annotation(attrs: &Vec<AttributeType>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            AttributeType::RuntimeVisibleParameterAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            AttributeType::RuntimeInvisibleParameterAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_type_annotation(attrs: &Vec<AttributeType>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            AttributeType::RuntimeVisibleTypeAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            AttributeType::RuntimeInvisibleTypeAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_annotation_default(attrs: &Vec<AttributeType>) -> Option<Vec<u8>> {
    let mut vis = None;

    for it in attrs.iter() {
        match it {
            AttributeType::AnnotationDefault { raw, .. } => {
                vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, None)
}

pub fn get_signature(attrs: &Vec<AttributeType>) -> u16 {
    for it in attrs.iter() {
        match it {
            AttributeType::Signature { signature_index } => {
                return *signature_index;
            }
            _ => (),
        }
    }

    0
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
