use crate::classfile::attr_info::{LineNumber, TargetInfo, TypeAnnotation};
use crate::classfile::{
    attr_info::{self, AttrTag, AttrType},
    constant_pool::*,
    field_info::FieldInfo,
    method_info::MethodInfo,
    ClassFile, Version,
};
use crate::types::*;
use std::io::{Cursor, Read};
//use std::path::Path;
use std::sync::Arc;
use nom::{named, named_args, do_parse,number::streaming::{be_u8,be_u16,be_u32}, switch, count, take, call, value};

struct Parser {
    buf: Cursor<Vec<U1>>,
}

impl Parser {
    fn new(raw: Vec<U1>) -> Self {
        Self {
            buf: Cursor::new(raw),
        }
    }
}

named!(version<Version>, do_parse!(
    minor: be_u16 >>
    major: be_u16 >>
    (Version {
        minor,
        major,
    })
));

named!(constant_tag<ConstantTag>, do_parse!(
    tag: be_u8 >>
    (ConstantTag::from(tag))
));

named!(cp_entry<ConstantType>, do_parse!(
    ct: constant_tag >>
    entry: switch!(ct,
        ConstantTag::Class => do_parse!(
            name_index: be_u16 >>
            (ConstantType::Class { name_index })
        ) |
        ConstantTag::FieldRef => do_parse!(
            class_index: be_u16 >>
            name_and_type_index: be_u16 >>
            (ConstantType::FieldRef { class_index, name_and_type_index })
        ) |
        ConstantTag::MethodRef => do_parse!(
            class_index: be_u16 >>
            name_and_type_index: be_u16 >>
            (ConstantType::MethodRef { class_index, name_and_type_index })
        ) |
        ConstantTag::InterfaceMethodRef => do_parse!(
            class_index: be_u16 >>
            name_and_type_index: be_u16 >>
            (ConstantType::InterfaceMethodRef { class_index, name_and_type_index })
        ) |
        ConstantTag::String => do_parse!(
            string_index: be_u16 >>
            (ConstantType::String { string_index })
        ) |
        ConstantTag::Integer => do_parse!(
            v: take!(4) >>
            (ConstantType::Integer { v })
        ) |
        ConstantTag::Float => do_parse!(
            v: take!(4) >>
            (ConstantType::Float { v })
        ) |
        ConstantTag::Long => do_parse!(
            v: take!(8) >>
            (ConstantType::Long { v })
        ) |
        ConstantTag::Double => do_parse!(
            v: take!(8) >>
            (ConstantType::Double { v })
        ) |
        ConstantTag::NameAndType => do_parse!(
            name_index: be_u16 >>
            desc_index: be_u16 >>
            (ConstantType::NameAndType { name_index, desc_index })
        ) |
        ConstantTag::Utf8 => do_parse!(
            length: be_u16 >>
            bytes: take!(length) >>
            (ConstantType::Utf8 {length, bytes})
        ) |
        ConstantTag::MethodHandle => do_parse!(
            ref_kind: be_u8 >>
            ref_index: be_u16 >>
            (ConstantType::MethodHandle { ref_kind, ref_index })
        ) |
        ConstantTag::MethodType => do_parse!(
            desc_index: be_u16 >>
            (ConstantType::MethodType { desc_index })
        ) |
        ConstantTag::InvokeDynamic => do_parse!(
            bootstrap_method_attr_index: be_u16 >>
            name_and_type_index: be_u16 >>
            (ConstantType::InvokeDynamic { bootstrap_method_attr_index, name_and_type_index })
        )
    ) >>
    (entry)
));

fn constant_pool(input: &[u8]) -> nom::IResult<&[u8], Vec<ConstantType>> {
    let (mut input, count) = be_u16(input)?;
    let output = Vec::new();
    output.push(ConstantType::Nop);
    for i in 0..count {
        let (new_input, constant_type) = cp_entry(input)?;
        input = new_input;
        match constant_type {
            ConstantType::Long {..} | ConstantType::Double {..} => {
                output.push(ConstantType::Nop);
            }
        }
    }
    Ok((input, output))
}

