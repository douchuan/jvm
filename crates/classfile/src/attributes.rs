use crate::{BytesRef, U1, U2, U4};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Type {
    ConstantValue {
        constant_value_index: U2,
    },
    Code(Code),
    StackMapTable {
        entries: Vec<StackMapFrame>,
    },
    Exceptions {
        exceptions: Vec<U2>,
    },
    InnerClasses {
        classes: Vec<InnerClass>,
    },
    EnclosingMethod {
        em: EnclosingMethod,
    },
    Synthetic,
    Signature {
        signature_index: U2,
    },
    SourceFile {
        source_file_index: U2,
    },
    SourceDebugExtension {
        debug_extension: BytesRef,
    },
    LineNumberTable {
        tables: Vec<LineNumber>,
    },
    LocalVariableTable {
        tables: Vec<LocalVariable>,
    },
    LocalVariableTypeTable {
        tables: Vec<LocalVariable>,
    },
    Deprecated,
    RuntimeVisibleAnnotations {
        raw: BytesRef,
        annotations: Vec<AnnotationEntry>,
    },
    RuntimeInvisibleAnnotations {
        raw: BytesRef,
        annotations: Vec<AnnotationEntry>,
    },
    RuntimeVisibleParameterAnnotations {
        raw: BytesRef,
        annotations: Vec<AnnotationEntry>,
    },
    RuntimeInvisibleParameterAnnotations {
        raw: BytesRef,
        annotations: Vec<AnnotationEntry>,
    },
    RuntimeVisibleTypeAnnotations {
        raw: BytesRef,
        annotations: Vec<TypeAnnotation>,
    },
    RuntimeInvisibleTypeAnnotations {
        raw: BytesRef,
        annotations: Vec<TypeAnnotation>,
    },
    AnnotationDefault {
        raw: BytesRef,
        default_value: ElementValueType,
    },
    BootstrapMethods {
        n: U2,
        methods: Vec<BootstrapMethod>,
    },
    MethodParameters {
        parameters: Vec<MethodParameter>,
    },
    Unknown,
}

#[derive(Clone, Copy)]
pub enum Tag {
    ConstantValue,
    Code,
    StackMapTable,
    Exceptions,
    InnerClasses,
    EnclosingMethod,
    Synthetic,
    Signature,
    SourceFile,
    SourceDebugExtension,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    Deprecated,
    RuntimeVisibleAnnotations,
    RuntimeInvisibleAnnotations,
    RuntimeVisibleParameterAnnotations,
    RuntimeInvisibleParameterAnnotations,
    RuntimeVisibleTypeAnnotations,
    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault,
    BootstrapMethods,
    MethodParameters,
    Unknown,
}

