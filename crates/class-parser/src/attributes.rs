use std::sync::Arc;

use classfile::attributes::*;
use classfile::constant_pool;
use classfile::{ConstantPool, U2};

use crate::reader::{Error, Reader, Result};

// ── Top-level attribute parsing ──────────────────────────────────────────

pub fn parse_attributes(r: &mut Reader, cp: &ConstantPool) -> Result<Vec<Type>> {
    let count = r.read_u16()?;
    let mut attrs = Vec::with_capacity(count as usize);
    for _ in 0..count {
        attrs.push(parse_attribute(r, cp)?);
    }
    Ok(attrs)
}

fn parse_attribute(r: &mut Reader, cp: &ConstantPool) -> Result<Type> {
    let name_index = r.read_u16()?;
    let name = constant_pool::get_utf8(cp, name_index as usize).clone();
    let length = r.read_u32()?;
    let tag = Tag::from(name.as_slice());

    match tag {
        Tag::ConstantValue => {
            let constant_value_index = r.read_u16()?;
            Ok(Type::ConstantValue {
                constant_value_index,
            })
        }
        Tag::Code => parse_code_attribute(r, cp, length),
        Tag::StackMapTable => parse_stack_map_table(r),
        Tag::Exceptions => parse_exceptions(r),
        Tag::InnerClasses => parse_inner_classes(r),
        Tag::EnclosingMethod => parse_enclosing_method(r),
        Tag::Synthetic => Ok(Type::Synthetic),
        Tag::Signature => {
            let signature_index = r.read_u16()?;
            Ok(Type::Signature { signature_index })
        }
        Tag::SourceFile => {
            let source_file_index = r.read_u16()?;
            Ok(Type::SourceFile { source_file_index })
        }
        Tag::SourceDebugExtension => {
            let debug_extension = r.read_bytes(length as usize)?;
            Ok(Type::SourceDebugExtension {
                debug_extension: Arc::new(debug_extension),
            })
        }
        Tag::LineNumberTable => parse_line_number_table(r),
        Tag::LocalVariableTable => parse_local_variable_table(r),
        Tag::LocalVariableTypeTable => parse_local_variable_type_table(r),
        Tag::Deprecated => Ok(Type::Deprecated),
        Tag::RuntimeVisibleAnnotations => {
            parse_runtime_visible_annotations(r, cp, length)
        }
        Tag::RuntimeInvisibleAnnotations => {
            parse_runtime_invisible_annotations(r, cp, length)
        }
        Tag::RuntimeVisibleParameterAnnotations => {
            parse_runtime_visible_parameter_annotations(r, cp, length)
        }
        Tag::RuntimeInvisibleParameterAnnotations => {
            parse_runtime_invisible_parameter_annotations(r, cp, length)
        }
        Tag::RuntimeVisibleTypeAnnotations => {
            parse_runtime_visible_type_annotations(r, cp, length)
        }
        Tag::RuntimeInvisibleTypeAnnotations => {
            parse_runtime_invisible_type_annotations(r, cp, length)
        }
        Tag::AnnotationDefault => parse_annotation_default(r, cp, length),
        Tag::BootstrapMethods => parse_bootstrap_methods(r),
        Tag::MethodParameters => parse_method_parameters(r),
        Tag::Unknown => {
            r.read_bytes(length as usize)?;
            Ok(Type::Unknown)
        }
    }
}

// ── Code attribute ──────────────────────────────────────────────────────

fn parse_code_attribute(r: &mut Reader, cp: &ConstantPool, _length: u32) -> Result<Type> {
    let max_stack = r.read_u16()?;
    let max_locals = r.read_u16()?;
    let code_length = r.read_u32()?;
    let code = r.read_bytes(code_length as usize)?;
    let exception_count = r.read_u16()?;
    let mut exceptions = Vec::with_capacity(exception_count as usize);
    for _ in 0..exception_count {
        exceptions.push(parse_code_exception(r)?);
    }
    let attrs = parse_attributes(r, cp)?;
    Ok(Type::Code(Code {
        max_stack,
        max_locals,
        code: Arc::new(code),
        exceptions,
        attrs,
    }))
}

fn parse_code_exception(r: &mut Reader) -> Result<CodeException> {
    let start_pc = r.read_u16()?;
    let end_pc = r.read_u16()?;
    let handler_pc = r.read_u16()?;
    let catch_type = r.read_u16()?;
    Ok(CodeException {
        start_pc,
        end_pc,
        handler_pc,
        catch_type,
    })
}