// fn attr(pool: Vec<ConstantType>) -> impl Fn(&[u8]) -> nom::IResult<&[u8], AttrType> {
//     use nom::value;
//     move |input| {
//         let (input, name_index) = be_u16(input)?;
//         // TODO: Move all expects from this parser to nom error
//         let name = get_utf8(pool, name_index as usize).expect("Missing attr name");
//         let tag = AttrTag::from(name.as_slice());
//         match tag {
//             AttrTag::Invalid => Ok((input, AttrType::Invalid)),
//             AttrTag::ConstantValue => do_parse!(input,
//                 length: be_u32 >> // Assert == 2
//                 constant_value_index: be_u16 >>
//                 (AttrType::ConstantValue {constant_value_index})
//             ),
//             AttrTag::Code => do_parse!(input,
//                 length: self.
//             ),
//         }
//     }
// }

named!(stack_map_frame<attr_info::StackMapFrame>, do_parse!(
    frame_type: be_u8 >>
    inner: switch!(frame_type,
        0..=63 => value!(attr_info::StackMapFrame::Same {offset_delta: frame_type as u16}) |
        64..=127 => do_parse!(
            offset_delta: value!((frame_type-64) as u16) >>
            type_info: call!(verification_type_info, 1) >>
            stack: value!(type_info.remove(0)) >>
            (attr_info::StackMapFrame::SameLocals1StackItem {
                offset_delta,
                stack,
            })
        ) |
        128..=246 => value!(attr_info::StackMapFrame::Reserved) |
        247 => do_parse!(
            offset_delta: be_u16 >>
            type_info: call!(verification_type_info, 1) >>
            stack: value!(type_info.remove(0)) >>
            (attr_info::StackMapFrame::SameLocals1StackItem {
                offset_delta,
                stack,
            })
        ) |
        248..=250 => do_parse!(
            offset_delta: be_u16 >>
            (attr_info::StackMapFrame::Chop {
                offset_delta,
            })
        ) |
        251 => do_parse!(
            offset_delta: be_u16 >>
            (attr_info::StackMapFrame::SameExtended {
                offset_delta
            })
        ) |
        252..=254 => do_parse!(
            offset_delta: be_u16 >>
            locals_count: value!(frame_type - 251) >>
            locals: call!(verification_type_info, locals_count as usize) >>
            (attr_info::StackMapFrame::Append {
                offset_delta,
                locals,
            })
        )
    ) >>
    (inner)
));

named_args!(attr_sized(tag: AttrTag, cp: Vec<ConstantType>)<AttrType>, switch!(tag,
    AttrTag::ConstantValue => do_parse!(
        constant_value_index: be_u16 >>
        (AttrType::ConstantValue {constant_value_index})
    ) |
    AttrTag::Code => do_parse!(
        max_stack: be_u16 >>
        max_locals: be_u16 >>
        len: be_u16 >>
        code: take!(len) >> // TODO: Parse code in same time?)
        exception_count: be_u16 >>
        exceptions: count!(code_exception, exception_count as usize) >>
        attrs: call!(attrs, cp) >>
        (AttrType::Code(attr_info::Code {
            max_stack,
            max_locals,
            code,
            exceptions,
            attrs,
        }))
    ) |
    AttrTag::StackMapTable => do_parse!(
        frame_count: be_u16 >>
        frames: count!(stack_map_frame, frame_count as usize) >>
        (AttrType::StackMapTable { entries: frames })
    ) |
    AttrTag::Exceptions => do_parse!(
        exception_count: be_u16 >>

    )
));

named_args!(attr(cp: Vec<ConstantType>)<AttrType>, do_parse!(
    name_index: be_u16 >>
    name: value!(get_utf8(cp, name_index as usize).expect("Missing name")) >>
    attr_tag: value!(AttrTag::from(name.as_slice())) >>
    attr: switch!(attr_tag,
        AttrTag::Invalid => value!(AttrType::Invalid) |
        _ => do_parse!(
            length: be_u32 >>
            data: take!(length) >>
            inner: call!(attr_sized, attr_tag) >>
            (inner)
        )
    ) >>
    (attr)
));

