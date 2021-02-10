#![allow(unused)]

use std::fmt;
use std::sync::{Arc, Condvar, Mutex, RwLock};

use classfile::{BytesRef, ClassFile};

use crate::new_br;
use crate::oop::class::ClassObject;
use crate::runtime::{require_class3, ClassLoader};
use crate::types::*;
use crate::util::oop::{get_java_lang_integer_value_offset, get_java_lang_string_value_offset};

pub use self::ary::{ArrayOopDesc, TypeArrayDesc};
pub use self::class::{Class, ClassKind};
pub use self::inst::InstOopDesc;
pub use self::mirror::MirrorOopDesc;
pub use self::reference::{RefKind, RefKindDesc};
pub use self::values::ValueType;

pub mod ary;
pub mod class;
pub mod consts;
pub mod field;
pub mod inst;
pub mod mirror;
pub mod reference;
pub mod values;

#[derive(Clone)]
pub enum Oop {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /*
        used by: Throwable.java
    private static final String NULL_CAUSE_MESSAGE = "Cannot suppress a null exception.";
        */
    ConstUtf8(BytesRef),
    //used by oop::field::Filed::get_constant_value
    Null,

    Ref(Arc<OopPtr>),
}

#[derive(Debug)]
pub struct OopPtr(u64);

impl fmt::Debug for Oop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Oop::Int(v) => write!(f, "Oop(Int({}))", *v),
            Oop::Long(v) => write!(f, "Oop(Long({}))", *v),
            Oop::Float(v) => write!(f, "Oop(Float({}))", *v),
            Oop::Double(v) => write!(f, "Oop(Double({}))", *v),
            Oop::ConstUtf8(v) => write!(
                f,
                "Oop(ConstUtf8({}))",
                String::from_utf8_lossy(v.as_slice())
            ),
            Oop::Null => write!(f, "Oop(Null)"),
            Oop::Ref(rf) => {
                let ptr = rf.get_raw_ptr();
                unsafe {
                    match &(*ptr).v {
                        RefKind::Array(ary) => write!(f, "Oop(OopRef(Array))"),
                        RefKind::Inst(inst) => write!(f, "Oop(OopRef(Instance))"),
                        RefKind::TypeArray(ary) => write!(f, "Oop(OopRef(TypeArray))"),
                        RefKind::Mirror(mirror) => write!(f, "Oop(OopRef(Mirror))"),
                    }
                }
            }
        }
    }
}

//primitive value factor
impl Oop {
    pub fn new_int(v: i32) -> Self {
        Oop::Int(v)
    }

    pub fn new_long(v: i64) -> Self {
        Oop::Long(v)
    }

    pub fn new_float(v: f32) -> Self {
        Oop::Float(v)
    }

    pub fn new_double(v: f64) -> Self {
        Oop::Double(v)
    }
}

//primitive ary value factor
impl Oop {
    pub fn char_ary_from1(v: &[u16]) -> Oop {
        let elms = Vec::from(v);
        Self::new_char_ary2(elms)
    }

    pub fn new_byte_ary(len: usize) -> Oop {
        let elms = vec![0; len];
        Self::new_byte_ary2(elms)
    }

    pub fn new_bool_ary(len: usize) -> Oop {
        let elms = vec![0; len];
        Self::new_bool_ary2(elms)
    }

    pub fn new_char_ary(len: usize) -> Oop {
        let elms = vec![0; len];
        Self::new_char_ary2(elms)
    }

    pub fn new_short_ary(len: usize) -> Oop {
        let elms = vec![0; len];
        Self::new_short_ary2(elms)
    }

    pub fn new_int_ary(len: usize) -> Oop {
        let elms = vec![0; len];
        Self::new_int_ary2(elms)
    }

    pub fn new_float_ary(len: usize) -> Oop {
        let elms = vec![0.0; len];
        Self::new_float_ary2(elms)
    }

    pub fn new_double_ary(len: usize) -> Oop {
        let elms = vec![0.0; len];
        Self::new_double_ary2(elms)
    }

    pub fn new_long_ary(len: usize) -> Oop {
        let elms = vec![0; len];
        Self::new_long_ary2(elms)
    }

