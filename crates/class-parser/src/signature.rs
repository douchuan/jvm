use classfile::BytesRef;
use classfile::SignatureType as Type;

use nom::bytes::complete::{take, take_till};
use nom::character::complete::{char, one_of};
use nom::combinator::{peek, verify};
use nom::error::make_error;
use nom::lib::std::fmt::{Error, Formatter};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    error::{ErrorKind, ParseError, VerboseError},
    multi::many1,
    sequence::delimited,
    AsBytes, Err, IResult,
};

#[derive(Debug)]
pub struct ClassSignature {
    pub items: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct MethodSignature {
    /*
    TestNG, org.testng.collections.Maps

    <K:Ljava/lang/Object;V:Ljava/lang/Object;>(Ljava/util/Map<TK;TV;>;)Ljava/util/Map<TK;TV;>;

    public static <K extends java.lang.Object, V extends java.lang.Object> java.util.Map<K, V> newHashMap(java.util.Map<K, V>);
    */
    pub generics: Vec<(BytesRef, Type)>,

    pub args: Vec<Type>,
    pub retype: Type,
}

pub struct FieldSignature {
    pub field_type: Type,
}

impl ClassSignature {
    pub fn new(raw: &[u8]) -> Self {
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let (_, cs) = Self::parse(s).unwrap();
        cs
    }

    fn parse(i: &str) -> IResult<&str, ClassSignature> {
        let (i, items) = parse_types(i)?;
        Ok((i, ClassSignature { items }))
    }
}

impl MethodSignature {
    pub fn new(raw: &[u8]) -> Self {
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let (_, r) = Self::parse(s).unwrap();
        r
    }

    fn parse(i: &str) -> IResult<&str, MethodSignature> {
        fn arg0(i: &str) -> IResult<&str, MethodSignature> {
            let (i, _) = tag("()")(i)?;
            let (i, retype) = parse_type(i)?;
            Ok((
                i,
                MethodSignature {
                    generics: vec![],
                    args: vec![],
                    retype,
                },
            ))
        }

        fn args(i: &str) -> IResult<&str, MethodSignature> {
            let (i_return, i_args) = delimited(char('('), is_not(")"), char(')'))(i)?;
            let (_, args) = parse_types(i_args)?;
            let (i, retype) = parse_type(i_return)?;
            Ok((
                i,
                MethodSignature {
                    generics: vec![],
                    args,
                    retype,
                },
            ))
        }

        fn generic(i: &str) -> IResult<&str, MethodSignature> {
            let (i, _) = tag("<")(i)?;
            let (i, generics) = many1(generic_declare)(i)?;
            let (i, _) = tag(">")(i)?;
            let (i, mut r) = MethodSignature::parse(i)?;

            r.generics = generics;

            Ok((i, r))
        }

        alt((arg0, args, generic))(i)
    }
}

impl FieldSignature {
    pub fn new(raw: &[u8]) -> Self {
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let (_, r) = Self::parse(s).unwrap();
        r
    }

    fn parse(mut i: &str) -> IResult<&str, FieldSignature> {
        let (i, field_type) = parse_type(i)?;
        Ok((i, FieldSignature { field_type }))
    }
}

impl Default for MethodSignature {
    fn default() -> Self {
        Self {
            generics: Vec::new(),
            args: Vec::new(),
            retype: Type::Void,
        }
    }
}

///////////////////////////
//parser
///////////////////////////

fn primitive<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    let (i, t) = one_of("BCDFIJSZV")(i)?;
    let t = match t {
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
    };

    Ok((i, t))
}

fn object_desc<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, BytesRef, E> {
    // should stop when reach ';' or '<'
    //such as:
    //(Lorg/testng/internal/IConfiguration;Lorg/testng/ISuite;Lorg/testng/xml/XmlTest;Ljava/lang/String;Lorg/testng/internal/annotations/IAnnotationFinder;ZLjava/util/List<Lorg/testng/IInvokedMethodListener;>;)V
    // if only take_till(|c| c == ';'), can't process like:
    //    Lxx/xx/xx<Lxx/xx/xx;>;
    let (_, _) = alt((tag("L"), tag("T")))(input)?;
    let (i, desc) = take_till(|c| c == ';' || c == '<')(input)?;
    let (i, _) = tag(";")(i)?;
    let mut buf = Vec::with_capacity(1 + desc.len() + 1);
    buf.extend_from_slice(desc.as_bytes());
    buf.push(b';');
    let desc = std::sync::Arc::new(buf);
    Ok((i, desc))
}