named_args!(attrs(cp: Vec<ConstantType>)<Vec<AttrType>>, do_parse!(
    attrs_count: be_u16 >>
    attrs: count!(call!(attr, cp), attrs_count as usize) >>
    (attrs)
));

named_args!(field(cp: Vec<ConstantType>)<FieldInfo>, do_parse!(
    acc_flags: be_u16 >>
    name_index: be_u16 >>
    desc_index: be_u16 >>
    attrs: call!(attrs, cp) >>
    (FieldInfo {
        acc_flags,
        name_index,
        desc_index,
        attrs,
    })
));

fn parse_class_file(input: &[u8]) -> nom::IResult<&[u8], ()>{
    use nom::tag;
    do_parse!(input,
        tag!(b"\xCA\xFE\xBA\xBE") >>
        ver_minor: version >>
        cp: constant_pool >>
        acc_flags: be_u16 >>
        this_class: be_u16 >>
        super_class: be_u16 >>
        interfaces_count: be_u16 >>
        interfaces: count!(be_u16, interfaces_count as usize) >>
        fields_count: be_u16 >>
        fields: count!(call!(field, cp), fields_count as usize) >>
    )
}

impl Parser {
    fn parse(&mut self) -> ClassFile {
        parse_class_file(self.buf.get_ref());

        let fields_count = self.get_fields_count();
        let fields = self.get_fields(fields_count, &cp);
        let methods_count = self.get_methods_count();
        let methods = self.get_methods(methods_count, &cp);
        let attrs_count = self.get_attrs_count();
        let attrs = self.get_attrs(attrs_count, &cp);
        //        info!("class attrs = {:?}", attrs);

        ClassFile {
            magic,
            version,
            cp_count,
            cp,
            acc_flags,
            this_class,
            super_class,
            interfaces_count,
            interfaces,
            fields_count,
            fields,
            methods_count,
            methods,
            attrs_count,
            attrs,
        }
    }

    fn get_u4(&mut self) -> U4 {
        self.buf.get_u32()
    }

    fn get_u2(&mut self) -> U2 {
        self.buf.get_u16()
    }

    fn get_u1(&mut self) -> U1 {
        self.buf.get_u8()
    }

    fn get_u1s(&mut self, len: usize) -> Vec<U1> {
        let mut bytes = Vec::with_capacity(len);
        bytes.resize(len, 0);
        let r = self.buf.read_exact(&mut bytes);
        assert!(r.is_ok());
        bytes
    }

    fn get_code_exceptions(&mut self, len: usize) -> Vec<attr_info::CodeException> {
        let mut exceptions = Vec::with_capacity(len);

        for _ in 0..len {
            let start_pc = self.get_u2();
            let end_pc = self.get_u2();
            let handler_pc = self.get_u2();
            let catch_type = self.get_u2();
            let exception = attr_info::CodeException {
                start_pc,
                end_pc,
                handler_pc,
                catch_type,
            };
            exceptions.push(exception);
        }

        exceptions
    }

    fn get_line_nums(&mut self, len: usize) -> Vec<LineNumber> {
        let mut tables = Vec::with_capacity(len);
        for _ in 0..len {
            let start_pc = self.get_u2();
            let number = self.get_u2();
            tables.push(attr_info::LineNumber { start_pc, number });
        }
        tables
    }

    fn get_verification_type_info(&mut self, n: usize) -> Vec<attr_info::VerificationTypeInfo> {
        let mut r = Vec::with_capacity(n);

        for _ in 0..n {
            let v = match self.get_u1() {
                0 => attr_info::VerificationTypeInfo::Top,
                1 => attr_info::VerificationTypeInfo::Integer,
                2 => attr_info::VerificationTypeInfo::Float,
                5 => attr_info::VerificationTypeInfo::Null,
                6 => attr_info::VerificationTypeInfo::UninitializedThis,
                7 => {
                    let cpool_index = self.get_u2();
                    attr_info::VerificationTypeInfo::Object { cpool_index }
                }
                8 => {
                    let offset = self.get_u2();
                    attr_info::VerificationTypeInfo::Uninitialized { offset }
                }
                4 => attr_info::VerificationTypeInfo::Long,
                3 => attr_info::VerificationTypeInfo::Double,
                _ => unreachable!(),
            };

            r.push(v);
        }

        r
    }
}

