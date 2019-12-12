use crate::classfile::types::*;

#[derive(Debug, Clone)]
pub enum AttrType {
    Invalid,
    ConstantValue {
        length: U4,
        constant_value_index: U2,
    },
    Code(Code),
    //    StackMapTable,
    Exceptions {
        length: U4,
        exceptions_n: U2,
        exceptions: Vec<U2>,
    },
    InnerClasses {
        length: U4,
        classes_n: U2,
        classes: Vec<InnerClass>,
    },
    EnclosingMethod {
        length: U4,
        class_index: U2,
        method_index: U2,
    },
    Synthetic {
        length: U4,
    },
    Signature {
        length: U4,
        signature_index: U2,
    },
    SourceFile {
        length: U4,
        source_file_index: U2,
    },
    //    SourceDebugExtension(SourceDebugExtension),
    LineNumberTable {
        length: U4,
        tables_n: U2,
        tables: Vec<LineNumber>,
    },
    LocalVariableTable {
        length: U4,
        tables_n: U2,
        tables: Vec<LocalVariable>,
    },
    LocalVariableTypeTable {
        length: U4,
        tables_n: U2,
        tables: Vec<LocalVariable>,
    },
    Deprecated {
        length: U4,
    },
    RuntimeVisibleAnnotations {
        length: U4,
        annotations_n: U2,
        annotations: Vec<AnnotationEntry>,
    },
    RuntimeInvisibleAnnotations {
        length: U4,
        annotations_n: U2,
        annotations: Vec<AnnotationEntry>,
    },
    RuntimeVisibleParameterAnnotations {
        length: U4,
        annotations_n: U2,
        annotations: Vec<AnnotationEntry>,
    },
    RuntimeInvisibleParameterAnnotations {
        length: U4,
        annotations_n: U2,
        annotations: Vec<AnnotationEntry>,
    },
    //    RuntimeVisibleTypeAnnotations,
    //    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault {
        length: U4,
        default_value: ElementValueType,
    },
    BootstrapMethods {
        length: U4,
        methods_n: U2,
        methods: Vec<BootstrapMethod>,
    },
    MethodParameters {
        length: U4,
        parameters_n: U1,
        parameters: Vec<MethodParameter>,
    },
    Unknown
}

pub enum AttrTag {
    Invalid,
    ConstantValue,
    Code,
    //    StackMapTable,
    Exceptions,
    InnerClasses,
    EnclosingMethod,
    Synthetic,
    Signature,
    SourceFile,
    //    SourceDebugExtension,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    Deprecated,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    //    RuntimeVisibleTypeAnnotations,
    //    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault,
    BootstrapMethods,
    MethodParameters,
    Unknown,
}

