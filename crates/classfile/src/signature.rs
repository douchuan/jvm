use crate::BytesRef;

use nom::bytes::complete::{take, take_till};
use nom::character::complete::{char, one_of};
use nom::combinator::peek;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    error::{ErrorKind, ParseError, VerboseError},
    sequence::delimited,
    Err, IResult,
};

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
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let (_, r) = parse_method(s).unwrap();
        r
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
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let (_, r) = parse_field(s).unwrap();
        r
    }
}

///////////////////////////
//parser
///////////////////////////

fn primitive<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    let (i, t) = one_of("BCDFIJSZV")(i)?;
    Ok((
        i,
        match t {
            'B' => Type::Byte,
            'C' => Type::Char,
            'D' => Type::Double,
            'F' => Type::Float,
            'I' => Type::Int,
            'J' => Type::Long,
            'S' => Type::Short,
            'Z' => Type::Boolean,
            'V' => Type::Void,
            _ => unreachable!(),
        },
    ))
}

fn object_desc<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, BytesRef, E> {
    let (i, desc) = take_till(|c| c == ';')(i)?;
    let (i, _) = tag(";")(i)?;
    let mut buf = Vec::with_capacity(1 + desc.len() + 1);
    buf.push(b'L');
    buf.extend_from_slice(desc.as_bytes());
    buf.push(b';');
    let desc = std::sync::Arc::new(buf);
    Ok((i, desc))
}

fn object<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    let (i, _) = tag("L")(i)?;
    let (i, desc) = object_desc(i)?;
    Ok((i, Type::Object(desc)))
}

fn array<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    let (i, _) = peek(tag("["))(i)?;
    let (i, ary_tags) = take_till(|c| c != '[')(i)?;
    let (mut i, t) = take(1u8)(i)?;

    let mut buf = vec![];
    buf.extend_from_slice(ary_tags.as_bytes());
    match t {
        "L" => {
            let (i2, desc) = object_desc(i)?;
            i = i2;
            buf.extend_from_slice(desc.as_slice());
        }
        v => buf.extend_from_slice(v.as_bytes()),
    }
    let desc = std::sync::Arc::new(buf);
    Ok((i, Type::Array(desc)))
}

fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    alt((primitive, object, array))(i)
}

fn parse_multi<'a, E: ParseError<&'a str>>(
    mut input: &'a str,
) -> IResult<&str, Vec<Type>, E> {
    let it = std::iter::from_fn(move || {
        match parse::<'a, E>(input) {
            // when successful, a nom parser returns a tuple of
            // the remaining input and the output value.
            // So we replace the captured input data with the
            // remaining input, to be parsed on the next call
            Ok((i, o)) => {
                input = i;
                Some(o)
            }
            _ => None,
        }
    });

    let mut args = vec![];
    for v in it {
        args.push(v);
    }

    Ok((input, args))
}

fn parse_method(i: &str) -> IResult<&str, MethodSignature> {
    fn arg0(i: &str) -> IResult<&str, MethodSignature> {
        let (i, _) = tag("()")(i)?;
        let (i, retype) = parse(i)?;
        Ok((
            i,
            MethodSignature {
                args: vec![],
                retype,
            },
        ))
    }

    fn args(i: &str) -> IResult<&str, MethodSignature> {
        let (i_return, i_args) = delimited(char('('), is_not(")"), char(')'))(i)?;
        let (_, args) = parse_multi(i_args)?;
        let (i, retype) = parse(i_return)?;
        Ok((i, MethodSignature { args, retype }))
    }

    alt((arg0, args))(i)
}

fn parse_field(mut i: &str) -> IResult<&str, FieldSignature> {
    let (i, field_type) = parse(i)?;
    Ok((i, FieldSignature { field_type }))
}

#[cfg(test)]
mod tests {
    use super::{parse_field, parse_method};
    use crate::FieldSignature;
    use crate::MethodSignature;
    use crate::SignatureType;
    use std::sync::Arc;

    #[test]
    fn t_method_no_arg() {
        let expected = MethodSignature {
            args: vec![],
            retype: SignatureType::Void,
        };
        let (_, r) = parse_method("()V").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn method_primitive() {
        let table = vec![
            (
                MethodSignature {
                    args: vec![SignatureType::Byte],
                    retype: SignatureType::Void,
                },
                "(B)V",
            ),
            (
                MethodSignature {
                    args: vec![SignatureType::Char],
                    retype: SignatureType::Void,
                },
                "(C)V",
            ),
            (
                MethodSignature {
                    args: vec![SignatureType::Double],
                    retype: SignatureType::Void,
                },
                "(D)V",
            ),
            (
                MethodSignature {
                    args: vec![SignatureType::Float],
                    retype: SignatureType::Void,
                },
                "(F)V",
            ),
            (
                MethodSignature {
                    args: vec![SignatureType::Int],
                    retype: SignatureType::Void,
                },
                "(I)V",
            ),
            (
                MethodSignature {
                    args: vec![SignatureType::Long],
                    retype: SignatureType::Void,
                },
                "(J)V",
            ),
            (
                MethodSignature {
                    args: vec![SignatureType::Short],
                    retype: SignatureType::Void,
                },
                "(S)V",
            ),
            (
                MethodSignature {
                    args: vec![SignatureType::Boolean],
                    retype: SignatureType::Void,
                },
                "(Z)V",
            ),
        ];

        for (expected, desc) in table.iter() {
            let (_, r) = parse_method(desc).unwrap();
            assert_eq!(r.args, expected.args);
            assert_eq!(r.retype, expected.retype);
        }
    }

    #[test]
    fn method_array_object() {
        let expected = MethodSignature {
            args: vec![SignatureType::Array(Arc::new(Vec::from(
                "[[Ljava/lang/String;",
            )))],
            retype: SignatureType::Void,
        };
        let (_, r) = parse_method("([[Ljava/lang/String;)V").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn method_mix() {
        let expected = MethodSignature {
            args: vec![
                SignatureType::Byte,
                SignatureType::Char,
                SignatureType::Double,
                SignatureType::Float,
                SignatureType::Int,
                SignatureType::Long,
                SignatureType::Short,
                SignatureType::Boolean,
                SignatureType::Object(Arc::new(Vec::from("Ljava/lang/Integer;"))),
            ],
            retype: SignatureType::Object(Arc::new(Vec::from("Ljava/lang/String;"))),
        };
        let (_, r) = parse_method("(BCDFIJSZLjava/lang/Integer;)Ljava/lang/String;").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn field() {
        macro_rules! setup_test {
            ($desc: expr, $tp: expr) => {
                let (_, sig) = parse_field($desc).unwrap();
                assert_eq!(sig.field_type, $tp);
            };
        }

        setup_test!("B", SignatureType::Byte);
        setup_test!("C", SignatureType::Char);
        setup_test!("D", SignatureType::Double);
        setup_test!("F", SignatureType::Float);
        setup_test!("I", SignatureType::Int);
        setup_test!("J", SignatureType::Long);

        let v = Vec::from("Ljava/lang/Object;");
        let v = Arc::new(v);
        setup_test!("Ljava/lang/Object;", SignatureType::Object(v));
        setup_test!("S", SignatureType::Short);
        setup_test!("Z", SignatureType::Boolean);

        let v = Vec::from("[Ljava/lang/Object;");
        let v = Arc::new(v);
        setup_test!("[Ljava/lang/Object;", SignatureType::Array(v));

        let v = Vec::from("[[[D");
        let v = Arc::new(v);
        setup_test!("[[[D", SignatureType::Array(v));
    }
}