impl From<&[u8]> for Tag {
    fn from(raw: &[u8]) -> Self {
        match raw {
            b"ConstantValue" => Tag::ConstantValue,
            b"Code" => Tag::Code,
            b"StackMapTable" => Tag::StackMapTable,
            b"Exceptions" => Tag::Exceptions,
            b"InnerClasses" => Tag::InnerClasses,
            b"EnclosingMethod" => Tag::EnclosingMethod,
            b"Synthetic" => Tag::Synthetic,
            b"Signature" => Tag::Signature,
            b"SourceFile" => Tag::SourceFile,
            b"SourceDebugExtension" => Tag::SourceDebugExtension,
            b"LineNumberTable" => Tag::LineNumberTable,
            b"LocalVariableTable" => Tag::LocalVariableTable,
            b"LocalVariableTypeTable" => Tag::LocalVariableTypeTable,
            b"Deprecated" => Tag::Deprecated,
            b"RuntimeVisibleAnnotations" => Tag::RuntimeVisibleAnnotations,
            b"RuntimeInvisibleAnnotations" => Tag::RuntimeInvisibleAnnotations,
            b"RuntimeVisibleParameterAnnotations" => Tag::RuntimeVisibleParameterAnnotations,
            b"RuntimeInvisibleParameterAnnotations" => Tag::RuntimeInvisibleParameterAnnotations,
            b"RuntimeVisibleTypeAnnotations" => Tag::RuntimeVisibleTypeAnnotations,
            b"RuntimeInvisibleTypeAnnotations" => Tag::RuntimeInvisibleTypeAnnotations,
            b"AnnotationDefault" => Tag::AnnotationDefault,
            b"BootstrapMethods" => Tag::BootstrapMethods,
            b"MethodParameters" => Tag::MethodParameters,
            _ => {
                info!("Unknown attr {}", unsafe {
                    std::str::from_utf8_unchecked(raw)
                });
                // error!("Unknown attr {}", String::from_utf8_lossy(raw));
                Tag::Unknown
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Code {
    pub max_stack: U2,
    pub max_locals: U2,
    pub code: Arc<Vec<U1>>,
    pub exceptions: Vec<CodeException>,
    pub attrs: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct CodeException {
    pub start_pc: U2,
    pub end_pc: U2,
    pub handler_pc: U2,
    pub catch_type: U2,
}

impl CodeException {
    pub fn contains(&self, pc: U2) -> bool {
        (self.start_pc..self.end_pc + 1).contains(&pc)
    }

    pub fn is_finally(&self) -> bool {
        self.catch_type == 0
    }
}

#[derive(Debug, Clone)]
pub struct AttributeInfo {
    pub name_index: U2,
    pub length: U4,
    pub info: Vec<U1>,
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
    pub inner_class_info_index: U2,
    pub outer_class_info_index: U2,
    pub inner_name_index: U2,
    pub inner_class_access_flags: U2,
}

#[derive(Debug, Copy, Clone)]
pub struct LineNumber {
    pub start_pc: U2,
    pub number: U2,
}

#[derive(Debug, Copy, Clone)]
pub struct LocalVariable {
    pub start_pc: U2,
    pub length: U2,
    pub name_index: U2,
    pub signature_index: U2,
    pub index: U2,
}

#[derive(Debug, Clone, Copy)]
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
    Byte { val_index: U2 },
    Char { val_index: U2 },
    Double { val_index: U2 },
    Float { val_index: U2 },
    Int { val_index: U2 },
    Long { val_index: U2 },
    Short { val_index: U2 },
    Boolean { val_index: U2 },
    String { val_index: U2 },
    Enum { type_index: U2, val_index: U2 },
    Class { index: U2 },
    Annotation(AnnotationElementValue),
    Array { values: Vec<ElementValueType> },
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AnnotationElementValue {
    pub value: AnnotationEntry,
}

#[derive(Debug, Clone)]
pub struct ElementValuePair {
    pub name_index: U2,
    pub value: ElementValueType,
}

#[derive(Debug, Clone)]
pub struct AnnotationEntry {
    pub type_name: BytesRef,
    pub pairs: Vec<ElementValuePair>,
}

#[derive(Debug, Clone)]
pub struct BootstrapMethod {
    pub method_ref: U2,
    pub args: Vec<U2>,
}

#[derive(Debug)]
pub enum MethodParameterAccessFlag {
    AccFinal = 0x0010,
    AccSynthetic = 0x1000,
    AccMandated = 0x8000,
}

#[derive(Debug, Copy, Clone)]
pub struct MethodParameter {
    pub name_index: U2,
    pub acc_flags: U2,
}

#[derive(Debug, Clone)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object { cpool_index: U2 },
    Uninitialized { offset: U2 },
}

#[derive(Debug, Clone)]
pub enum StackMapFrame {
    Same {
        tag: U1,
        offset_delta: U2,
    },
    SameLocals1StackItem {
        tag: U1,
        offset_delta: U2,
        stack: [VerificationTypeInfo; 1],
    },
    SameLocals1StackItemExtended {
        tag: U1,
        offset_delta: U2,
        stack: [VerificationTypeInfo; 1],
    },
    Chop {
        tag: U1,
        offset_delta: U2,
    },
    SameExtended {
        tag: U1,
        offset_delta: U2,
    },
    Append {
        tag: U1,
        offset_delta: U2,
        locals: Vec<VerificationTypeInfo>,
    },
    Full {
        tag: U1,
        offset_delta: U2,
        locals: Vec<VerificationTypeInfo>,
        stack: Vec<VerificationTypeInfo>,
    },
    Reserved(U1),
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub target_info: TargetInfo,
    pub target_path: Vec<TypePath>,
    pub type_index: U2,
    pub pairs: Vec<ElementValuePair>,
}

#[derive(Debug, Clone)]
pub enum TargetInfo {
    TypeParameter {
        type_parameter_index: U1,
    },
    SuperType {
        supertype_index: U2,
    },
    TypeParameterBound {
        type_parameter_index: U1,
        bound_index: U1,
    },
    Empty,
    FormalParameter {
        formal_parameter_index: U1,
    },
    Throws {
        throws_type_index: U2,
    },
    LocalVar {
        table: Vec<LocalVarTargetTable>,
    },
    Catch {
        exception_table_index: U2,
    },
    Offset {
        offset: U2,
    },
    TypeArgument {
        offset: U2,
        type_argument_index: U1,
    },
}

#[derive(Debug, Clone)]
pub struct LocalVarTargetTable {
    pub start_pc: U2,
    pub length: U2,
    pub index: U2,
}

#[derive(Debug, Clone)]
pub struct TypePath {
    pub type_path_kind: U1,
    pub type_argument_index: U1,
}

#[derive(Debug, Clone)]
pub struct EnclosingMethod {
    pub class_index: U2,
    pub method_index: U2,
}
