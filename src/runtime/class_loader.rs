use crate::classfile::{constant_pool, types::*};
use crate::oop::{self, ClassObject, ClassRef, ValueType};
use crate::parser as class_parser;
use crate::runtime::{self, ClassPathResult};
use crate::util;
use std::sync::{Arc, Mutex};

#[derive(Debug, Copy, Clone)]
pub enum ClassLoader {
    Base,
    Bootstrap,
}

pub fn require_class(class_loader: Option<ClassLoader>, name: BytesRef) -> Option<ClassRef> {
    require_class3(class_loader, name.as_slice())
}

pub fn require_class2(index: U2, cp: &ConstantPool) -> Option<ClassRef> {
    let class = constant_pool::get_class_name(cp, index as usize)?;
    require_class3(None, class.as_slice())
}

pub fn require_class3(class_loader: Option<ClassLoader>, name: &[u8]) -> Option<ClassRef> {
    let class_loader = class_loader.unwrap_or(ClassLoader::Bootstrap);
    class_loader.load_class(name)
}

impl ClassLoader {
    fn load_class(&self, name: &[u8]) -> Option<ClassRef> {
        if is_array(name) {
            self.load_array_class(name)
        } else {
            match self {
                ClassLoader::Base => (),
                ClassLoader::Bootstrap => {
                    let it = runtime::sys_dic_find(name);
                    if it.is_some() {
                        return it;
                    }
                }
            }

            let mut class = self.load_class_from_path(name);

            match class.clone() {
                Some(mut class) => match self {
                    ClassLoader::Base => (),

                    ClassLoader::Bootstrap => {
                        runtime::sys_dic_put(name, class.clone());
                        let this_ref = class.clone();
                        util::sync_call_ctx(&class, move |it| {
                            it.set_class_state(oop::class::State::Loaded);
                            it.link_class(this_ref);
                        });
                    }
                },

                None => (),
            }

            class
        }
    }

    fn load_array_class(&self, name: &[u8]) -> Option<ClassRef> {
        match calc_dimension(name) {
            Some(1) => {
                // dimension == 1
                match name.get(1) {
                    Some(b'L') => {
                        //[Ljava/lang/Object;
                        let elm = &name[2..name.len() - 1];
                        match self.load_class(elm) {
                            Some(elm) => {
                                let class = ClassObject::new_object_ary(*self, elm, name);
                                Some(new_ref!(class))
                            }
                            None => None,
                        }
                    }

                    Some(t) => {
                        //B, Z...
                        let elm = t.into();
                        let class = ClassObject::new_prime_ary(*self, elm);
                        Some(new_ref!(class))
                    }

                    None => unreachable!(),
                }
            }

            _ => {
                // dimension > 1
                let down_type_name = &name[1..];
                match self.load_array_class(down_type_name) {
                    Some(down_type) => {
                        let class = ClassObject::new_wrapped_ary(*self, down_type);
                        Some(new_ref!(class))
                    }

                    None => None,
                }
            }
        }
    }

    fn load_class_from_path(&self, name: &[u8]) -> Option<ClassRef> {
        let name = String::from_utf8_lossy(name);
        match runtime::find_class_in_classpath(&name) {
            Ok(ClassPathResult(_, _, buf)) => match class_parser::parse_buf(buf) {
                Ok(cf) => {
                    let cfr = Arc::new(cf);
                    let class = ClassObject::new_class(cfr, Some(*self));
                    Some(new_ref!(class))
                }

                Err(_) => None,
            },

            Err(_) => None,
        }
    }
}

fn calc_dimension(name: &[u8]) -> Option<usize> {
    if is_array(name) {
        name.iter().position(|&c| c != b'[')
    } else {
        None
    }
}

fn is_array(name: &[u8]) -> bool {
    name.starts_with(&[b'['])
}

#[cfg(test)]
mod tests {
    #[test]
    fn t_basic() {
        use super::calc_dimension;
        assert_eq!(calc_dimension("".as_bytes()), None);
        assert_eq!(calc_dimension("Ljava/lang/Object;".as_bytes()), None);
        assert_eq!(calc_dimension("Z".as_bytes()), None);
        assert_eq!(calc_dimension("[B".as_bytes()), Some(1));
        assert_eq!(calc_dimension("[[B".as_bytes()), Some(2));
        assert_eq!(calc_dimension("[[[B".as_bytes()), Some(3));
        assert_eq!(calc_dimension("[[[[B".as_bytes()), Some(4));
        assert_eq!(calc_dimension("[[[[[B".as_bytes()), Some(5));
        assert_eq!(calc_dimension("[Ljava/lang/Object;".as_bytes()), Some(1));
        assert_eq!(calc_dimension("[[Ljava/lang/Object;".as_bytes()), Some(2));

        let name = "[Ljava/lang/Object;";
        assert_eq!("java/lang/Object", &name[2..name.len() - 1]);
    }
}
