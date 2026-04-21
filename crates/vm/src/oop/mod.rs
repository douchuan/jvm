#![allow(unused)]

use std::fmt;
use std::sync::Arc;

use classfile::{BytesRef, ClassFile};

use crate::new_br;
use crate::oop::class::ClassObject;
use crate::runtime::{require_class3, ClassLoader};
use crate::types::*;
use crate::util::oop::{get_java_lang_integer_value_offset, get_java_lang_string_value_offset};

pub mod ary;
pub mod class;
pub mod consts;
pub mod field;
pub mod heap;
pub mod inst;
pub mod mirror;
pub mod reference;
pub mod values;

pub use self::ary::{ArrayOopDesc, TypeArrayDesc, TypeArrayEnum};
pub use self::class::{Class, ClassKind};
pub use self::heap::Heap;
pub use self::inst::InstOopDesc;
pub use self::mirror::MirrorOopDesc;
pub use self::reference::{RefKind, RefKindDesc};
pub use self::values::ValueType;

use std::sync::Mutex;

/// Global VM state — provides Heap access to all code paths.
/// Initialized once at VM startup.
static VM_STATE: std::sync::OnceLock<VmState> = std::sync::OnceLock::new();

pub struct VmState {
    pub heap: Mutex<Heap>,
}

/// Initialize the global VM state. Must be called once before any Oop operations.
pub fn init_vm_state() {
    VM_STATE.get_or_init(|| VmState {
        heap: Mutex::new(Heap::new()),
    });
}

/// Access the global Heap immutably.
pub fn with_heap<R>(f: impl FnOnce(&Heap) -> R) -> R {
    let state = VM_STATE.get().expect("VM state not initialized");
    let heap = state.heap.lock().unwrap();
    f(&heap)
}

/// Access the global Heap mutably.
pub fn with_heap_mut<R>(f: impl FnOnce(&mut Heap) -> R) -> R {
    let state = VM_STATE.get().expect("VM state not initialized");
    let mut heap = state.heap.lock().unwrap();
    f(&mut heap)
}

/// JVM object pointer.
///
/// Primitive values are stored inline. Reference types use a `slot_id`
/// that indexes into the `Heap` — no raw pointers, no unsafe.
#[derive(Clone)]
pub enum Oop {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /// Static constant strings in class files
    ConstUtf8(BytesRef),
    Null,
    /// Heap slot index. Access via `heap.get(slot_id)`.
    Ref(u32),
}

impl fmt::Debug for Oop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Oop::Int(v) => write!(f, "Oop(Int({}))", v),
            Oop::Long(v) => write!(f, "Oop(Long({}))", v),
            Oop::Float(v) => write!(f, "Oop(Float({}))", v),
            Oop::Double(v) => write!(f, "Oop(Double({}))", v),
            Oop::ConstUtf8(v) => write!(
                f,
                "Oop(ConstUtf8({}))",
                String::from_utf8_lossy(v.as_slice())
            ),
            Oop::Null => write!(f, "Oop(Null)"),
            Oop::Ref(slot_id) => write!(f, "Oop(Ref(slot={}))", slot_id),
        }
    }
}

// Primitive value factories
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

// Reference value factories (allocate into heap)
impl Oop {
    pub fn new_null() -> Self {
        Oop::Null
    }
    pub fn new_const_utf8(v: BytesRef) -> Self {
        Oop::ConstUtf8(v)
    }

    pub fn new_inst(cls_obj: ClassRef) -> Oop {
        let v = InstOopDesc::new(cls_obj);
        with_heap_mut(|heap| Self::new_ref(RefKind::Inst(v), heap))
    }

