use crate::types::BytesRef;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Object(BytesRef),
    Short,
    Boolean,
    Array(BytesRef),
    Void,
}

#[derive(Debug)]
pub struct MethodSignature {
    pub args: Vec<Type>,
    pub retype: Type,
}

impl MethodSignature {
    pub fn new(raw: &[u8]) -> Self {
        let mut ts = parse(raw);
        match ts.pop() {
            Some(retype) => Self { args: ts, retype },
            None => Self::default(),
        }
    }
}

impl Default for MethodSignature {
    fn default() -> Self {
        Self {
            args: Vec::new(),
            retype: Type::Void,
        }
    }
}

pub struct FieldSignature {
    pub field_type: Type,
}

impl FieldSignature {
    pub fn new(raw: &[u8]) -> Self {
        let mut v = parse(raw);
        Self {
            field_type: v.pop().unwrap(),
        }
    }
}

fn parse(raw: &[u8]) -> Vec<Type> {
    enum State {
        One,
        Obj,
        Ary,
    }

    let mut state = State::One;
    let mut types: Vec<Type> = Vec::new();
    let mut buf = Vec::new();

    for v in raw {
        match state {
            State::One => match v {
                b'B' => types.push(Type::Byte),
                b'C' => types.push(Type::Char),
                b'D' => types.push(Type::Double),
                b'F' => types.push(Type::Float),
                b'I' => types.push(Type::Int),
                b'J' => types.push(Type::Long),
                b'S' => types.push(Type::Short),
                b'Z' => types.push(Type::Boolean),
                b'V' => types.push(Type::Void),
                b'L' => {
                    buf.push(v.clone());
                    state = State::Obj;
                }
                b'[' => {
                    buf.push(v.clone());
                    state = State::Ary;
                }
                _ => (),
            },
            State::Obj => match v {
                b';' => {
                    buf.push(v.clone());

                    let v = Vec::from(&buf[..]);
                    let v = Arc::new(v);

                    if buf[0] == b'[' {
                        types.push(Type::Array(v));
                    } else {
                        types.push(Type::Object(v));
                    }

                    buf.clear();
                    state = State::One;
                }
                _ => buf.push(v.clone()),
            },
            State::Ary => match v {
                b'L' => {
                    buf.push(v.clone());
                    state = State::Obj;
                }
                b'[' => buf.push(v.clone()),
                _ => {
                    let t = match v {
                        b'B' => Type::Byte,
                        b'C' => Type::Char,
                        b'D' => Type::Double,
                        b'F' => Type::Float,
                        b'I' => Type::Int,
                        b'J' => Type::Long,
                        b'S' => Type::Short,
                        b'Z' => Type::Boolean,
                        _ => unreachable!("unknown type v={}", v),
                    };

                    let mut v1 = Vec::from(&buf[..]);
                    v1.push(*v);
                    let v = Arc::new(v1);
                    types.push(Type::Array(v));

                    buf.clear();
                    state = State::One;
                }
            },
        }
    }

    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_parse() {
        let args = "()V";
        let ts = vec![Type::Void];
        assert_eq!(parse(args.as_bytes()), ts);

        let args = "([[Ljava/lang/String;)V";
        let ts = vec![
            Type::Array(Arc::new(Vec::from("[[Ljava/lang/String;"))),
            Type::Void,
        ];
        assert_eq!(parse(args.as_bytes()), ts);

        let args = "(BCDFIJSZLjava/lang/Integer;)Ljava/lang/String;";
        let ts = vec![
            Type::Byte,
            Type::Char,
            Type::Double,
            Type::Float,
            Type::Int,
            Type::Long,
            Type::Short,
            Type::Boolean,
            Type::Object(Arc::new(Vec::from("Ljava/lang/Integer;"))),
            Type::Object(Arc::new(Vec::from("Ljava/lang/String;"))),
        ];
        assert_eq!(parse(args.as_bytes()), ts);
    }

    #[test]
    fn t_parse2() {
        let args = "()V";
        let sig = MethodSignature::new(args.as_bytes());
        assert_eq!(sig.args, vec![]);
        assert_eq!(sig.retype, Type::Void);

        let args = "([[Ljava/lang/String;)V";
        let sig = MethodSignature::new(args.as_bytes());
        assert_eq!(
            sig.args,
            vec![Type::Array(Arc::new(Vec::from("[[Ljava/lang/String;")))]
        );
        assert_eq!(sig.retype, Type::Void);

        let args = "(BCDFIJSZLjava/lang/Integer;)Ljava/lang/String;";
        let ts = vec![
            Type::Byte,
            Type::Char,
            Type::Double,
            Type::Float,
            Type::Int,
            Type::Long,
            Type::Short,
            Type::Boolean,
            Type::Object(Arc::new(Vec::from("Ljava/lang/Integer;"))),
            Type::Object(Arc::new(Vec::from("Ljava/lang/String;"))),
        ];
        let sig = MethodSignature::new(args.as_bytes());
        assert_eq!(
            sig.args,
            vec![
                Type::Byte,
                Type::Char,
                Type::Double,
                Type::Float,
                Type::Int,
                Type::Long,
                Type::Short,
                Type::Boolean,
                Type::Object(Arc::new(Vec::from("Ljava/lang/Integer;")))
            ]
        );
        assert_eq!(
            sig.retype,
            Type::Object(Arc::new(Vec::from("Ljava/lang/String;")))
        );
    }

    #[test]
    fn t_parse3() {
        macro_rules! setup_test {
            ($desc: expr, $tp: expr) => {
                let sig = crate::classfile::signature::FieldSignature::new($desc);
                assert_eq!(sig.field_type, $tp);
            };
        }

        setup_test!("B".as_bytes(), Type::Byte);
        setup_test!("C".as_bytes(), Type::Char);
        setup_test!("D".as_bytes(), Type::Double);
        setup_test!("F".as_bytes(), Type::Float);
        setup_test!("I".as_bytes(), Type::Int);
        setup_test!("J".as_bytes(), Type::Long);

        let v = Vec::from("Ljava/lang/Object;");
        let v = Arc::new(v);
        setup_test!("Ljava/lang/Object;".as_bytes(), Type::Object(v));
        setup_test!("S".as_bytes(), Type::Short);
        setup_test!("Z".as_bytes(), Type::Boolean);

        let v = Vec::from("[Ljava/lang/Object;");
        let v = Arc::new(v);
        setup_test!("[Ljava/lang/Object;".as_bytes(), Type::Array(v));

        let v = Vec::from("[[[D");
        let v = Arc::new(v);
        setup_test!("[[[D".as_bytes(), Type::Array(v));
    }
}
