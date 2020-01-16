use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::Arc;

use bytes::Buf;

use crate::classfile::{
    attr_info::{self, AttrTag, AttrType},
    constant_pool::*,
    field_info::FieldInfo,
    method_info::MethodInfo,
    types::*,
    ClassFile, Version,
};
use crate::util;
use crate::classfile::attr_info::LineNumber;

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

impl Parser {
    fn parse(&mut self) -> ClassFile {
        let magic = self.get_magic();
        let version = self.get_version();
        let cp_count = self.get_cp_count();
        let cp = self.get_cp(cp_count);
        let acc_flags = self.get_acc_flags();
        let this_class = self.get_this_class();
        let super_class = self.get_super_class();
        let interfaces_count = self.get_interface_count();
        let interfaces = self.get_interfaces(interfaces_count);
        let fields_count = self.get_fields_count();
        let fields = self.get_fields(fields_count, &cp);
        let methods_count = self.get_methods_count();
        let methods = self.get_methods(methods_count, &cp);
        let attrs_count = self.get_attrs_count();
        let attrs = self.get_attrs(attrs_count, &cp);

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
        self.buf.get_u32_be()
    }

    fn get_u2(&mut self) -> U2 {
        self.buf.get_u16_be()
    }

    fn get_u1(&mut self) -> U1 {
        self.buf.get_u8()
    }

    fn get_u1s(&mut self, len: usize) -> Vec<U1> {
        let mut bytes = Vec::with_capacity(len);
        unsafe { bytes.set_len(len) }
        let _ = self.buf.read_exact(&mut bytes);
        bytes
    }

    fn get_code_exceptions(&mut self, len: usize) -> Vec<attr_info::CodeException> {
        let mut exceptions = Vec::new();

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
        let mut tables = Vec::new();
        for _ in 0..len {
            let start_pc = self.get_u2();
            let number = self.get_u2();
            tables.push(attr_info::LineNumber { start_pc, number });
        }
        tables
    }
}

trait ClassFileParser {
    fn get_magic(&mut self) -> U4;
    fn get_version(&mut self) -> Version;
    fn get_cp_count(&mut self) -> U2;
    fn get_cp(&mut self, n: U2) -> ConstantPool;
    fn get_acc_flags(&mut self) -> U2;
    fn get_this_class(&mut self) -> U2;
    fn get_super_class(&mut self) -> U2;
    fn get_interface_count(&mut self) -> U2;
    fn get_interfaces(&mut self, n: U2) -> Vec<U2>;
    fn get_fields_count(&mut self) -> U2;
    fn get_fields(&mut self, n: U2, cp: &ConstantPool) -> Vec<FieldInfo>;
    fn get_methods_count(&mut self) -> U2;
    fn get_methods(&mut self, n: U2, cp: &ConstantPool) -> Vec<MethodInfo>;
    fn get_attrs_count(&mut self) -> U2;
    fn get_attrs(&mut self, n: U2, cp: &ConstantPool) -> Vec<AttrType>;
}

impl ClassFileParser for Parser {
    fn get_magic(&mut self) -> U4 {
        self.get_u4()
    }

    fn get_version(&mut self) -> Version {
        let minor = self.get_u2();
        let major = self.get_u2();
        Version { minor, major }
    }

    fn get_cp_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_cp(&mut self, n: U2) -> ConstantPool {
        let mut v = Vec::new();

        v.push(ConstantType::NOP);

        let mut i = 1;
        while i < n {
            let tag = self.get_u1();
            let tag = ConstantTag::from(tag);
            let vv = match tag {
                ConstantTag::Class => self.get_constant_class(),
                ConstantTag::FieldRef => self.get_constant_field_ref(),
                ConstantTag::MethodRef => self.get_constant_method_ref(),
                ConstantTag::InterfaceMethodRef => self.get_constant_interface_method_ref(),
                ConstantTag::String => self.get_constant_string(),
                ConstantTag::Integer => self.get_constant_integer(),
                ConstantTag::Float => self.get_constant_float(),
                ConstantTag::Long => {
                    i += 1;
                    self.get_constant_long()
                },
                ConstantTag::Double => {
                    i += 1;
                    self.get_constant_double()
                },
                ConstantTag::NameAndType => self.get_constant_name_and_type(),
                ConstantTag::Utf8 => self.get_constant_utf8(),
                ConstantTag::MethodHandle => self.get_constant_method_handle(),
                ConstantTag::MethodType => self.get_constant_method_type(),
                ConstantTag::InvokeDynamic => self.get_constant_invoke_dynamic(),
                ConstantTag::Unknown => ConstantType::Unknown,
            };

            i += 1;

            v.push(vv);
        }

        v
    }