trait ClassFileParser {
    fn get_fields_count(&mut self) -> U2;
    fn get_fields(&mut self, n: U2, cp: &ConstantPool) -> Vec<FieldInfo>;
    fn get_methods_count(&mut self) -> U2;
    fn get_methods(&mut self, n: U2, cp: &ConstantPool) -> Vec<MethodInfo>;
    fn get_attrs_count(&mut self) -> U2;
    fn get_attrs(&mut self, n: U2, cp: &ConstantPool) -> Vec<AttrType>;
}

impl ClassFileParser for Parser {
    fn get_fields_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_fields(&mut self, n: U2, cp: &ConstantPool) -> Vec<FieldInfo> {
        let mut v = Vec::with_capacity(n as usize);
        for _ in 0..n {
            v.push(self.get_field(cp))
        }
        v
    }

    fn get_methods_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_methods(&mut self, n: U2, cp: &ConstantPool) -> Vec<MethodInfo> {
        let mut v = Vec::with_capacity(n as usize);
        for _ in 0..n {
            v.push(self.get_method(cp));
        }
        v
    }

    fn get_attrs_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_attrs(&mut self, n: U2, cp: &ConstantPool) -> Vec<AttrType> {
        let mut v = Vec::with_capacity(n as usize);
        for _ in 0..n {
            v.push(self.get_attr_type(cp));
        }
        v
    }
}

trait MethodParser {
    fn get_method(&mut self, cp: &ConstantPool) -> MethodInfo;
}

impl MethodParser for Parser {
    fn get_method(&mut self, cp: &ConstantPool) -> MethodInfo {
        let acc_flags = self.get_u2();
        let name_index = self.get_u2();
        let desc_index = self.get_u2();
        let attrs_count = self.get_attrs_count();
        let attrs = self.get_attrs(attrs_count, cp);
        //        info!("method attrs = {:?}", attrs);
        MethodInfo {
            acc_flags,
            name_index,
            desc_index,
            attrs,
        }
    }
}