// ── StackMapTable ───────────────────────────────────────────────────────

fn parse_stack_map_table(r: &mut Reader) -> Result<Type> {
    let frame_count = r.read_u16()?;
    let mut entries = Vec::with_capacity(frame_count as usize);
    for _ in 0..frame_count {
        entries.push(parse_stack_map_frame(r)?);
    }
    Ok(Type::StackMapTable { entries })
}

fn parse_stack_map_frame(r: &mut Reader) -> Result<StackMapFrame> {
    let frame_type = r.read_u8()?;
    match frame_type {
        0..=63 => Ok(StackMapFrame::Same {
            tag: frame_type,
            offset_delta: frame_type as U2,
        }),
        64..=127 => {
            let type_info = parse_verification_type_info(r)?;
            Ok(StackMapFrame::SameLocals1StackItem {
                tag: frame_type,
                offset_delta: (frame_type - 64) as U2,
                stack: [type_info],
            })
        }
        128..=246 => Ok(StackMapFrame::Reserved(frame_type)),
        247 => {
            let offset_delta = r.read_u16()?;
            let type_info = parse_verification_type_info(r)?;
            Ok(StackMapFrame::SameLocals1StackItemExtended {
                tag: frame_type,
                offset_delta,
                stack: [type_info],
            })
        }
        248..=250 => {
            let offset_delta = r.read_u16()?;
            Ok(StackMapFrame::Chop {
                tag: frame_type,
                offset_delta,
            })
        }
        251 => {
            let offset_delta = r.read_u16()?;
            Ok(StackMapFrame::SameExtended {
                tag: frame_type,
                offset_delta,
            })
        }
        252..=254 => {
            let offset_delta = r.read_u16()?;
            let locals_count = frame_type - 251;
            let mut locals = Vec::with_capacity(locals_count as usize);
            for _ in 0..locals_count {
                locals.push(parse_verification_type_info(r)?);
            }
            Ok(StackMapFrame::Append {
                tag: frame_type,
                offset_delta,
                locals,
            })
        }
        255 => {
            let offset_delta = r.read_u16()?;
            let locals_count = r.read_u16()?;
            let mut locals = Vec::with_capacity(locals_count as usize);
            for _ in 0..locals_count {
                locals.push(parse_verification_type_info(r)?);
            }
            let stack_count = r.read_u16()?;
            let mut stack = Vec::with_capacity(stack_count as usize);
            for _ in 0..stack_count {
                stack.push(parse_verification_type_info(r)?);
            }
            Ok(StackMapFrame::Full {
                tag: frame_type,
                offset_delta,
                locals,
                stack,
            })
        }
    }
}

fn parse_verification_type_info(r: &mut Reader) -> Result<VerificationTypeInfo> {
    let id = r.read_u8()?;
    match id {
        0 => Ok(VerificationTypeInfo::Top),
        1 => Ok(VerificationTypeInfo::Integer),
        2 => Ok(VerificationTypeInfo::Float),
        3 => Ok(VerificationTypeInfo::Long),
        4 => Ok(VerificationTypeInfo::Double),
        5 => Ok(VerificationTypeInfo::Null),
        6 => Ok(VerificationTypeInfo::UninitializedThis),
        7 => {
            let cpool_index = r.read_u16()?;
            Ok(VerificationTypeInfo::Object { cpool_index })
        }
        8 => {
            let offset = r.read_u16()?;
            Ok(VerificationTypeInfo::Uninitialized { offset })
        }
        _ => Err(Error::BadVerificationType(id)),
    }
}

// ── Simple attributes ───────────────────────────────────────────────────

fn parse_exceptions(r: &mut Reader) -> Result<Type> {
    let exception_count = r.read_u16()?;
    let mut exceptions = Vec::with_capacity(exception_count as usize);
    for _ in 0..exception_count {
        exceptions.push(r.read_u16()?);
    }
    Ok(Type::Exceptions { exceptions })
}

fn parse_inner_classes(r: &mut Reader) -> Result<Type> {
    let class_count = r.read_u16()?;
    let mut classes = Vec::with_capacity(class_count as usize);
    for _ in 0..class_count {
        classes.push(InnerClass {
            inner_class_info_index: r.read_u16()?,
            outer_class_info_index: r.read_u16()?,
            inner_name_index: r.read_u16()?,
            inner_class_access_flags: r.read_u16()?,
        });
    }
    Ok(Type::InnerClasses { classes })
}

