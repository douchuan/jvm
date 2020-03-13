use crate::classfile::attributes::Type;
use crate::types::BytesRef;

pub fn assemble_annotation(attrs: &Vec<Type>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            Type::RuntimeVisibleAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            Type::RuntimeInvisibleAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_param_annotation(attrs: &Vec<Type>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            Type::RuntimeVisibleParameterAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            Type::RuntimeInvisibleParameterAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_type_annotation(attrs: &Vec<Type>) -> Option<Vec<u8>> {
    let mut vis = None;
    let mut in_vis = None;

    for it in attrs.iter() {
        match it {
            Type::RuntimeVisibleTypeAnnotations { raw, .. } => {
                vis = Some(raw.clone());
            }
            Type::RuntimeInvisibleTypeAnnotations { raw, .. } => {
                in_vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, in_vis)
}

pub fn assemble_annotation_default(attrs: &Vec<Type>) -> Option<Vec<u8>> {
    let mut vis = None;

    for it in attrs.iter() {
        match it {
            Type::AnnotationDefault { raw, .. } => {
                vis = Some(raw.clone());
            }
            _ => (),
        }
    }

    do_assemble(vis, None)
}

pub fn get_signature(attrs: &Vec<Type>) -> u16 {
    for it in attrs.iter() {
        match it {
            Type::Signature { signature_index } => {
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