trait AttrTypeParser {
    fn get_attr_exceptions(&mut self) -> AttrType;
    fn get_attr_inner_classes(&mut self) -> AttrType;
    fn get_attr_enclosing_method(&mut self) -> AttrType;
    fn get_attr_synthetic(&mut self) -> AttrType;
    fn get_attr_signature(&mut self) -> AttrType;
    fn get_attr_source_file(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_source_debug_ext(&mut self) -> AttrType;
    fn get_attr_line_num_table(&mut self) -> AttrType;
    fn get_attr_local_var_table(&mut self) -> AttrType;
    fn get_attr_local_var_type_table(&mut self) -> AttrType;
    fn get_attr_deprecated(&mut self) -> AttrType;
    fn get_attr_rt_vis_annotations(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_rt_in_vis_annotations(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_rt_vis_parameter_annotations(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_rt_in_vis_parameter_annotations(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_rt_vis_type_annotations(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_rt_in_vis_type_annotations(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_annotation_default(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_bootstrap_methods(&mut self) -> AttrType;
    fn get_attr_method_parameters(&mut self) -> AttrType;
    fn get_attr_unknown(&mut self) -> AttrType;
}

trait AttrTypeParserUtils {
    fn get_attr_util_get_annotation(&mut self, cp: &ConstantPool) -> attr_info::AnnotationEntry;
    fn get_attr_util_get_type_annotation(&mut self, cp: &ConstantPool)
        -> attr_info::TypeAnnotation;
    fn get_attr_util_get_target_info(&mut self, target_type: U1) -> attr_info::TargetInfo;
    fn get_attr_util_get_local_var(&mut self) -> attr_info::LocalVariable;
    fn get_attr_util_get_element_val(&mut self, cp: &ConstantPool) -> attr_info::ElementValueType;
}

impl AttrTypeParser for Parser {

    fn get_attr_exceptions(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut exceptions = Vec::with_capacity(n as usize);
        for _ in 0..n {
            exceptions.push(self.get_u2());
        }
        AttrType::Exceptions { exceptions }
    }

    fn get_attr_inner_classes(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut classes = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let inner_class_info_index = self.get_u2();
            let outer_class_info_index = self.get_u2();
            let inner_name_index = self.get_u2();
            let inner_class_access_flags = self.get_u2();
            classes.push(attr_info::InnerClass {
                inner_class_info_index,
                outer_class_info_index,
                inner_name_index,
                inner_class_access_flags,
            });
        }
        AttrType::InnerClasses { classes }
    }

    fn get_attr_enclosing_method(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 4);
        let class_index = self.get_u2();
        let method_index = self.get_u2();
        let em = attr_info::EnclosingMethod {
            class_index,
            method_index,
        };
        AttrType::EnclosingMethod { em }
    }

    fn get_attr_synthetic(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 0);
        AttrType::Synthetic
    }

    fn get_attr_signature(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 2);
        let signature_index = self.get_u2();
        AttrType::Signature { signature_index }
    }

    fn get_attr_source_file(&mut self, _cp: &ConstantPool) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 2);
        let source_file_index = self.get_u2();
        //        let name = get_utf8(cp, source_file_index as usize).unwrap();
        //        println!("src name = {}", String::from_utf8_lossy(name.as_slice()));
        AttrType::SourceFile { source_file_index }
    }

    fn get_attr_source_debug_ext(&mut self) -> AttrType {
        let length = self.get_u4();
        let debug_extension = self.get_u1s(length as usize);
        let debug_extension = Arc::new(debug_extension);
        AttrType::SourceDebugExtension { debug_extension }
    }

    fn get_attr_line_num_table(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let tables = self.get_line_nums(n as usize);
        AttrType::LineNumberTable { tables }
    }

    fn get_attr_local_var_table(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut tables = Vec::with_capacity(n as usize);
        for _ in 0..n {
            tables.push(self.get_attr_util_get_local_var());
        }
        AttrType::LocalVariableTable { tables }
    }

    fn get_attr_local_var_type_table(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut tables = Vec::with_capacity(n as usize);
        for _ in 0..n {
            tables.push(self.get_attr_util_get_local_var());
        }
        AttrType::LocalVariableTypeTable { tables }
    }

    fn get_attr_deprecated(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 0);
        AttrType::Deprecated
    }

    fn get_attr_rt_vis_annotations(&mut self, cp: &ConstantPool) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut annotations = Vec::with_capacity(n as usize);
        for _ in 0..n {
            annotations.push(self.get_attr_util_get_annotation(cp));
        }
        AttrType::RuntimeVisibleAnnotations { annotations }
    }