fn parse_enclosing_method(r: &mut Reader) -> Result<Type> {
    let class_index = r.read_u16()?;
    let method_index = r.read_u16()?;
    Ok(Type::EnclosingMethod {
        em: EnclosingMethod {
            class_index,
            method_index,
        },
    })
}

fn parse_line_number_table(r: &mut Reader) -> Result<Type> {
    let line_count = r.read_u16()?;
    let mut tables = Vec::with_capacity(line_count as usize);
    for _ in 0..line_count {
        tables.push(LineNumber {
            start_pc: r.read_u16()?,
            number: r.read_u16()?,
        });
    }
    Ok(Type::LineNumberTable { tables })
}

fn parse_local_variable_table(r: &mut Reader) -> Result<Type> {
    let variable_count = r.read_u16()?;
    let mut tables = Vec::with_capacity(variable_count as usize);
    for _ in 0..variable_count {
        tables.push(LocalVariable {
            start_pc: r.read_u16()?,
            length: r.read_u16()?,
            name_index: r.read_u16()?,
            signature_index: r.read_u16()?,
            index: r.read_u16()?,
        });
    }
    Ok(Type::LocalVariableTable { tables })
}

fn parse_local_variable_type_table(r: &mut Reader) -> Result<Type> {
    let variable_count = r.read_u16()?;
    let mut tables = Vec::with_capacity(variable_count as usize);
    for _ in 0..variable_count {
        tables.push(LocalVariable {
            start_pc: r.read_u16()?,
            length: r.read_u16()?,
            name_index: r.read_u16()?,
            signature_index: r.read_u16()?,
            index: r.read_u16()?,
        });
    }
    Ok(Type::LocalVariableTypeTable { tables })
}

fn parse_bootstrap_methods(r: &mut Reader) -> Result<Type> {
    let method_count = r.read_u16()?;
    let mut methods = Vec::with_capacity(method_count as usize);
    for _ in 0..method_count {
        methods.push(BootstrapMethod {
            method_ref: r.read_u16()?,
            args: {
                let arg_count = r.read_u16()?;
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(r.read_u16()?);
                }
                args
            },
        });
    }
    Ok(Type::BootstrapMethods {
        n: method_count,
        methods,
    })
}

fn parse_method_parameters(r: &mut Reader) -> Result<Type> {
    let parameter_count = r.read_u8()?;
    let mut parameters = Vec::with_capacity(parameter_count as usize);
    for _ in 0..parameter_count {
        parameters.push(MethodParameter {
            name_index: r.read_u16()?,
            acc_flags: r.read_u16()?,
        });
    }
    Ok(Type::MethodParameters { parameters })
}

// ── Annotation parsing (needs ConstantPool) ─────────────────────────────

fn parse_annotations_list(r: &mut Reader, cp: &ConstantPool) -> Result<Vec<AnnotationEntry>> {
    let count = r.read_u16()?;
    let mut annotations = Vec::with_capacity(count as usize);
    for _ in 0..count {
        annotations.push(parse_annotation_entry(r, cp)?);
    }
    Ok(annotations)
}

fn parse_type_annotations_list(
    r: &mut Reader,
    cp: &ConstantPool,
) -> Result<Vec<TypeAnnotation>> {
    let count = r.read_u16()?;
    let mut annotations = Vec::with_capacity(count as usize);
    for _ in 0..count {
        annotations.push(parse_type_annotation(r, cp)?);
    }
    Ok(annotations)
}

fn parse_runtime_visible_annotations(
    r: &mut Reader,
    cp: &ConstantPool,
    length: u32,
) -> Result<Type> {
    let raw = r.read_bytes(length as usize)?;
    let raw_ref = Arc::new(raw.clone());
    let mut sub = Reader::new(raw);
    let annotations = parse_annotations_list(&mut sub, cp)?;
    Ok(Type::RuntimeVisibleAnnotations {
        raw: raw_ref,
        annotations,
    })
}