    pub fn new_mirror(target: ClassRef) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values,
            value_type: ValueType::OBJECT,
        };
        with_heap_mut(|heap| Self::new_ref(RefKind::Mirror(v), heap))
    }

    pub fn new_prim_mirror(value_type: ValueType, target: Option<ClassRef>) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target,
            field_values,
            value_type,
        };
        with_heap_mut(|heap| Self::new_ref(RefKind::Mirror(v), heap))
    }

    pub fn new_ary_mirror(target: ClassRef, value_type: ValueType) -> Oop {
        let java_lang_class = require_class3(None, b"java/lang/Class").unwrap();
        let field_values = field::build_inited_field_values(java_lang_class);
        let v = MirrorOopDesc {
            target: Some(target),
            field_values: vec![],
            value_type,
        };
        with_heap_mut(|heap| Self::new_ref(RefKind::Mirror(v), heap))
    }

    pub fn new_ref_ary(ary_cls_obj: ClassRef, len: usize) -> Oop {
        let elements = vec![Oop::Null; len];
        Self::new_ref_ary2(ary_cls_obj, elements)
    }

    pub fn new_ref_ary2(ary_cls_obj: ClassRef, elms: Vec<Oop>) -> Oop {
        let v = ArrayOopDesc::new(ary_cls_obj, elms);
        with_heap_mut(|heap| Self::new_ref(RefKind::Array(v), heap))
    }

    pub fn new_type_ary(v: u8, len: usize) -> Oop {
        match TypeArrayEnum::from(v) {
            TypeArrayEnum::Boolean => Self::new_bool_ary(len),
            TypeArrayEnum::Char => Self::new_char_ary(len),
            TypeArrayEnum::Float => Self::new_float_ary(len),
            TypeArrayEnum::Double => Self::new_double_ary(len),
            TypeArrayEnum::Byte => Self::new_byte_ary(len),
            TypeArrayEnum::Short => Self::new_short_ary(len),
            TypeArrayEnum::Int => Self::new_int_ary(len),
            TypeArrayEnum::Long => Self::new_long_ary(len),
        }
    }

    pub fn char_ary_from1(v: &[u16]) -> Oop {
        let elms = Vec::from(v);
        Self::new_char_ary2(elms)
    }

    pub fn new_byte_ary(len: usize) -> Oop {
        Self::new_byte_ary2(vec![0; len])
    }

    fn new_bool_ary(len: usize) -> Oop {
        Self::new_bool_ary2(vec![0; len])
    }

    fn new_char_ary(len: usize) -> Oop {
        Self::new_char_ary2(vec![0; len])
    }

    fn new_short_ary(len: usize) -> Oop {
        Self::new_short_ary2(vec![0; len])
    }

    fn new_int_ary(len: usize) -> Oop {
        Self::new_int_ary2(vec![0; len])
    }

    fn new_float_ary(len: usize) -> Oop {
        Self::new_float_ary2(vec![0.0; len])
    }

    fn new_double_ary(len: usize) -> Oop {
        Self::new_double_ary2(vec![0.0; len])
    }

    fn new_long_ary(len: usize) -> Oop {
        Self::new_long_ary2(vec![0; len])
    }

    pub fn new_byte_ary2(elms: Vec<u8>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(
                RefKind::TypeArray(TypeArrayDesc::Byte(Box::new(elms))),
                heap,
            )
        })
    }

    pub fn new_bool_ary2(elms: Vec<u8>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(
                RefKind::TypeArray(TypeArrayDesc::Bool(Box::new(elms))),
                heap,
            )
        })
    }

    pub fn new_char_ary2(elms: Vec<u16>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(
                RefKind::TypeArray(TypeArrayDesc::Char(Box::new(elms))),
                heap,
            )
        })
    }

    pub fn new_short_ary2(elms: Vec<i16>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(
                RefKind::TypeArray(TypeArrayDesc::Short(Box::new(elms))),
                heap,
            )
        })
    }

    pub fn new_int_ary2(elms: Vec<i32>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(RefKind::TypeArray(TypeArrayDesc::Int(Box::new(elms))), heap)
        })
    }

    pub fn new_float_ary2(elms: Vec<f32>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(
                RefKind::TypeArray(TypeArrayDesc::Float(Box::new(elms))),
                heap,
            )
        })
    }

    pub fn new_double_ary2(elms: Vec<f64>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(
                RefKind::TypeArray(TypeArrayDesc::Double(Box::new(elms))),
                heap,
            )
        })
    }

    pub fn new_long_ary2(elms: Vec<i64>) -> Oop {
        with_heap_mut(|heap| {
            Self::new_ref(
                RefKind::TypeArray(TypeArrayDesc::Long(Box::new(elms))),
                heap,
            )
        })
    }

    fn new_ref(v: RefKind, heap: &mut Heap) -> Oop {
        let desc = RefKindDesc::new(v);
        let slot_id = heap.alloc(desc);
        Oop::Ref(slot_id)
    }
}

// Extractors
impl Oop {
    pub fn is_null(&self) -> bool {
        matches!(self, Oop::Null)
    }

    #[inline]
    pub fn extract_int(&self) -> i32 {
        match self {
            Oop::Int(v) => *v,
            Oop::Null => 0, // Tolerate null when expecting int
            _ => unreachable!("expected Int, got {:?}", self),
        }
    }

    #[inline]
    pub fn extract_float(&self) -> f32 {
        match self {
            Oop::Float(v) => *v,
            _ => unreachable!("expected Float, got {:?}", self),
        }
    }

    #[inline]
    pub fn extract_long(&self) -> i64 {
        match self {
            Oop::Long(v) => *v,
            _ => unreachable!("expected Long, got {:?}", self),
        }
    }

    #[inline]
    pub fn extract_double(&self) -> f64 {
        match self {
            Oop::Double(v) => *v,
            _ => unreachable!("expected Double, got {:?}", self),
        }
    }

    #[inline]
    pub fn extract_ref(&self) -> u32 {
        match self {
            Oop::Ref(slot_id) => *slot_id,
            t => unreachable!("expected Ref, got {:?}", t),
        }
    }
}

// Hash code
impl Oop {
    pub fn hash_code(&self) -> i32 {
        match self {
            Oop::Ref(slot_id) => {
                if Self::is_java_lang_string(*slot_id) {
                    Self::java_lang_string_hash(*slot_id)
                } else {
                    *slot_id as i32
                }
            }
            Oop::Null => 0,
            _ => unreachable!(),
        }
    }
}