impl From<&[u8]> for AttrTag {
    fn from(raw: &[u8]) -> Self {
        match raw {
            b"ConstantValue" => AttrTag::ConstantValue,
            b"Code" => AttrTag::Code,
            //            b"StackMapTable" => AttributeTag::StackMapTable,
            b"Exceptions" => AttrTag::Exceptions,
            b"InnerClasses" => AttrTag::InnerClasses,
            b"EnclosingMethod" => AttrTag::EnclosingMethod,
            b"Synthetic" => AttrTag::Synthetic,
            b"Signature" => AttrTag::Signature,
            b"SourceFile" => AttrTag::SourceFile,
            //            b"SourceDebugExtension" => AttrTag::SourceDebugExtension,
            b"LineNumberTable" => AttrTag::LineNumberTable,
            b"LocalVariableTable" => AttrTag::LocalVariableTable,
            b"LocalVariableTypeTable" => AttrTag::LocalVariableTypeTable,
            b"Deprecated" => AttrTag::Deprecated,
            b"RuntimeVisibleAnnotations" => AttrTag::RuntimeVisibleAnnotations,
            b"RuntimeInvisibleAnnotations" => AttrTag::RuntimeInvisibleAnnotations,
            b"RuntimeVisibleParameterAnnotations" => AttrTag::RuntimeVisibleParameterAnnotations,
            b"RuntimeInvisibleParameterAnnotations" => {
                AttrTag::RuntimeInvisibleParameterAnnotations
            }
            //            b"RuntimeVisibleTypeAnnotations" => AttributeTag::RuntimeVisibleTypeAnnotations,
            //            b"RuntimeInvisibleTypeAnnotations" => AttributeTag::RuntimeInvisibleTypeAnnotations,
            b"AnnotationDefault" => AttrTag::AnnotationDefault,
            b"BootstrapMethods" => AttrTag::BootstrapMethods,
            b"MethodParameters" => AttrTag::MethodParameters,
            _ => {
                warn!("Unknown attr {}", String::from_utf8_lossy(raw));
                AttrTag::Unknown
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Code {
    length: U4,
    pub max_stack: U2,
    pub max_locals: U2,
    code_n: U4,
    pub code: Vec<U1>,
    pub exceptions_n: U2,
    pub exceptions: Vec<CodeException>,
    attrs_n: U2,
    attrs: Vec<AttrType>,
}

impl Code {
    pub fn new(
        length: U4,
        max_stack: U2,
        max_locals: U2,
        code_n: U4,
        code: Vec<U1>,
        exceptions_n: U2,
        exceptions: Vec<CodeException>,
        attrs_n: U2,
        attrs: Vec<AttrType>,
    ) -> Self {
        Self {
            length,
            max_stack,
            max_locals,
            code_n,
            code,
            exceptions_n,
            exceptions,
            attrs_n,
            attrs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeException {
    start_pc: U2,
    end_pc: U2,
    pub handler_pc: U2,
    pub catch_type: U2,
}

impl CodeException {
    pub fn new(start_pc: U2, end_pc: U2, handler_pc: U2, catch_type: U2) -> Self {
        Self {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        }
    }

    pub fn contains(&self, pc: U2) -> bool {
        (self.start_pc..self.end_pc).contains(&pc)
    }

    pub fn is_finally(&self) -> bool {
        self.catch_type == 0
    }
}

pub enum NestedClassAccessPropertyFlag {
    AccPublic,
    AccPrivate,
    AccProtected,
    AccStatic,
    AccFinal,
    AccInterface,
    AccAbstract,
    AccSynthetic,
    AccAnnotation,
    AccEnum,
}

#[derive(Debug, Copy, Clone)]
pub struct InnerClass {
    inner_class_info_index: U2,
    outer_class_info_index: U2,
    inner_name_index: U2,
    inner_class_access_flags: U2,
}

impl InnerClass {
    pub fn new(
        inner_class_info_index: U2,
        outer_class_info_index: U2,
        inner_name_index: U2,
        inner_class_access_flags: U2,
    ) -> Self {
        Self {
            inner_class_info_index,
            outer_class_info_index,
            inner_name_index,
            inner_class_access_flags,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LineNumber {
    start_pc: U2,
    number: U2,
}

impl LineNumber {
    pub fn new(start_pc: U2, number: U2) -> Self {
        Self { start_pc, number }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LocalVariable {
    start_pc: U2,
    length: U2,
    name_index: U2,
    signature_index: U2,
    index: U2,
}

impl LocalVariable {
    pub fn new(start_pc: U2, length: U2, name_index: U2, signature_index: U2, index: U2) -> Self {
        Self {
            start_pc,
            length,
            name_index,
            signature_index,
            index,
        }
    }
}

#[derive(Debug)]
pub enum ElementValueTag {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    String,
    Enum,
    Class,
    Annotation,
    Array,
    Unknown,
}

impl From<u8> for ElementValueTag {
    fn from(v: u8) -> Self {
        match v {
            b'B' => ElementValueTag::Byte,
            b'C' => ElementValueTag::Char,
            b'D' => ElementValueTag::Double,
            b'F' => ElementValueTag::Float,
            b'I' => ElementValueTag::Int,
            b'J' => ElementValueTag::Long,
            b'S' => ElementValueTag::Short,
            b'Z' => ElementValueTag::Boolean,
            b's' => ElementValueTag::String,
            b'e' => ElementValueTag::Enum,
            b'c' => ElementValueTag::Class,
            b'@' => ElementValueTag::Annotation,
            b'[' => ElementValueTag::Array,
            _ => ElementValueTag::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElementValueType {
    Byte {tag: U1, val_index: U2},
    Char {tag: U1, val_index: U2},
    Double {tag: U1, val_index: U2},
    Float {tag: U1, val_index: U2},
    Int {tag: U1, val_index: U2},
    Long {tag: U1, val_index: U2},
    Short {tag: U1, val_index: U2},
    Boolean {tag: U1, val_index: U2},
    String {tag: U1, val_index: U2},
    Enum {tag: U1, type_index: U2, val_index: U2},
    Class {tag: U1, index: U2},
    Annotation(AnnotationElementValue),
    Array {n: U2, values: Vec<ElementValueType>},
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AnnotationElementValue {
    value: AnnotationEntry,
}

impl AnnotationElementValue {
    pub fn new(value: AnnotationEntry) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone)]
pub struct ElementValuePair {
    name_index: U2,
    value: ElementValueType,
}

impl ElementValuePair {
    pub fn new(name_index: U2, value: ElementValueType) -> Self {
        Self { name_index, value }
    }
}

#[derive(Debug, Clone)]
pub struct AnnotationEntry {
    type_index: U2,
    pairs_n: U2,
    pairs: Vec<ElementValuePair>,
}

impl AnnotationEntry {
    pub fn new(type_index: U2, pairs_n: U2, pairs: Vec<ElementValuePair>) -> Self {
        Self {
            type_index,
            pairs_n,
            pairs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapMethod {
    method_ref: U2,
    arguments_n: U2,
    arguments: Vec<U2>,
}

impl BootstrapMethod {
    pub fn new(method_ref: U2, arguments_n: U2, arguments: Vec<U2>) -> Self {
        Self {
            method_ref,
            arguments_n,
            arguments,
        }
    }
}

#[derive(Debug)]
pub enum MethodParameterAccessFlag {
    AccFinal = 0x0010,
    AccSynthetic = 0x1000,
    AccMandated = 0x8000,
}

#[derive(Debug, Copy, Clone)]
pub struct MethodParameter {
    name_index: U2,
    acc_flags: U2,
}

impl MethodParameter {
    pub fn new(name_index: U2, acc_flags: U2) -> Self {
        Self { name_index, acc_flags }
    }
}