fn parse_runtime_invisible_annotations(
    r: &mut Reader,
    cp: &ConstantPool,
    length: u32,
) -> Result<Type> {
    let raw = r.read_bytes(length as usize)?;
    let raw_ref = Arc::new(raw.clone());
    let mut sub = Reader::new(raw);
    let annotations = parse_annotations_list(&mut sub, cp)?;
    Ok(Type::RuntimeInvisibleAnnotations {
        raw: raw_ref,
        annotations,
    })
}

fn parse_runtime_visible_parameter_annotations(
    r: &mut Reader,
    cp: &ConstantPool,
    length: u32,
) -> Result<Type> {
    let raw = r.read_bytes(length as usize)?;
    let raw_ref = Arc::new(raw.clone());
    let mut sub = Reader::new(raw);
    let annotations = parse_annotations_list(&mut sub, cp)?;
    Ok(Type::RuntimeVisibleParameterAnnotations {
        raw: raw_ref,
        annotations,
    })
}

fn parse_runtime_invisible_parameter_annotations(
    r: &mut Reader,
    cp: &ConstantPool,
    length: u32,
) -> Result<Type> {
    let raw = r.read_bytes(length as usize)?;
    let raw_ref = Arc::new(raw.clone());
    let mut sub = Reader::new(raw);
    let annotations = parse_annotations_list(&mut sub, cp)?;
    Ok(Type::RuntimeInvisibleParameterAnnotations {
        raw: raw_ref,
        annotations,
    })
}

fn parse_runtime_visible_type_annotations(
    r: &mut Reader,
    cp: &ConstantPool,
    length: u32,
) -> Result<Type> {
    let raw = r.read_bytes(length as usize)?;
    let raw_ref = Arc::new(raw.clone());
    let mut sub = Reader::new(raw);
    let annotations = parse_type_annotations_list(&mut sub, cp)?;
    Ok(Type::RuntimeVisibleTypeAnnotations {
        raw: raw_ref,
        annotations,
    })
}

fn parse_runtime_invisible_type_annotations(
    r: &mut Reader,
    cp: &ConstantPool,
    length: u32,
) -> Result<Type> {
    let raw = r.read_bytes(length as usize)?;
    let raw_ref = Arc::new(raw.clone());
    let mut sub = Reader::new(raw);
    let annotations = parse_type_annotations_list(&mut sub, cp)?;
    Ok(Type::RuntimeInvisibleTypeAnnotations {
        raw: raw_ref,
        annotations,
    })
}

fn parse_annotation_default(
    r: &mut Reader,
    cp: &ConstantPool,
    length: u32,
) -> Result<Type> {
    let raw = r.read_bytes(length as usize)?;
    let raw_ref = Arc::new(raw.clone());
    let mut sub = Reader::new(raw);
    let default_value = parse_element_value_type(&mut sub, cp)?;
    Ok(Type::AnnotationDefault {
        raw: raw_ref,
        default_value,
    })
}

// ── Annotation structures ───────────────────────────────────────────────

fn parse_annotation_entry(r: &mut Reader, cp: &ConstantPool) -> Result<AnnotationEntry> {
    let type_index = r.read_u16()?;
    let pair_count = r.read_u16()?;
    let mut pairs = Vec::with_capacity(pair_count as usize);
    for _ in 0..pair_count {
        pairs.push(parse_element_value_pair(r, cp)?);
    }
    let type_name = constant_pool::get_utf8(cp, type_index as usize).clone();
    Ok(AnnotationEntry { type_name, pairs })
}

fn parse_element_value_pair(r: &mut Reader, cp: &ConstantPool) -> Result<ElementValuePair> {
    let name_index = r.read_u16()?;
    let value = parse_element_value_type(r, cp)?;
    Ok(ElementValuePair { name_index, value })
}