// Helper methods for java.lang.String operations
impl Oop {
    pub fn is_java_lang_string(slot_id: u32) -> bool {
        with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            match &guard.v {
                RefKind::Inst(inst) => {
                    let cls = inst.class.get_class();
                    cls.name.as_slice() == b"java/lang/String"
                }
                _ => false,
            }
        })
    }

    pub fn java_lang_string(slot_id: u32) -> String {
        let v = Self::java_lang_string_value(slot_id);
        String::from_utf16_lossy(&v)
    }

    pub fn java_lang_string_value(slot_id: u32) -> Vec<u16> {
        let offset = get_java_lang_string_value_offset();
        let v = Class::get_field_value2(slot_id, offset);
        let slot_id2 = v.extract_ref();
        with_heap(|heap| {
            let desc = heap.get(slot_id2);
            let guard = desc.read().unwrap();
            let ary = guard.v.extract_type_array();
            // JDK 9+: String.value is byte[], JDK 8: char[]
            match ary {
                TypeArrayDesc::Char(chars) => chars.to_vec(),
                TypeArrayDesc::Byte(bytes) => bytes.iter().map(|&b| b as u16).collect(),
                _ => unreachable!(),
            }
        })
    }

    pub fn java_lang_string_hash(slot_id: u32) -> i32 {
        let offset = get_java_lang_string_value_offset();
        let v = Class::get_field_value2(slot_id, offset);
        let slot_id2 = v.extract_ref();
        with_heap(|heap| {
            let desc = heap.get(slot_id2);
            let guard = desc.read().unwrap();
            let chars = guard.v.extract_type_array().extract_chars();

            let mut h = 0i32;
            for v in chars.iter() {
                h = h.wrapping_mul(31).wrapping_add(*v as i32);
            }
            h
        })
    }

    pub fn java_lang_integer_value(slot_id: u32) -> i32 {
        let offset = get_java_lang_integer_value_offset();
        Class::get_field_value2(slot_id, offset).extract_int()
    }

    pub fn java_lang_thread_eetop(slot_id: u32) -> i64 {
        let inst = with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            guard.v.extract_inst().class.clone()
        });
        let cls = inst.get_class();
        let fid = cls.get_field_id(&new_br("eetop"), &new_br("J"), false);
        Class::get_field_value2(slot_id, fid.offset).extract_long()
    }

    /// Extract the mirror target ClassRef and value_type from a java.lang.Class oop.
    pub fn mirror_target_and_vt(slot_id: u32) -> (Option<ClassRef>, ValueType) {
        with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            let mirror = guard.v.extract_mirror();
            (mirror.target.clone(), mirror.value_type)
        })
    }

    /// Extract just the mirror target ClassRef from a java.lang.Class oop.
    pub fn mirror_target(slot_id: u32) -> Option<ClassRef> {
        with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            guard.v.extract_mirror().target.clone()
        })
    }

    /// Monitor operations — acquire the monitor for this object and notify all waiters.
    pub fn notify_all(&self) {
        let slot_id = match self {
            Oop::Ref(id) => *id,
            _ => return,
        };
        with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            guard.notify_all();
        })
    }

    /// Wait indefinitely on this object's monitor.
    pub fn wait(&self) {
        let slot_id = match self {
            Oop::Ref(id) => *id,
            _ => return,
        };
        with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            guard.wait();
        })
    }

    /// Wait with a timeout on this object's monitor.
    pub fn wait_timeout(&self, dur: std::time::Duration) {
        let slot_id = match self {
            Oop::Ref(id) => *id,
            _ => return,
        };
        with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            guard.wait_timeout(dur);
        })
    }

    /// Object identity equality. Also compares java.lang.String content.
    pub fn is_eq(l: u32, r: u32) -> bool {
        if with_heap(|heap| heap.is_same_slot(l, r)) {
            return true;
        }
        if Self::is_java_lang_string(l) && Self::is_java_lang_string(r) {
            Self::is_java_lang_string_eq(l, r)
        } else {
            false
        }
    }

    fn is_java_lang_string_eq(l: u32, r: u32) -> bool {
        let offset = get_java_lang_string_value_offset();
        let v1 = Class::get_field_value2(l, offset);
        let v2 = Class::get_field_value2(r, offset);
        let slot1 = v1.extract_ref();
        let slot2 = v2.extract_ref();
        with_heap(|heap| {
            let desc1 = heap.get(slot1);
            let guard1 = desc1.read().unwrap();
            let ary1 = guard1.v.extract_type_array();
            let desc2 = heap.get(slot2);
            let guard2 = desc2.read().unwrap();
            let ary2 = guard2.v.extract_type_array();
            // JDK 9+: String.value is byte[], JDK 8: char[]
            match (ary1, ary2) {
                (TypeArrayDesc::Char(c1), TypeArrayDesc::Char(c2)) => c1 == c2,
                (TypeArrayDesc::Byte(b1), TypeArrayDesc::Byte(b2)) => b1 == b2,
                _ => false,
            }
        })
    }
}

pub fn init() {
    consts::init();
}