    pub fn new_byte_ary2(elms: Vec<u8>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Byte(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_bool_ary2(elms: Vec<u8>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Bool(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_char_ary2(elms: Vec<u16>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Char(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_short_ary2(elms: Vec<i16>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Short(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_int_ary2(elms: Vec<i32>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Int(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_float_ary2(elms: Vec<f32>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Float(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_double_ary2(elms: Vec<f64>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Double(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }

    pub fn new_long_ary2(elms: Vec<i64>) -> Oop {
        let ary = Box::new(elms);
        let v = TypeArrayDesc::Long(ary);
        Self::new_ref(RefKind::TypeArray(v))
    }
}

//reference value factory
impl Oop {
    pub fn new_const_utf8(v: BytesRef) -> Self {
        Oop::ConstUtf8(v)
    }

    pub fn new_null() -> Self {
        Oop::Null
    }

    pub fn new_inst(cls_obj: ClassRef) -> Oop {
        let v = InstOopDesc::new(cls_obj);
        Self::new_ref(RefKind::Inst(v))
    }
}

//mirror
impl Oop {
    pub fn new_mirror(target: ClassRef) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values,
            value_type: ValueType::OBJECT,
        };

        Self::new_ref(RefKind::Mirror(v))
    }

    pub fn new_prim_mirror(value_type: ValueType, target: Option<ClassRef>) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target,
            field_values,
            value_type,
        };

        Self::new_ref(RefKind::Mirror(v))
    }

    pub fn new_ary_mirror(target: ClassRef, value_type: ValueType) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values: vec![],
            value_type,
        };

        Self::new_ref(RefKind::Mirror(v))
    }
}

//array reference factory
impl Oop {
    pub fn new_ref_ary(ary_cls_obj: ClassRef, len: usize) -> Oop {
        let elements = vec![consts::get_null(); len];
        Self::new_ref_ary2(ary_cls_obj, elements)
    }

    pub fn new_ref_ary2(ary_cls_obj: ClassRef, elms: Vec<Oop>) -> Oop {
        let v = ArrayOopDesc::new(ary_cls_obj, elms);
        Self::new_ref(RefKind::Array(v))
    }
}

//private helper
impl Oop {
    fn new_ref(v: RefKind) -> Oop {
        let v = RefKindDesc::new(v);
        let v = Box::new(v);
        let ptr = Box::into_raw(v) as u64;
        let rf = Arc::new(OopPtr(ptr));
        Oop::Ref(rf)
    }
}

impl Oop {
    pub fn hash_code(&self) -> i32 {
        match self {
            Oop::Ref(rf) => {
                if OopPtr::is_java_lang_string(rf.clone()) {
                    OopPtr::java_lang_string_hash(rf.clone())
                } else {
                    rf.0 as i32
                }
            }
            Oop::Null => 0,
            _ => unreachable!(),
        }
    }
}

impl Oop {
    pub fn is_null(&self) -> bool {
        match self {
            Oop::Null => true,
            _ => false,
        }
    }

    #[inline]
    pub fn extract_int(&self) -> i32 {
        match self {
            Oop::Int(v) => *v,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract_float(&self) -> f32 {
        match &self {
            Oop::Float(v) => *v,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract_long(&self) -> i64 {
        match self {
            Oop::Long(v) => *v,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract_double(&self) -> f64 {
        match self {
            Oop::Double(v) => *v,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn extract_ref(&self) -> Arc<OopPtr> {
        match self {
            Oop::Ref(v) => v.clone(),
            t => unreachable!("t = {:?}", t),
        }
    }
}

impl OopPtr {
    pub fn get_raw_ptr(&self) -> *const RefKindDesc {
        self.0 as *const RefKindDesc
    }

    pub fn get_mut_raw_ptr(&self) -> *mut RefKindDesc {
        self.0 as *mut RefKindDesc
    }
}

impl OopPtr {
    pub fn extract_inst(&self) -> &InstOopDesc {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).v.extract_inst() }
    }

    pub fn extract_array(&self) -> &ArrayOopDesc {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).v.extract_array() }
    }

    pub fn extract_mut_array(&self) -> &mut ArrayOopDesc {
        let ptr = self.get_mut_raw_ptr();
        unsafe { (*ptr).v.extract_mut_array() }
    }

    pub fn extract_type_array(&self) -> &TypeArrayDesc {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).v.extract_type_array() }
    }

    pub fn extract_mut_type_array(&self) -> &mut TypeArrayDesc {
        let ptr = self.get_mut_raw_ptr();
        unsafe { (*ptr).v.extract_mut_type_array() }
    }

    pub fn extract_mirror(&self) -> &MirrorOopDesc {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).v.extract_mirror() }
    }
}

impl OopPtr {
    pub fn monitor_enter(&self) {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).monitor_enter() };
    }

    pub fn monitor_exit(&self) {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).monitor_exit() };
    }

    pub fn notify_all(&self) {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).notify_all() }
    }

    pub fn wait(&self) {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).wait() }
    }

    pub fn wait_timeout(&self, duration: std::time::Duration) {
        let ptr = self.get_raw_ptr();
        unsafe { (*ptr).wait_timeout(duration) }
    }
}

