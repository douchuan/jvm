use crate::native;
use crate::oop::class::ClassPtr;
use crate::oop::{self, Class, ValueType};
use crate::runtime::{self, ClassPathResult};
use crate::types::*;
use crate::util;
use class_parser::parse_class;
use classfile::{constant_pool, types::U2, BytesRef, ConstantPool};
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
    let class = constant_pool::get_class_name(cp, index as usize);
    // trace!("require_class2 class = {}", String::from_utf8_lossy(class.as_slice()));
    require_class3(None, class.as_slice())
}

pub fn require_class3(class_loader: Option<ClassLoader>, name: &[u8]) -> Option<ClassRef> {
    let class_loader = class_loader.unwrap_or(ClassLoader::Bootstrap);
    class_loader.load_class(name)
}

impl ClassLoader {
    fn load_class(&self, name: &[u8]) -> Option<ClassRef> {
        debug_assert!(!name.contains(&b'.'));
        //        error!("load_class name = {}", String::from_utf8_lossy(name));
        match self {
            ClassLoader::Base => (),
            ClassLoader::Bootstrap => {
                let it = runtime::sys_dic_find(name);
                if it.is_some() {
                    //                    info!("load_class in dic: {}", String::from_utf8_lossy(name));
                    return it;
                }
            }
        }

        if is_array(name) {
            self.load_array_class(name)
        } else {
            let class = self.load_class_from_path(name);
            if let Some(class) = &class {
                match self {
                    ClassLoader::Base => (),
                    ClassLoader::Bootstrap => {
                        runtime::sys_dic_put(name, class.clone());
                        let this_ref = class.clone();

                        {
                            let mut cls = class.get_mut_class();
                            cls.set_class_state(oop::class::State::Loaded);
                            cls.link_class(this_ref);
                        }

                        native::java_lang_Class::create_mirror(class.clone());
                    }
                }
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
                                let mut class = Class::new_object_ary(*self, elm, name);
                                let class = ClassPtr::new(class);
                                {
                                    let this_ref = class.clone();
                                    let mut class = class.get_mut_class();
                                    class.link_class(this_ref);
                                }
                                match self {
                                    ClassLoader::Base => (),
                                    ClassLoader::Bootstrap => {
                                        runtime::sys_dic_put(name, class.clone());
                                    }
                                }

                                native::java_lang_Class::create_mirror(class.clone());

                                Some(class)
                            }
                            None => None,
                        }
                    }

                    Some(t) => {
                        //B, Z...
                        let elm = t.into();
                        let class = Class::new_prime_ary(*self, elm);
                        let class = ClassPtr::new(class);

                        {
                            let this_ref = class.clone();
                            let mut class = class.get_mut_class();
                            class.link_class(this_ref);
                        }

                        match self {
                            ClassLoader::Base => (),
                            ClassLoader::Bootstrap => {
                                runtime::sys_dic_put(name, class.clone());
                            }
                        }

                        //mirror has been created when vm inited

                        Some(class)
                    }

                    None => unreachable!(),
                }
            }

            _ => {
                // dimension > 1
                let down_type_name = &name[1..];
                match self.load_array_class(down_type_name) {
                    Some(down_type) => {
                        let class = Class::new_wrapped_ary(*self, down_type);
                        let class = ClassPtr::new(class);
                        match self {
                            ClassLoader::Base => (),
                            ClassLoader::Bootstrap => {
                                runtime::sys_dic_put(name, class.clone());
                            }
                        }

                        native::java_lang_Class::create_mirror(class.clone());

                        Some(class)
                    }

                    None => None,
                }
            }
        }
    }

    fn load_class_from_path(&self, name: &[u8]) -> Option<ClassRef> {
        let name = unsafe { std::str::from_utf8_unchecked(name) };
        match runtime::find_class_in_classpath(name) {
            Ok(ClassPathResult(_, buf)) => match parse_class(&buf) {
                Ok(cf) => {
                    let cfr = Arc::new(Box::new(cf.1));
                    let class = Class::new_class(cfr, Some(*self));
                    Some(ClassPtr::new(class))
                }

                Err(e) => unreachable!("name={}, {}", name, e),
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