    fn get_attr_rt_in_vis_annotations(&mut self, cp: &ConstantPool) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut annotations = Vec::with_capacity(n as usize);
        for _ in 0..n {
            annotations.push(self.get_attr_util_get_annotation(cp));
        }
        AttrType::RuntimeInvisibleAnnotations { annotations }
    }

    fn get_attr_rt_vis_parameter_annotations(&mut self, cp: &ConstantPool) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut annotations = Vec::with_capacity(n as usize);
        for _ in 0..n {
            annotations.push(self.get_attr_util_get_annotation(cp));
        }
        AttrType::RuntimeVisibleParameterAnnotations { annotations }
    }

    fn get_attr_rt_in_vis_parameter_annotations(&mut self, cp: &ConstantPool) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut annotations = Vec::with_capacity(n as usize);
        for _ in 0..n {
            annotations.push(self.get_attr_util_get_annotation(cp));
        }
        AttrType::RuntimeInvisibleParameterAnnotations { annotations }
    }

    fn get_attr_rt_vis_type_annotations(&mut self, cp: &ConstantPool) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut annotations = Vec::with_capacity(n as usize);
        for _ in 0..n {
            annotations.push(self.get_attr_util_get_type_annotation(cp));
        }
        AttrType::RuntimeVisibleTypeAnnotations { annotations }
    }

    fn get_attr_rt_in_vis_type_annotations(&mut self, cp: &ConstantPool) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut annotations = Vec::with_capacity(n as usize);
        for _ in 0..n {
            annotations.push(self.get_attr_util_get_type_annotation(cp));
        }
        AttrType::RuntimeInvisibleTypeAnnotations { annotations }
    }

    fn get_attr_annotation_default(&mut self, cp: &ConstantPool) -> AttrType {
        let _length = self.get_u4();
        let default_value = self.get_attr_util_get_element_val(cp);
        AttrType::AnnotationDefault { default_value }
    }

    fn get_attr_bootstrap_methods(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut methods = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let method_ref: U2 = self.get_u2();
            let n_arg: U2 = self.get_u2();
            let mut args = Vec::with_capacity(n_arg as usize);
            for _ in 0..n_arg {
                args.push(self.get_u2());
            }
            methods.push(attr_info::BootstrapMethod { method_ref, args });
        }

        AttrType::BootstrapMethods { n, methods }
    }

    fn get_attr_method_parameters(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u1();
        let mut parameters = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let name_index = self.get_u2();
            let acc_flags = self.get_u2();
            parameters.push(attr_info::MethodParameter {
                name_index,
                acc_flags,
            });
        }

        AttrType::MethodParameters { parameters }
    }

    fn get_attr_unknown(&mut self) -> AttrType {
        let len = self.get_u4();
        let _v = self.get_u1s(len as usize);
        AttrType::Unknown
    }
}

