use crate::BytesRef;
use std::fmt::Formatter;

#[derive(Clone, PartialEq)]
pub enum Type {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    //the 1st, container class
    //the 2nd, generic class's arg
    //the 3rd, if there is a '+'
    //  Ljava/util/List<Lcom/google/inject/Module;>;)
    //    => java.util.List<com.google.inject.Module>
    //  Ljava/lang/Class<+Lcom/google/inject/Module;>;
    //    => java.lang.Class<? extends com.google.inject.Module>
    Object(BytesRef, Option<Vec<Type>>, Option<u8>),
    Short,
    Boolean,
    Array(BytesRef),
    Void,
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Byte => write!(f, "B"),
            Type::Char => write!(f, "C"),
            Type::Double => write!(f, "D"),
            Type::Float => write!(f, "F"),
            Type::Int => write!(f, "I"),
            Type::Long => write!(f, "J"),
            Type::Object(container, args, prefix) => {
                write!(f, "Object(");
                write!(f, "\"{}\",", String::from_utf8_lossy(container.as_slice()));
                write!(f, "{:?},", args);
                write!(f, "{:?}", prefix);
                write!(f, ")")
            }
            Type::Short => write!(f, "S"),
            Type::Boolean => write!(f, "Z"),
            Type::Array(desc) => write!(f, "Array({})", String::from_utf8_lossy(desc.as_slice())),
            Type::Void => write!(f, "V"),
        }
    }
}