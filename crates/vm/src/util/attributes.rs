use classfile::{AttributeType, BytesRef};

pub fn assemble_annotation(attrs: &[AttributeType]) -> Option<Vec<u8>> {
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

pub fn assemble_param_annotation(attrs: &[AttributeType]) -> Option<Vec<u8>> {
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

pub fn assemble_type_annotation(attrs: &[AttributeType]) -> Option<Vec<u8>> {
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

pub fn assemble_annotation_default(attrs: &[AttributeType]) -> Option<Vec<u8>> {
    let mut vis = None;

    for it in attrs.iter() {
        if let AttributeType::AnnotationDefault { raw, .. } = it {
            vis = Some(raw.clone());
        }
    }

    do_assemble(vis, None)
}

pub fn get_signature(attrs: &[AttributeType]) -> u16 {
    for it in attrs.iter() {
        if let AttributeType::Signature { signature_index } = it {
            return *signature_index;
        }
    }

    0
}

fn do_assemble(vis: Option<BytesRef>, in_vis: Option<BytesRef>) -> Option<Vec<u8>> {
    let mut raw = None;

    if let Some(v) = vis {
        raw = Some(Vec::from(v.as_slice()));
    }

    if let Some(v) = in_vis {
        if let Some(raw) = raw.as_mut() {
            raw.extend_from_slice(v.as_slice());
        }
    }

    raw
}