fn object_generic<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    let (i, tag_prefix) = alt((tag("L"), tag("T")))(i)?;
    let (i, container) = take_till(|c| c == '<')(i)?;
    let (mut i, _) = tag("<")(i)?;

    //signature like:
    //Ljava/lang/Class<+Lcom/google/inject/Module;>;
    //<=> 'java.lang.Class<? extends com.google.inject.Module>'
    let mut prefix = None;
    if i.starts_with('+') {
        prefix = Some(b'+');
        let (i2, _) = tag("+")(i)?;
        i = i2;
    }

    let (i, generic_args) = many1(parse_type)(i)?;
    let (i, _) = tag(">")(i)?;
    let (i, _) = tag(";")(i)?;

    //build results
    let mut buf = Vec::with_capacity(1 + container.len() + 1);
    buf.extend_from_slice(tag_prefix.as_bytes());
    buf.extend_from_slice(container.as_bytes());
    buf.push(b';');
    let desc = std::sync::Arc::new(buf);
    Ok((i, Type::Object(desc, Some(generic_args), prefix)))
}

fn object_normal<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    let (i, _) = peek(alt((tag("L"), tag("T"))))(i)?;
    let (i, desc) = object_desc(i)?;
    Ok((i, Type::Object(desc, None, None)))
}

fn object<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    alt((object_normal, object_generic))(i)
}

fn array<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    let (i, _) = peek(tag("["))(i)?;
    let (i, ary_tags) = take_till(|c| c != '[')(i)?;
    let (mut i, t) = peek(take(1u8))(i)?;

    let mut buf = vec![];
    buf.extend_from_slice(ary_tags.as_bytes());
    match t {
        "L" | "T" => {
            let (i2, desc) = object_desc(i)?;
            i = i2;
            buf.extend_from_slice(desc.as_slice());
        }
        v => {
            let (i2, _) = take(1u8)(i)?;
            i = i2;
            buf.extend_from_slice(v.as_bytes())
        }
    }
    let desc = std::sync::Arc::new(buf);
    Ok((i, Type::Array(desc)))
}

fn generic_declare<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, (BytesRef, Type), E> {
    let (i, generic_type) = take_till(|c| c == ':')(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, t) = parse_type(i)?;
    let generic_type = std::sync::Arc::new(Vec::from(generic_type));
    Ok((i, (generic_type, t)))
}

fn parse_type<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Type, E> {
    alt((primitive, object, array))(i)
}

