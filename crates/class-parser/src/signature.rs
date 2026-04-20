use classfile::BytesRef;
use classfile::SignatureType as Type;

/// Minimal cursor-based parser for JVM signature strings.
/// Replaces the nom 7.x version with character-by-character parsing.
///
/// Grammar reference (JVM Spec §4.7.9.1):
///   JavaTypeSignature ::= BaseType | ObjectType | ArrayType
///   BaseType          ::= B | C | D | F | I | J | S | Z | V
///   ObjectType        ::= L ClassName [ < TypeArgs > ] ;
///                       | T TypeVar   [ < TypeArgs > ] ;
///   ArrayType         ::= [ JavaTypeSignature
///   MethodSignature   ::= [ < FormalTypeParameters> ] ( JavaTypeSignature* ) ReturnType
///   ClassSignature    ::= JavaTypeSignature+

#[derive(Debug)]
pub struct ClassSignature {
    pub items: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub generics: Vec<(BytesRef, Type)>,
    pub args: Vec<Type>,
    pub retype: Type,
}

pub struct FieldSignature {
    pub field_type: Type,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser { s, pos: 0 }
    }

    fn peek_char(&self) -> Option<char> {
        self.s[self.pos..].chars().next()
    }

    fn advance(&mut self) -> char {
        let c = self.s[self.pos..].chars().next().unwrap();
        self.pos += c.len_utf8();
        c
    }

    fn expect(&mut self, expected: char) {
        let got = self.advance();
        assert_eq!(got, expected, "expected '{expected}', got '{got}'");
    }

    /// Take bytes until any stop char, not consuming the stop char.
    fn take_until(&mut self, stops: &[char]) -> &'a str {
        let start = self.pos;
        let rest = &self.s[self.pos..];
        for (i, c) in rest.char_indices() {
            if stops.contains(&c) {
                self.pos = self.pos + i;
                return &self.s[start..self.pos];
            }
        }
        // No stop char found — consume to end
        self.pos = self.s.len();
        &self.s[start..]
    }

    // --- Type parsing ---

    fn parse_primitive(&mut self) -> Type {
        match self.advance() {
            'B' => Type::Byte,
            'C' => Type::Char,
            'D' => Type::Double,
            'F' => Type::Float,
            'I' => Type::Int,
            'J' => Type::Long,
            'S' => Type::Short,
            'Z' => Type::Boolean,
            'V' => Type::Void,
            c => unreachable!("bad primitive char: '{c}'"),
        }
    }

    fn parse_object(&mut self) -> Type {
        let prefix = self.advance(); // 'L' or 'T'
        let name = self.take_until(&[';', '<']);
        let mut buf = Vec::with_capacity(1 + name.len() + 1);
        buf.push(prefix as u8);
        buf.extend_from_slice(name.as_bytes());

        if self.peek_char() == Some('<') {
            // Has generic args
            self.advance(); // '<'
            let generic_args = self.parse_generic_args();
            self.expect('>');
            self.expect(';');
            buf.push(b';');
            Type::Object(std::sync::Arc::new(buf), Some(generic_args), None)
        } else {
            self.expect(';');
            buf.push(b';');
            Type::Object(std::sync::Arc::new(buf), None, None)
        }
    }

    fn parse_generic_args(&mut self) -> Vec<Type> {
        let mut args = Vec::new();
        while self.peek_char().is_some() && self.peek_char() != Some('>') {
            // Handle variance markers: +, -, *
            match self.peek_char() {
                Some('+') | Some('-') | Some('*') => {
                    self.advance(); // skip marker, for now we don't track variance in Type
                    args.push(self.parse_type());
                }
                _ => {
                    args.push(self.parse_type());
                }
            }
        }
        args
    }

    fn parse_array(&mut self) -> Type {
        // Count leading '['
        let mut num_arrays = 0;
        while self.peek_char() == Some('[') {
            num_arrays += 1;
            self.advance();
        }
        // Parse element type
        let element = self.parse_type();

        // Build raw descriptor: '[' * N + element_bytes
        let element_bytes: Vec<u8> = match &element {
            Type::Object(bytes, _, _) => bytes.to_vec(),
            Type::Array(bytes) => bytes.to_vec(),
            _ => {
                // For primitives, extract the element chars from the source string
                let elem_start = self.pos - 1; // primitives are 1 char
                self.s[elem_start..self.pos].as_bytes().to_vec()
            }
        };

        let mut buf = Vec::with_capacity(num_arrays + element_bytes.len());
        for _ in 0..num_arrays {
            buf.push(b'[');
        }
        buf.extend_from_slice(&element_bytes);

        Type::Array(std::sync::Arc::new(buf))
    }

    fn parse_type(&mut self) -> Type {
        match self.peek_char() {
            Some('B') | Some('C') | Some('D') | Some('F') | Some('I') | Some('J') | Some('S')
            | Some('Z') | Some('V') => self.parse_primitive(),
            Some('L') | Some('T') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some(c) => unreachable!("bad type char: '{c}'"),
            None => unreachable!("unexpected end of signature"),
        }
    }

    fn parse_types(&mut self) -> Vec<Type> {
        let mut types = Vec::new();
        while self.peek_char().is_some() {
            types.push(self.parse_type());
        }
        types
    }

    fn parse_types_until(&mut self, stop: char) -> Vec<Type> {
        let mut types = Vec::new();
        while self.peek_char().is_some() && self.peek_char() != Some(stop) {
            types.push(self.parse_type());
        }
        types
    }

    // --- Entry points ---

    fn parse_method(&mut self) -> MethodSignature {
        if self.peek_char() == Some('<') {
            self.parse_generic_method()
        } else {
            self.parse_non_generic_method()
        }
    }

    fn parse_generic_method(&mut self) -> MethodSignature {
        self.expect('<');
        let mut generics = Vec::new();
        while self.peek_char().is_some() && self.peek_char() != Some('>') {
            let name = self.take_until(&[':']);
            self.expect(':');
            let bound = self.parse_type();
            generics.push((std::sync::Arc::new(name.as_bytes().to_vec()), bound));
        }
        self.expect('>');

        let mut result = self.parse_non_generic_method();
        result.generics = generics;
        result
    }

    fn parse_non_generic_method(&mut self) -> MethodSignature {
        self.expect('(');
        let args = self.parse_types_until(')');
        self.expect(')');
        let retype = self.parse_type();
        MethodSignature {
            generics: Vec::new(),
            args,
            retype,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl ClassSignature {
    pub fn new(raw: &[u8]) -> Self {
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let mut p = Parser::new(s);
        let items = p.parse_types();
        ClassSignature { items }
    }
}

impl MethodSignature {
    pub fn new(raw: &[u8]) -> Self {
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let mut p = Parser::new(s);
        p.parse_method()
    }
}

impl FieldSignature {
    pub fn new(raw: &[u8]) -> Self {
        let s = unsafe { std::str::from_utf8_unchecked(raw) };
        let mut p = Parser::new(s);
        let field_type = p.parse_type();
        FieldSignature { field_type }
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

// ---------------------------------------------------------------------------
// Tests — same as before
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
        let r = MethodSignature::new(b"()V");
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
            let r = MethodSignature::new(desc.as_bytes());
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
        let r = MethodSignature::new(b"([[Ljava/lang/String;)V");
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
        let r = MethodSignature::new(b"(BCDFIJSZLjava/lang/Integer;)Ljava/lang/String;");
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
        let r = MethodSignature::new(b"(Ljava/util/List<Ljava/lang/String;>;)V");
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
        let r = MethodSignature::new(b"(Lorg/testng/internal/IConfiguration;Lorg/testng/ISuite;Lorg/testng/xml/XmlTest;Ljava/lang/String;Lorg/testng/internal/annotations/IAnnotationFinder;ZLjava/util/List<Lorg/testng/IInvokedMethodListener;>;)V");
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
        let r = MethodSignature::new(b"(TK;TV;)V");
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

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
        let r = MethodSignature::new(b"(TK;)Ljava/util/Set<TV;>;");
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
        let r = MethodSignature::new(
            b"()Ljava/util/Set<Ljava/util/Map$Entry<TK;Ljava/util/Set<TV;>;>;>;",
        );
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
        let r = MethodSignature::new(b"<K:Ljava/lang/Object;V:Ljava/lang/Object;>(Ljava/util/Map<TK;TV;>;)Ljava/util/Map<TK;TV;>;");
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
        let r = MethodSignature::new(b"()Ljava/util/List<Lorg/testng/ITestNGListener;>;");
        assert_eq!(r.args, expected.args);
        assert_eq!(r.retype, expected.retype);
    }

    #[test]
    fn field() {
        use super::FieldSignature;

        macro_rules! setup_test {
            ($desc: expr, $tp: expr) => {
                let sig = FieldSignature::new($desc.as_bytes());
                assert_eq!(sig.field_type, $tp);
            };
        }

        setup_test!("B", SignatureType::Byte);
        setup_test!("C", SignatureType::Char);
        setup_test!("D", SignatureType::Double);
        setup_test!("F", SignatureType::Float);
        setup_test!("I", SignatureType::Int);
        setup_test!("J", SignatureType::Long);

        let v = Arc::new(Vec::from("Ljava/lang/Object;"));
        setup_test!("Ljava/lang/Object;", SignatureType::Object(v, None, None));
        setup_test!("S", SignatureType::Short);
        setup_test!("Z", SignatureType::Boolean);

        let v = Arc::new(Vec::from("[Ljava/lang/Object;"));
        setup_test!("[Ljava/lang/Object;", SignatureType::Array(v));

        let v = Arc::new(Vec::from("[[[D"));
        setup_test!("[[[D", SignatureType::Array(v));
    }

    #[test]
    fn t_class_signature() {
        use super::ClassSignature;

        let cs = ClassSignature::new(b"Ljava/lang/Object;Lorg/testng/ITestContext;Lorg/testng/internal/ITestResultNotifier;Lorg/testng/internal/thread/graph/IThreadWorkerFactory<Lorg/testng/ITestNGMethod;>;");
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