impl AttrTypeParserUtils for Parser {
    fn get_attr_util_get_annotation(&mut self, cp: &ConstantPool) -> attr_info::AnnotationEntry {
        let type_index = self.get_u2();
        let n = self.get_u2();
        let mut pairs = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let name_index = self.get_u2();
            let value = self.get_attr_util_get_element_val(cp);
            pairs.push(attr_info::ElementValuePair { name_index, value });
        }
        let type_name = get_utf8(cp, type_index as usize).expect("Missing type name");
        attr_info::AnnotationEntry { type_name, pairs }
    }

    fn get_attr_util_get_type_annotation(&mut self, cp: &ConstantPool) -> TypeAnnotation {
        let target_type = self.get_u1();
        //Table 4.7.20
        let target_info = self.get_attr_util_get_target_info(target_type);
        let n = self.get_u1();
        let mut target_path = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let type_path_kind = self.get_u1();
            let type_argument_index = self.get_u1();
            target_path.push(attr_info::TypePath {
                type_path_kind,
                type_argument_index,
            });
        }
        let type_index = self.get_u2();
        let n = self.get_u2();
        let mut pairs = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let name_index = self.get_u2();
            let value = self.get_attr_util_get_element_val(cp);
            pairs.push(attr_info::ElementValuePair { name_index, value });
        }
        attr_info::TypeAnnotation {
            target_type,
            target_info,
            target_path,
            type_index,
            pairs,
        }
    }

    fn get_attr_util_get_target_info(&mut self, target_type: U1) -> TargetInfo {
        match target_type {
            0x00 | 0x01 => {
                let type_parameter_index = self.get_u1();
                attr_info::TargetInfo::TypeParameter {
                    type_parameter_index,
                }
            }
            0x10 => {
                let supertype_index = self.get_u2();
                attr_info::TargetInfo::SuperType { supertype_index }
            }
            0x11 | 0x12 => {
                let type_parameter_index = self.get_u1();
                let bound_index = self.get_u1();
                attr_info::TargetInfo::TypeParameterBound {
                    type_parameter_index,
                    bound_index,
                }
            }
            0x13 | 0x14 | 0x15 => attr_info::TargetInfo::Empty,
            0x16 => {
                let formal_parameter_index = self.get_u1();
                attr_info::TargetInfo::FormalParameter {
                    formal_parameter_index,
                }
            }
            0x17 => {
                let throws_type_index = self.get_u2();
                attr_info::TargetInfo::Throws { throws_type_index }
            }
            0x40 | 0x41 => {
                let n = self.get_u2();
                let mut table = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    let start_pc = self.get_u2();
                    let length = self.get_u2();
                    let index = self.get_u2();
                    table.push(attr_info::LocalVarTargetTable {
                        start_pc,
                        length,
                        index,
                    });
                }
                attr_info::TargetInfo::LocalVar { table }
            }
            0x42 => {
                let exception_table_index = self.get_u2();
                attr_info::TargetInfo::Catch {
                    exception_table_index,
                }
            }
            0x43 | 0x44 | 0x45 | 0x46 => {
                let offset = self.get_u2();
                attr_info::TargetInfo::Offset { offset }
            }
            0x47 | 0x48 | 0x49 | 0x4A | 0x4B => {
                let offset = self.get_u2();
                let type_argument_index = self.get_u1();
                attr_info::TargetInfo::TypeArgument {
                    offset,
                    type_argument_index,
                }
            }
            _ => unreachable!(),
        }
    }

    fn get_attr_util_get_local_var(&mut self) -> attr_info::LocalVariable {
        let start_pc = self.get_u2();
        let length = self.get_u2();
        let name_index = self.get_u2();
        let signature_index = self.get_u2();
        let index = self.get_u2();
        attr_info::LocalVariable {
            start_pc,
            length,
            name_index,
            signature_index,
            index,
        }
    }

    fn get_attr_util_get_element_val(&mut self, cp: &ConstantPool) -> attr_info::ElementValueType {
        let tag = self.get_u1();
        match attr_info::ElementValueTag::from(tag) {
            attr_info::ElementValueTag::Byte => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Byte { tag, val_index }
            }
            attr_info::ElementValueTag::Char => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Char { tag, val_index }
            }
            attr_info::ElementValueTag::Double => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Double { tag, val_index }
            }
            attr_info::ElementValueTag::Float => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Float { tag, val_index }
            }
            attr_info::ElementValueTag::Int => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Int { tag, val_index }
            }
            attr_info::ElementValueTag::Long => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Long { tag, val_index }
            }
            attr_info::ElementValueTag::Short => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Short { tag, val_index }
            }
            attr_info::ElementValueTag::Boolean => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::Boolean { tag, val_index }
            }
            attr_info::ElementValueTag::String => {
                let val_index = self.get_u2();
                attr_info::ElementValueType::String { tag, val_index }
            }
            attr_info::ElementValueTag::Enum => {
                let type_index = self.get_u2();
                let val_index = self.get_u2();
                attr_info::ElementValueType::Enum {
                    tag,
                    type_index,
                    val_index,
                }
            }
            attr_info::ElementValueTag::Class => {
                let index = self.get_u2();
                attr_info::ElementValueType::Class { tag, index }
            }
            attr_info::ElementValueTag::Annotation => {
                let value = self.get_attr_util_get_annotation(cp);
                let v = attr_info::AnnotationElementValue { value };
                attr_info::ElementValueType::Annotation(v)
            }
            attr_info::ElementValueTag::Array => {
                let n = self.get_u2();
                let mut values = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    values.push(self.get_attr_util_get_element_val(cp));
                }
                attr_info::ElementValueType::Array { n, values }
            }
            attr_info::ElementValueTag::Unknown => attr_info::ElementValueType::Unknown,
        }
    }
}

/*
pub fn parse<P: AsRef<Path>>(path: P) -> std::io::Result<ClassFile> {
    let buf = util::read(path);
    parse_buf(buf)
}
*/

pub fn parse_buf(buf: Vec<u8>) -> std::io::Result<ClassFile> {
    let mut parser = Parser::new(buf);
    Ok(parser.parse())
}
