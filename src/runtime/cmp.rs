use crate::classfile;
use crate::oop;
use crate::runtime::require_class3;
use crate::types::ClassRef;
use std::sync::Arc;

pub fn instance_of(s: ClassRef, t: ClassRef) -> bool {
    // Return if S and T are the same class
    if Arc::ptr_eq(&s, &t) {
        return true;
    }

    let (s_kind, s_is_intf) = {
        let cls = s.read().unwrap();
        (cls.get_class_kind_type(), cls.is_interface())
    };

    let (t_kind, t_is_intf) = {
        let cls = t.read().unwrap();
        (cls.get_class_kind_type(), cls.is_interface())
    };

    // If S is an ordinary (non-array) class
    if s_kind == oop::class::ClassKindType::Instance && !s_is_intf {
        // If T is an interface type, then S must implement interface T.
        if t_is_intf {
            let s = s.read().unwrap();
            return s.check_interface(t);
        }

        // If T is a class type, then S must be the same class as T,
        // or S must be a subclass of T;
        if t_kind == oop::class::ClassKindType::Instance {
            return check_inherit(s, t);
        }

        return false;
    }

    // If S is an interface type
    if s_is_intf {
        // If T is an interface type, then T must be the same interface as S
        // or a superinterface of S.
        if t_is_intf {
            return check_inherit(s, t);
        }

        // If T is a class type, then T must be Object
        if t_kind == oop::class::ClassKindType::Instance {
            let object = require_class3(None, classfile::consts::J_OBJECT).unwrap();
            return Arc::ptr_eq(&t, &object);
        }

        return false;
    }

    // If S is a class representing the array type SC[],
    // that is, an array of components of type SC
    match s_kind {
        oop::class::ClassKindType::TypAry | oop::class::ClassKindType::ObjectAry => {
            // If T is an interface type, then T must be one of the interfaces
            // implemented by arrays (JLS §4.10.3).
            // https://docs.oracle.com/javase/specs/jls/se7/html/jls-4.html#jls-4.10.3
            // array implements:
            // 1. java/lang/Cloneable
            // 2. java/io/Serializable
            if t_is_intf {
                let serializable = require_class3(None, classfile::consts::J_SERIALIZABLE).unwrap();
                let cloneable = require_class3(None, classfile::consts::J_CLONEABLE).unwrap();
                return Arc::ptr_eq(&t, &serializable) || Arc::ptr_eq(&t, &cloneable);
            }

            if t_kind == oop::class::ClassKindType::Instance {
                let object = require_class3(None, classfile::consts::J_OBJECT).unwrap();
                return Arc::ptr_eq(&t, &object);
            }

            if t_kind == oop::class::ClassKindType::TypAry
                && s_kind == oop::class::ClassKindType::TypAry
            {
                let cls = s.read().unwrap();
                let (s_dimension, s_value_type) = match &cls.kind {
                    oop::class::ClassKind::TypeArray(cls) => (cls.get_dimension(), cls.value_type),
                    _ => unreachable!(),
                };

                let cls = t.read().unwrap();
                let (t_dimension, t_value_type) = match &cls.kind {
                    oop::class::ClassKind::TypeArray(cls) => (cls.get_dimension(), cls.value_type),
                    _ => unreachable!(),
                };
                return s_dimension == t_dimension && s_value_type == t_value_type;
            }

            if t_kind == oop::class::ClassKindType::ObjectAry
                && s_kind == oop::class::ClassKindType::ObjectAry
            {
                let cls = s.read().unwrap();
                let (s_dimension, s_component) = match &cls.kind {
                    oop::class::ClassKind::ObjectArray(cls) => {
                        (cls.get_dimension(), cls.component.clone().unwrap())
                    }
                    _ => unreachable!(),
                };
                let cls = t.read().unwrap();
                let (t_dimension, t_component) = match &cls.kind {
                    oop::class::ClassKind::ObjectArray(cls) => {
                        (cls.get_dimension(), cls.component.clone().unwrap())
                    }
                    _ => unreachable!(),
                };
                return s_dimension == t_dimension && check_inherit(s_component, t_component);
            }
        }
        _ => (),
    }

    false
}

pub fn check_inherit(s: ClassRef, t: ClassRef) -> bool {
    let mut super_cls = s;

    loop {
        if Arc::ptr_eq(&super_cls, &t) {
            return true;
        }

        let cls = { super_cls.read().unwrap().super_class.clone() };
        match cls {
            Some(cls) => super_cls = cls,
            None => break,
        }
    }

    false
}