    fn get_acc_flags(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_this_class(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_super_class(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_interface_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_interfaces(&mut self, n: U2) -> Vec<U2> {
        let mut v = Vec::new();
        for _ in 0..n {
            v.push(self.get_u2())
        }
        v
    }

    fn get_fields_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_fields(&mut self, n: U2, cp: &ConstantPool) -> Vec<FieldInfo> {
        let mut v = Vec::new();
        for _ in 0..n {
            v.push(self.get_field(cp))
        }
        v
    }

    fn get_methods_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_methods(&mut self, n: U2, cp: &ConstantPool) -> Vec<MethodInfo> {
        let mut v = Vec::new();
        for _ in 0..n {
            v.push(self.get_method(cp));
        }
        v
    }

    fn get_attrs_count(&mut self) -> U2 {
        self.get_u2()
    }

    fn get_attrs(&mut self, n: U2, cp: &ConstantPool) -> Vec<AttrType> {
        let mut v = Vec::new();
        for _ in 0..n {
            v.push(self.get_attr_type(cp));
        }
        v
    }
}

trait ConstantPoolParser {
    fn get_constant_class(&mut self) -> ConstantType;
    fn get_constant_field_ref(&mut self) -> ConstantType;
    fn get_constant_method_ref(&mut self) -> ConstantType;
    fn get_constant_interface_method_ref(&mut self) -> ConstantType;
    fn get_constant_string(&mut self) -> ConstantType;
    fn get_constant_integer(&mut self) -> ConstantType;
    fn get_constant_float(&mut self) -> ConstantType;
    fn get_constant_long(&mut self) -> ConstantType;
    fn get_constant_double(&mut self) -> ConstantType;
    fn get_constant_name_and_type(&mut self) -> ConstantType;
    fn get_constant_utf8(&mut self) -> ConstantType;
    fn get_constant_method_handle(&mut self) -> ConstantType;
    fn get_constant_method_type(&mut self) -> ConstantType;
    fn get_constant_invoke_dynamic(&mut self) -> ConstantType;
}

impl ConstantPoolParser for Parser {
    fn get_constant_class(&mut self) -> ConstantType {
        ConstantType::Class {
            name_index: self.get_u2(),
        }
    }

    fn get_constant_field_ref(&mut self) -> ConstantType {
        ConstantType::FieldRef {
            class_index: self.get_u2(),
            name_and_type_index: self.get_u2(),
        }
    }

    fn get_constant_method_ref(&mut self) -> ConstantType {
        ConstantType::MethodRef {
            class_index: self.get_u2(),
            name_and_type_index: self.get_u2(),
        }
    }

    fn get_constant_interface_method_ref(&mut self) -> ConstantType {
        ConstantType::InterfaceMethodRef {
            class_index: self.get_u2(),
            name_and_type_index: self.get_u2(),
        }
    }

    fn get_constant_string(&mut self) -> ConstantType {
        ConstantType::String {
            string_index: self.get_u2(),
        }
    }

    fn get_constant_integer(&mut self) -> ConstantType {
        let mut v = [0; 4];
        let _ = self.buf.read_exact(&mut v);
        ConstantType::Integer { v }
    }

    fn get_constant_float(&mut self) -> ConstantType {
        let mut v = [0; 4];
        let _ = self.buf.read_exact(&mut v);
        ConstantType::Float { v }
    }

    fn get_constant_long(&mut self) -> ConstantType {
        let mut v = [0; 8];
        let _ = self.buf.read_exact(&mut v);
        ConstantType::Long { v }
    }

    fn get_constant_double(&mut self) -> ConstantType {
        let mut v = [0; 8];
        let _ = self.buf.read_exact(&mut v);
        ConstantType::Double { v }
    }

    fn get_constant_name_and_type(&mut self) -> ConstantType {
        ConstantType::NameAndType {
            name_index: self.get_u2(),
            desc_index: self.get_u2(),
        }
    }

    fn get_constant_utf8(&mut self) -> ConstantType {
        let length = self.get_u2();
        let bytes = self.get_u1s(length as usize);
        let bytes = Arc::new(bytes);
        ConstantType::Utf8 { length, bytes }
    }

    fn get_constant_method_handle(&mut self) -> ConstantType {
        ConstantType::MethodHandle {
            ref_kind: self.get_u1(),
            ref_index: self.get_u2(),
        }
    }

    fn get_constant_method_type(&mut self) -> ConstantType {
        ConstantType::MethodType {
            desc_index: self.get_u2(),
        }
    }

    fn get_constant_invoke_dynamic(&mut self) -> ConstantType {
        ConstantType::InvokeDynamic {
            bootstrap_method_attr_index: self.get_u2(),
            name_and_type_index: self.get_u2(),
        }
    }
}

trait FieldParser {
    fn get_field(&mut self, cp: &ConstantPool) -> FieldInfo;
}

impl FieldParser for Parser {
    fn get_field(&mut self, cp: &ConstantPool) -> FieldInfo {
        let acc_flags = self.get_u2();
        let name_index = self.get_u2();
        let desc_index = self.get_u2();
        let attrs_count = self.get_attrs_count();
        let attrs = self.get_attrs(attrs_count, cp);
        //        println!("field name = {}", String::from_utf8_lossy(get_utf8(name_index, cp).unwrap()));
        //        println!("field desc = {}", String::from_utf8_lossy(get_utf8(desc_index, cp).unwrap()));
        //        println!("{:?}", attrs);
        FieldInfo {
            acc_flags,
            name_index,
            desc_index,
            attrs_count,
            attrs,
        }
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

        //        println!("method name = {}", String::from_utf8_lossy(get_utf8(name_index, cp).unwrap()));
        //        println!("method desc = {}", String::from_utf8_lossy(get_utf8(desc_index, cp).unwrap()));
//                println!("attrs: {:?}", attrs);

        MethodInfo {
            acc_flags,
            name_index,
            desc_index,
            attrs_count,
            attrs,
        }
    }
}

trait AttrTypeParser {
    fn get_attr_type(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_constant_value(&mut self) -> AttrType;
    fn get_attr_code(&mut self, cp: &ConstantPool) -> AttrType;
    fn get_attr_exceptions(&mut self) -> AttrType;
    fn get_attr_inner_classes(&mut self) -> AttrType;
    fn get_attr_enclosing_method(&mut self) -> AttrType;
    fn get_attr_synthetic(&mut self) -> AttrType;
    fn get_attr_signature(&mut self) -> AttrType;
    fn get_attr_source_file(&mut self) -> AttrType;
    fn get_attr_line_num_table(&mut self) -> AttrType;
    fn get_attr_local_var_table(&mut self) -> AttrType;
    fn get_attr_local_var_type_table(&mut self) -> AttrType;
    fn get_attr_deprecated(&mut self) -> AttrType;
    fn get_attr_rt_vis_annotations(&mut self) -> AttrType;
    fn get_attr_rt_in_vis_annotations(&mut self) -> AttrType;
    fn get_attr_rt_vis_parameter_annotations(&mut self) -> AttrType;
    fn get_attr_rt_in_vis_parameter_annotations(&mut self) -> AttrType;
    fn get_attr_annotation_default(&mut self) -> AttrType;
    fn get_attr_bootstrap_methods(&mut self) -> AttrType;
    fn get_attr_method_parameters(&mut self) -> AttrType;
    fn get_attr_unknown(&mut self) -> AttrType;
}

trait AttrTypeParserUtils {
    fn get_attr_util_get_annotation(&mut self) -> attr_info::AnnotationEntry;
    fn get_attr_util_get_local_var(&mut self) -> attr_info::LocalVariable;
    fn get_attr_util_get_element_val(&mut self) -> attr_info::ElementValueType;
}

impl AttrTypeParser for Parser {
    fn get_attr_type(&mut self, cp: &ConstantPool) -> AttrType {
        let name_index = self.get_u2();
        let tag = match cp.get(name_index as usize) {
            Some(v) => match v {
                ConstantType::Utf8 { length: _, bytes } => AttrTag::from(bytes.as_slice()),
                _ => AttrTag::Unknown,
            },
            _ => unreachable!(),
        };

//        let tag = AttrTag::Unknown;
        match tag {
            AttrTag::Invalid => AttrType::Invalid,
            AttrTag::ConstantValue => self.get_attr_constant_value(),
            AttrTag::Code => self.get_attr_code(cp),
            AttrTag::Exceptions => self.get_attr_exceptions(),
            AttrTag::InnerClasses => self.get_attr_inner_classes(),
            AttrTag::EnclosingMethod => self.get_attr_enclosing_method(),
            AttrTag::Synthetic => self.get_attr_synthetic(),
            AttrTag::Signature => self.get_attr_signature(),
            AttrTag::SourceFile => self.get_attr_source_file(),
            AttrTag::LineNumberTable => self.get_attr_line_num_table(),
            AttrTag::LocalVariableTable => self.get_attr_local_var_table(),
            AttrTag::LocalVariableTypeTable => self.get_attr_local_var_type_table(),
            AttrTag::Deprecated => self.get_attr_deprecated(),
            AttrTag::RuntimeVisibleAnnotations => self.get_attr_rt_vis_annotations(),
            AttrTag::RuntimeInvisibleAnnotations => self.get_attr_rt_in_vis_annotations(),
            AttrTag::RuntimeVisibleParameterAnnotations => {
                self.get_attr_rt_vis_parameter_annotations()
            }
            AttrTag::RuntimeInvisibleParameterAnnotations => {
                self.get_attr_rt_in_vis_parameter_annotations()
            }
            AttrTag::AnnotationDefault => self.get_attr_annotation_default(),
            AttrTag::BootstrapMethods => self.get_attr_bootstrap_methods(),
            AttrTag::MethodParameters => self.get_attr_method_parameters(),
            AttrTag::Unknown => self.get_attr_unknown(),
        }
    }

    fn get_attr_constant_value(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 2);
        let constant_value_index = self.get_u2();
        AttrType::ConstantValue {
            constant_value_index,
        }
    }

    fn get_attr_code(&mut self, cp: &ConstantPool) -> AttrType {
        let length = self.get_u4();
        let max_stack = self.get_u2();
        let max_locals = self.get_u2();
        let code_len = self.get_u4();
        let code = self.get_u1s(code_len as usize);
        let code = Arc::new(code);
        let exceptions_n = self.get_u2();
        let exceptions = self.get_code_exceptions(exceptions_n as usize);
        let attrs_n = self.get_u2();
        let mut attrs = Vec::new();
        for _ in 0..attrs_n {
            attrs.push(self.get_attr_type(cp));
        }

        AttrType::Code(attr_info::Code {
            length,
            max_stack,
            max_locals,
            code_len,
            code,
            exceptions_n,
            exceptions,
            attrs_n,
            attrs,
        })
    }

    fn get_attr_exceptions(&mut self) -> AttrType {
        let _length = self.get_u4();
        let exceptions_n = self.get_u2();
        let mut exceptions = Vec::new();
        for _ in 0..exceptions_n {
            exceptions.push(self.get_u2());
        }
        AttrType::Exceptions {
            exceptions_n,
            exceptions,
        }
    }

    fn get_attr_inner_classes(&mut self) -> AttrType {
        let _length = self.get_u4();
        let classes_n = self.get_u2();
        let mut classes = Vec::new();
        for _ in 0..classes_n {
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
        AttrType::InnerClasses {
            classes_n,
            classes,
        }
    }

    fn get_attr_enclosing_method(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 4);
        let class_index = self.get_u2();
        let method_index = self.get_u2();
        AttrType::EnclosingMethod {
            class_index,
            method_index,
        }
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
        AttrType::Signature {
            signature_index,
        }
    }

    fn get_attr_source_file(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 2);
        let source_file_index = self.get_u2();
        AttrType::SourceFile {
            source_file_index,
        }
    }

    fn get_attr_line_num_table(&mut self) -> AttrType {
        let _length = self.get_u4();
        let tables_n = self.get_u2();
        let tables = self.get_line_nums(tables_n  as usize);
        AttrType::LineNumberTable {
            tables_n,
            tables,
        }
    }

    fn get_attr_local_var_table(&mut self) -> AttrType {
        let _length = self.get_u4();
        let tables_n = self.get_u2();
        let mut tables = Vec::new();
        for _ in 0..tables_n {
            tables.push(self.get_attr_util_get_local_var());
        }
        AttrType::LocalVariableTable {
            tables_n,
            tables,
        }
    }

    fn get_attr_local_var_type_table(&mut self) -> AttrType {
        let _length = self.get_u4();
        let tables_n = self.get_u2();
        let mut tables = Vec::new();
        for _ in 0..tables_n {
            tables.push(self.get_attr_util_get_local_var());
        }
        AttrType::LocalVariableTypeTable {
            tables_n,
            tables,
        }
    }

    fn get_attr_deprecated(&mut self) -> AttrType {
        let length = self.get_u4();
        assert_eq!(length, 0);
        AttrType::Deprecated
    }

    fn get_attr_rt_vis_annotations(&mut self) -> AttrType {
        let _length = self.get_u4();
        let annotations_n = self.get_u2();
        let mut annotations = Vec::new();
        for _ in 0..annotations_n {
            annotations.push(self.get_attr_util_get_annotation());
        }
        AttrType::RuntimeVisibleAnnotations {
            annotations_n,
            annotations,
        }
    }

    fn get_attr_rt_in_vis_annotations(&mut self) -> AttrType {
        let _length = self.get_u4();
        let annotations_n = self.get_u2();
        let mut annotations = Vec::new();
        for _ in 0..annotations_n {
            annotations.push(self.get_attr_util_get_annotation());
        }
        AttrType::RuntimeInvisibleAnnotations {
            annotations_n,
            annotations,
        }
    }

    fn get_attr_rt_vis_parameter_annotations(&mut self) -> AttrType {
        let _length = self.get_u4();
        let annotations_n = self.get_u2();
        let mut annotations = Vec::new();
        for _ in 0..annotations_n {
            annotations.push(self.get_attr_util_get_annotation());
        }
        AttrType::RuntimeVisibleParameterAnnotations {
            annotations_n,
            annotations,
        }
    }

    fn get_attr_rt_in_vis_parameter_annotations(&mut self) -> AttrType {
        let _length = self.get_u4();
        let annotations_n = self.get_u2();
        let mut annotations = Vec::new();
        for _ in 0..annotations_n {
            annotations.push(self.get_attr_util_get_annotation());
        }
        AttrType::RuntimeInvisibleParameterAnnotations {
            annotations_n,
            annotations,
        }
    }

    fn get_attr_annotation_default(&mut self) -> AttrType {
        let _length = self.get_u4();
        let default_value = self.get_attr_util_get_element_val();
        AttrType::AnnotationDefault {
            default_value,
        }
    }

    fn get_attr_bootstrap_methods(&mut self) -> AttrType {
        let _length = self.get_u4();
        let n = self.get_u2();
        let mut methods = Vec::new();
        for _ in 0..n {
            let method_ref: U2 = self.get_u2();
            let n_arg: U2 = self.get_u2();
            let mut args = Vec::new();
            for _ in 0..n_arg {
                args.push(self.get_u2());
            }
            methods.push(attr_info::BootstrapMethod {
                method_ref,
                n_arg,
                args,
            });
        }

        AttrType::BootstrapMethods { n, methods }
    }

    fn get_attr_method_parameters(&mut self) -> AttrType {
        let _length = self.get_u4();
        let parameters_n = self.get_u1();
        let mut parameters = Vec::new();
        for _ in 0..parameters_n {
            let name_index = self.get_u2();
            let acc_flags = self.get_u2();
            parameters.push(attr_info::MethodParameter {
                name_index,
                acc_flags,
            });
        }

        AttrType::MethodParameters {
            parameters_n,
            parameters,
        }
    }

    fn get_attr_unknown(&mut self) -> AttrType {
        let len = self.get_u4();
        let _v = self.get_u1s(len as usize);
        info!("get_attr_unknown len={}, _v.len = {}", len, _v.len());
        AttrType::Unknown
    }
}

impl AttrTypeParserUtils for Parser {
    fn get_attr_util_get_annotation(&mut self) -> attr_info::AnnotationEntry {
        let type_index = self.get_u2();
        let pairs_n = self.get_u2();
        let mut pairs = Vec::new();
        for _ in 0..pairs_n {
            let name_index = self.get_u2();
            let value = self.get_attr_util_get_element_val();
            pairs.push(attr_info::ElementValuePair { name_index, value });
        }
        attr_info::AnnotationEntry {
            type_index,
            pairs_n,
            pairs,
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

    fn get_attr_util_get_element_val(&mut self) -> attr_info::ElementValueType {
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
                let value = self.get_attr_util_get_annotation();
                let v = attr_info::AnnotationElementValue { value };
                attr_info::ElementValueType::Annotation(v)
            }
            attr_info::ElementValueTag::Array => {
                let n = self.get_u2();
                let mut values = Vec::new();
                for _ in 0..n {
                    values.push(self.get_attr_util_get_element_val());
                }
                attr_info::ElementValueType::Array { n, values }
            }
            attr_info::ElementValueTag::Unknown => attr_info::ElementValueType::Unknown,
        }
    }
}

pub fn parse<P: AsRef<Path>>(path: P) -> std::io::Result<ClassFile> {
    let buf = util::read(path);
    parse_buf(buf)
}

pub fn parse_buf(buf: Vec<u8>) -> std::io::Result<ClassFile> {
    let mut parser = Parser::new(buf);
    Ok(parser.parse())
}
