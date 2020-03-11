use crate::classfile::attr_info::AttrType;

pub fn assemble_annotation(attrs: &Vec<AttrType>) -> Vec<u8> {
    let mut raw_annotation = Vec::new();

    for it in attrs.iter() {
        match it {
            AttrType::RuntimeVisibleAnnotations { raw, .. } => {
                raw_annotation.extend_from_slice(raw.as_slice());
            }
            AttrType::RuntimeInvisibleAnnotations { raw, ..} => {
                raw_annotation.extend_from_slice(raw.as_slice());
            }
            _ => (),
        }
    }

    raw_annotation
}

pub fn assemble_param_annotation(attrs: &Vec<AttrType>) -> Vec<u8> {
    let mut raw_annotation = Vec::new();

    for it in attrs.iter() {
        match it {
            AttrType::RuntimeVisibleParameterAnnotations { raw, .. } => {
                raw_annotation.extend_from_slice(raw.as_slice());
            }
            AttrType::RuntimeInvisibleParameterAnnotations { raw, ..} => {
                raw_annotation.extend_from_slice(raw.as_slice());
            }
            _ => (),
        }
    }

    raw_annotation
}

pub fn assemble_type_annotation(attrs: &Vec<AttrType>) -> Vec<u8> {
    let mut raw_annotation = Vec::new();

    for it in attrs.iter() {
        match it {
            AttrType::RuntimeVisibleTypeAnnotations { raw, .. } => {
                raw_annotation.extend_from_slice(raw.as_slice());
            }
            AttrType::RuntimeInvisibleTypeAnnotations { raw, ..} => {
                raw_annotation.extend_from_slice(raw.as_slice());
            }
            _ => (),
        }
    }

    raw_annotation
}

pub fn assemble_annotation_default(attrs: &Vec<AttrType>) -> Vec<u8> {
    let mut raw_annotation = Vec::new();

    for it in attrs.iter() {
        match it {
            AttrType::AnnotationDefault { raw, .. } => {
                raw_annotation.extend_from_slice(raw.as_slice());
            }
            _ => (),
        }
    }

    raw_annotation
}