fn parse_element_value_type(r: &mut Reader, cp: &ConstantPool) -> Result<ElementValueType> {
    let tag_byte = r.read_u8()?;
    let tag = ElementValueTag::from(tag_byte);
    match tag {
        ElementValueTag::Byte
        | ElementValueTag::Char
        | ElementValueTag::Double
        | ElementValueTag::Float
        | ElementValueTag::Int
        | ElementValueTag::Long
        | ElementValueTag::Short
        | ElementValueTag::Boolean
        | ElementValueTag::String => {
            let val_index = r.read_u16()?;
            Ok(match tag {
                ElementValueTag::Byte => ElementValueType::Byte { val_index },
                ElementValueTag::Char => ElementValueType::Char { val_index },
                ElementValueTag::Double => ElementValueType::Double { val_index },
                ElementValueTag::Float => ElementValueType::Float { val_index },
                ElementValueTag::Int => ElementValueType::Int { val_index },
                ElementValueTag::Long => ElementValueType::Long { val_index },
                ElementValueTag::Short => ElementValueType::Short { val_index },
                ElementValueTag::Boolean => ElementValueType::Boolean { val_index },
                ElementValueTag::String => ElementValueType::String { val_index },
                _ => unreachable!(),
            })
        }
        ElementValueTag::Enum => {
            let type_index = r.read_u16()?;
            let val_index = r.read_u16()?;
            Ok(ElementValueType::Enum {
                type_index,
                val_index,
            })
        }
        ElementValueTag::Class => {
            let index = r.read_u16()?;
            Ok(ElementValueType::Class { index })
        }
        ElementValueTag::Annotation => {
            let value = parse_annotation_entry(r, cp)?;
            Ok(ElementValueType::Annotation(AnnotationElementValue { value }))
        }
        ElementValueTag::Array => {
            let array_size = r.read_u16()?;
            let mut values = Vec::with_capacity(array_size as usize);
            for _ in 0..array_size {
                values.push(parse_element_value_type(r, cp)?);
            }
            Ok(ElementValueType::Array { values })
        }
        ElementValueTag::Unknown => Ok(ElementValueType::Unknown),
    }
}

// ── Type annotation ─────────────────────────────────────────────────────

fn parse_type_annotation(r: &mut Reader, cp: &ConstantPool) -> Result<TypeAnnotation> {
    let target_info = parse_target_info(r)?;
    let target_path_part_count = r.read_u8()?;
    let mut target_path = Vec::with_capacity(target_path_part_count as usize);
    for _ in 0..target_path_part_count {
        target_path.push(TypePath {
            type_path_kind: r.read_u8()?,
            type_argument_index: r.read_u8()?,
        });
    }
    let type_index = r.read_u16()?;
    let pair_count = r.read_u16()?;
    let mut pairs = Vec::with_capacity(pair_count as usize);
    for _ in 0..pair_count {
        pairs.push(parse_element_value_pair(r, cp)?);
    }
    Ok(TypeAnnotation {
        target_info,
        target_path,
        type_index,
        pairs,
    })
}

fn parse_target_info(r: &mut Reader) -> Result<TargetInfo> {
    let target_type = r.read_u8()?;
    match target_type {
        0x00 | 0x01 => {
            let type_parameter_index = r.read_u8()?;
            Ok(TargetInfo::TypeParameter {
                type_parameter_index,
            })
        }
        0x10 => {
            let supertype_index = r.read_u16()?;
            Ok(TargetInfo::SuperType { supertype_index })
        }
        0x11 | 0x12 => {
            let type_parameter_index = r.read_u8()?;
            let bound_index = r.read_u8()?;
            Ok(TargetInfo::TypeParameterBound {
                type_parameter_index,
                bound_index,
            })
        }
        0x13 | 0x14 | 0x15 => Ok(TargetInfo::Empty),
        0x16 => {
            let formal_parameter_index = r.read_u8()?;
            Ok(TargetInfo::FormalParameter {
                formal_parameter_index,
            })
        }
        0x17 => {
            let throws_type_index = r.read_u16()?;
            Ok(TargetInfo::Throws { throws_type_index })
        }
        0x40 | 0x41 => {
            let item_count = r.read_u16()?;
            let mut table = Vec::with_capacity(item_count as usize);
            for _ in 0..item_count {
                table.push(LocalVarTargetTable {
                    start_pc: r.read_u16()?,
                    length: r.read_u16()?,
                    index: r.read_u16()?,
                });
            }
            Ok(TargetInfo::LocalVar { table })
        }
        0x42 => {
            let exception_table_index = r.read_u16()?;
            Ok(TargetInfo::Catch {
                exception_table_index,
            })
        }
        0x43 | 0x44 | 0x45 | 0x46 => {
            let offset = r.read_u16()?;
            Ok(TargetInfo::Offset { offset })
        }
        0x47 | 0x48 | 0x49 | 0x4A | 0x4B => {
            let offset = r.read_u16()?;
            let type_argument_index = r.read_u8()?;
            Ok(TargetInfo::TypeArgument {
                offset,
                type_argument_index,
            })
        }
        _ => Err(Error::BadTargetType(target_type)),
    }
}