impl OopPtr {
    pub fn is_eq(l: &Oop, r: &Oop) -> bool {
        let l_is_null = l.is_null();
        let r_is_null = r.is_null();

        match (l_is_null, r_is_null) {
            (true, true) => return true,
            (true, false) => return false,
            (false, true) => return false,
            (false, false) => (),
        }

        let l = l.extract_ref();
        let r = r.extract_ref();

        if l.0 == r.0 {
            true
        } else if Self::is_java_lang_string(l.clone()) && Self::is_java_lang_string(r.clone()) {
            Self::is_java_lang_string_eq(l, r)
        } else {
            false
        }
    }

    pub fn is_java_lang_string(rf: Arc<Self>) -> bool {
        let ptr = rf.get_raw_ptr();
        unsafe {
            match &(*ptr).v {
                RefKind::Inst(inst) => {
                    let cls = inst.class.get_class();
                    cls.name.as_slice() == b"java/lang/String"
                }
                _ => false,
            }
        }
    }

    fn is_java_lang_string_eq(l: Arc<Self>, r: Arc<Self>) -> bool {
        let offset = get_java_lang_string_value_offset();

        //java.lang.String.value
        let v1 = Class::get_field_value2(l, offset);
        let v2 = Class::get_field_value2(r, offset);

        let rf1 = v1.extract_ref();
        let rf2 = v2.extract_ref();

        let chars1 = rf1.extract_type_array().extract_chars();
        let chars2 = rf2.extract_type_array().extract_chars();

        chars1 == chars2
    }
}

impl OopPtr {
    pub fn java_lang_string(rf: Arc<Self>) -> String {
        let v = Self::java_lang_string_value(rf);
        String::from_utf16_lossy(v.as_slice())
    }

    //java.lang.String.value
    pub fn java_lang_string_value(rf: Arc<Self>) -> Vec<u16> {
        let offset = get_java_lang_string_value_offset();
        let v = Class::get_field_value2(rf, offset);
        let rf = v.extract_ref();
        let chars = rf.extract_type_array().extract_chars();
        chars.to_vec()
    }

    pub fn java_lang_string_hash(rf: Arc<Self>) -> i32 {
        let offset = get_java_lang_string_value_offset();
        let v = Class::get_field_value2(rf, offset);

        let rf = v.extract_ref();
        let chars = rf.extract_type_array().extract_chars();

        let mut h = 0i32;
        for v in chars.iter() {
            h = h.wrapping_mul(31).wrapping_add(*v as i32);
        }

        h
    }

    //java.lang.Integer.value
    pub fn java_lang_integer_value(rf: Arc<Self>) -> i32 {
        let offset = get_java_lang_integer_value_offset();
        Class::get_field_value2(rf, offset).extract_int()
    }

    //java.lang.Thread.eetop
    pub fn java_lang_thread_eetop(rf: Arc<Self>) -> i64 {
        let fid = {
            let inst = rf.extract_inst();
            let cls = inst.class.clone();
            let cls = cls.get_class();
            cls.get_field_id(&new_br("eetop"), &new_br("J"), false)
        };

        Class::get_field_value(rf, fid).extract_long()
    }
}

impl Drop for OopPtr {
    fn drop(&mut self) {
        let _v = unsafe { Box::from_raw(self.0 as *mut RefKindDesc) };
    }
}

pub fn init() {
    consts::init();
}