fn parse_types<'a, E: ParseError<&'a str>>(mut input: &'a str) -> IResult<&str, Vec<Type>, E> {
    let it = std::iter::from_fn(move || {
        match parse_type::<'a, E>(input) {
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

#[cfg(test)]
mod tests {
    use super::ClassSignature;
    use super::FieldSignature;
    use super::MethodSignature;
    use super::Type as SignatureType;
    use std::sync::Arc;

    #[test]
    fn t_method_no_arg() {
        let expected = MethodSignature {
            generics: vec![],
            args: vec![],
            retype: SignatureType::Void,
        };
        let (_, r) = MethodSignature::parse("()V").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn method_primitive() {
        let table = vec![
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Byte],
                    retype: SignatureType::Void,
                },
                "(B)V",
            ),
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Char],
                    retype: SignatureType::Void,
                },
                "(C)V",
            ),
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Double],
                    retype: SignatureType::Void,
                },
                "(D)V",
            ),
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Float],
                    retype: SignatureType::Void,
                },
                "(F)V",
            ),
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Int],
                    retype: SignatureType::Void,
                },
                "(I)V",
            ),
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Long],
                    retype: SignatureType::Void,
                },
                "(J)V",
            ),
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Short],
                    retype: SignatureType::Void,
                },
                "(S)V",
            ),
            (
                MethodSignature {
                    generics: vec![],
                    args: vec![SignatureType::Boolean],
                    retype: SignatureType::Void,
                },
                "(Z)V",
            ),
        ];

        for (expected, desc) in table.iter() {
            let (_, r) = MethodSignature::parse(desc).unwrap();
            assert_eq!(r.args, expected.args);
            assert_eq!(r.retype, expected.retype);
        }
    }

    #[test]
    fn method_array_object() {
        let expected = MethodSignature {
            generics: vec![],
            args: vec![SignatureType::Array(Arc::new(Vec::from(
                "[[Ljava/lang/String;",
            )))],
            retype: SignatureType::Void,
        };
        let (_, r) = MethodSignature::parse("([[Ljava/lang/String;)V").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn method_mix() {
        let expected = MethodSignature {
            generics: vec![],
            args: vec![
                SignatureType::Byte,
                SignatureType::Char,
                SignatureType::Double,
                SignatureType::Float,
                SignatureType::Int,
                SignatureType::Long,
                SignatureType::Short,
                SignatureType::Boolean,
                SignatureType::Object(Arc::new(Vec::from("Ljava/lang/Integer;")), None, None),
            ],
            retype: SignatureType::Object(Arc::new(Vec::from("Ljava/lang/String;")), None, None),
        };
        let (_, r) =
            MethodSignature::parse("(BCDFIJSZLjava/lang/Integer;)Ljava/lang/String;").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn method_arg_generic() {
        let generic_args = vec![SignatureType::Object(
            Arc::new(Vec::from("Ljava/lang/String;")),
            None,
            None,
        )];
        let expected = MethodSignature {
            generics: vec![],
            args: vec![SignatureType::Object(
                Arc::new(Vec::from("Ljava/util/List;")),
                Some(generic_args),
                None,
            )],
            retype: SignatureType::Void,
        };
        let (_, r) = MethodSignature::parse("(Ljava/util/List<Ljava/lang/String;>;)V").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);

        let expected = MethodSignature {
            generics: vec![],
            args: vec![
                SignatureType::Object(
                    Arc::new(Vec::from("Lorg/testng/internal/IConfiguration;")),
                    None,
                    None,
                ),
                SignatureType::Object(Arc::new(Vec::from("Lorg/testng/ISuite;")), None, None),
                SignatureType::Object(Arc::new(Vec::from("Lorg/testng/xml/XmlTest;")), None, None),
                SignatureType::Object(Arc::new(Vec::from("Ljava/lang/String;")), None, None),
                SignatureType::Object(
                    Arc::new(Vec::from(
                        "Lorg/testng/internal/annotations/IAnnotationFinder;",
                    )),
                    None,
                    None,
                ),
                SignatureType::Boolean,
                SignatureType::Object(
                    Arc::new(Vec::from("Ljava/util/List;")),
                    Some(vec![SignatureType::Object(
                        Arc::new(Vec::from("Lorg/testng/IInvokedMethodListener;")),
                        None,
                        None,
                    )]),
                    None,
                ),
            ],
            retype: SignatureType::Void,
        };
        let (_, r) = MethodSignature::parse("(Lorg/testng/internal/IConfiguration;Lorg/testng/ISuite;Lorg/testng/xml/XmlTest;Ljava/lang/String;Lorg/testng/internal/annotations/IAnnotationFinder;ZLjava/util/List<Lorg/testng/IInvokedMethodListener;>;)V").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn generic1() {
        let expected = MethodSignature {
            generics: vec![],
            args: vec![
                SignatureType::Object(Arc::new(Vec::from("TK;")), None, None),
                SignatureType::Object(Arc::new(Vec::from("TV;")), None, None),
            ],
            retype: SignatureType::Void,
        };
        let (_, r) = MethodSignature::parse("(TK;TV;)V").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    //'T' tag in args
    #[test]
    fn generic2() {
        let expected = MethodSignature {
            generics: vec![],
            args: vec![SignatureType::Object(
                Arc::new(Vec::from("TK;")),
                None,
                None,
            )],
            retype: SignatureType::Object(
                Arc::new(Vec::from("Ljava/util/Set;")),
                Some(vec![SignatureType::Object(
                    Arc::new(Vec::from("TV;")),
                    None,
                    None,
                )]),
                None,
            ),
        };
        let (_, r) = MethodSignature::parse("(TK;)Ljava/util/Set<TV;>;").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn generic_nest1() {
        let expected = MethodSignature {
            generics: vec![],
            args: vec![],
            retype: SignatureType::Object(
                Arc::new(Vec::from("Ljava/util/Set;")),
                Some(vec![SignatureType::Object(
                    Arc::new(Vec::from("Ljava/util/Map$Entry;")),
                    Some(vec![
                        SignatureType::Object(Arc::new(Vec::from("TK;")), None, None),
                        SignatureType::Object(
                            Arc::new(Vec::from("Ljava/util/Set;")),
                            Some(vec![SignatureType::Object(
                                Arc::new(Vec::from("TV;")),
                                None,
                                None,
                            )]),
                            None,
                        ),
                    ]),
                    None,
                )]),
                None,
            ),
        };
        let (_, r) = MethodSignature::parse(
            "()Ljava/util/Set<Ljava/util/Map$Entry<TK;Ljava/util/Set<TV;>;>;>;",
        )
        .unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn generic_method() {
        let expected = MethodSignature {
            generics: vec![
                (
                    Arc::new(Vec::from("K")),
                    SignatureType::Object(Arc::new(Vec::from("Ljava/lang/Object;")), None, None),
                ),
                (
                    Arc::new(Vec::from("V")),
                    SignatureType::Object(Arc::new(Vec::from("Ljava/lang/Object;")), None, None),
                ),
            ],
            args: vec![SignatureType::Object(
                Arc::new(Vec::from("Ljava/util/Map;")),
                Some(vec![
                    SignatureType::Object(Arc::new(Vec::from("TK;")), None, None),
                    SignatureType::Object(Arc::new(Vec::from("TV;")), None, None),
                ]),
                None,
            )],
            retype: SignatureType::Object(
                Arc::new(Vec::from("Ljava/util/Map;")),
                Some(vec![
                    SignatureType::Object(Arc::new(Vec::from("TK;")), None, None),
                    SignatureType::Object(Arc::new(Vec::from("TV;")), None, None),
                ]),
                None,
            ),
        };
        let (_, r) = MethodSignature::parse("<K:Ljava/lang/Object;V:Ljava/lang/Object;>(Ljava/util/Map<TK;TV;>;)Ljava/util/Map<TK;TV;>;").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn method_return_generic() {
        let generic_args = vec![SignatureType::Object(
            Arc::new(Vec::from("Lorg/testng/ITestNGListener;")),
            None,
            None,
        )];
        let expected = MethodSignature {
            generics: vec![],
            args: vec![],
            retype: SignatureType::Object(
                Arc::new(Vec::from("Ljava/util/List;")),
                Some(generic_args),
                None,
            ),
        };
        let (_, r) =
            MethodSignature::parse("()Ljava/util/List<Lorg/testng/ITestNGListener;>;").unwrap();
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn field() {
        macro_rules! setup_test {
            ($desc: expr, $tp: expr) => {
                let (_, sig) = FieldSignature::parse($desc).unwrap();
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
        setup_test!("Ljava/lang/Object;", SignatureType::Object(v, None, None));
        setup_test!("S", SignatureType::Short);
        setup_test!("Z", SignatureType::Boolean);

        let v = Vec::from("[Ljava/lang/Object;");
        let v = Arc::new(v);
        setup_test!("[Ljava/lang/Object;", SignatureType::Array(v));

        let v = Vec::from("[[[D");
        let v = Arc::new(v);
        setup_test!("[[[D", SignatureType::Array(v));
    }

    #[test]
    fn t_class_signature() {
        let (_, cs) = ClassSignature::parse("Ljava/lang/Object;Lorg/testng/ITestContext;Lorg/testng/internal/ITestResultNotifier;Lorg/testng/internal/thread/graph/IThreadWorkerFactory<Lorg/testng/ITestNGMethod;>;").unwrap();
        let expected = ClassSignature {
            items: vec![
                SignatureType::Object(Arc::new(Vec::from("Ljava/lang/Object;")), None, None),
                SignatureType::Object(Arc::new(Vec::from("Lorg/testng/ITestContext;")), None, None),
                SignatureType::Object(
                    Arc::new(Vec::from("Lorg/testng/internal/ITestResultNotifier;")),
                    None,
                    None,
                ),
                SignatureType::Object(
                    Arc::new(Vec::from(
                        "Lorg/testng/internal/thread/graph/IThreadWorkerFactory;",
                    )),
                    Some(vec![SignatureType::Object(
                        Arc::new(Vec::from("Lorg/testng/ITestNGMethod;")),
                        None,
                        None,
                    )]),
                    None,
                ),
            ],
        };
        assert_eq!(cs.items, expected.items);
    }
}
