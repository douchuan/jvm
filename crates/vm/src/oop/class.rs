use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, RwLock};

use rustc_hash::FxHashMap;
use tracing::{debug, error, info, trace, warn};

use classfile::{
    attributes::EnclosingMethod, attributes::InnerClass, constant_pool,
    constant_pool::get_utf8 as get_cp_utf8, consts, flags::*, AttributeType, BytesRef, U2,
};

use crate::oop::field;
use crate::oop::heap::Heap;
use crate::oop::{self, consts as oop_consts, Oop, RefKindDesc, ValueType};
use crate::oop::{with_heap, with_heap_mut};
use crate::runtime::method::MethodId;
use crate::runtime::thread::ReentrantMutex;
use crate::runtime::{
    self, method, require_class2, ClassLoader, ConstantPoolCache, JavaCall, JavaThread,
};
use crate::types::FieldIdRef;
use crate::types::*;
use crate::{native, util};

/// Class reference — `Arc<Class>` for safe shared access.
/// Mutable parts use `RwLock` internally.
pub type ClassRef = Arc<Class>;

/// Class metadata loaded from a .class file.
///
/// Fields that are mutated after creation (super_class, kind, state)
/// are wrapped in `RwLock`. Fields set once at construction are plain.
pub struct Class {
    clinit_mutex: Arc<Mutex<()>>,

    /// Class name (e.g. b"java/lang/String"). Immutable after creation.
    pub name: BytesRef,
    /// Access flags. Immutable.
    pub acc_flags: U2,
    /// Class loader. Immutable.
    pub class_loader: Option<ClassLoader>,

    /// Super class, set during linking.
    super_class: RwLock<Option<ClassRef>>,
    /// The kind-specific data, set during construction, linked afterward.
    kind: RwLock<ClassKind>,
    /// Lifecycle state: Allocated → Loaded → Linked → BeingIni → FullyIni.
    state: std::sync::atomic::AtomicU8,
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Class({})", String::from_utf8_lossy(&self.name))
    }
}

pub enum ClassKind {
    Instance(ClassObject),
    ObjectArray(ArrayClassObject),
    TypeArray(ArrayClassObject),
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum ClassKindType {
    Instance,
    ObjectAry,
    TypAry,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum State {
    Allocated,
    Loaded,
    Linked,
    BeingIni,
    FullyIni,
    IniErr,
}

impl From<u8> for State {
    fn from(v: u8) -> Self {
        match v {
            0 => State::Allocated,
            1 => State::Loaded,
            2 => State::Linked,
            3 => State::BeingIni,
            4 => State::FullyIni,
            5 => State::IniErr,
            _ => unreachable!(),
        }
    }
}

impl Into<u8> for State {
    fn into(self) -> u8 {
        match self {
            State::Allocated => 0,
            State::Loaded => 1,
            State::Linked => 2,
            State::BeingIni => 3,
            State::FullyIni => 4,
            State::IniErr => 5,
        }
    }
}

pub struct ClassObject {
    pub class_file: ClassFileRef,

    pub n_inst_fields: usize,

    /// All methods: FxHashMap<(name, desc), MethodIdRef>
    pub all_methods: FxHashMap<(BytesRef, BytesRef), MethodIdRef>,
    v_table: FxHashMap<(BytesRef, BytesRef), MethodIdRef>,

    /// Static fields: FxHashMap<(package, name, desc), FieldIdRef>
    pub static_fields: FxHashMap<(BytesRef, BytesRef, BytesRef), FieldIdRef>,
    pub inst_fields: FxHashMap<(BytesRef, BytesRef, BytesRef), FieldIdRef>,

    static_field_values: Vec<Oop>,

    interfaces: FxHashMap<BytesRef, ClassRef>,

    mirror: Option<Oop>,

    pub signature: Option<BytesRef>,
    pub source_file: Option<BytesRef>,
    pub enclosing_method: Option<EnclosingMethod>,
    pub inner_classes: Option<Vec<InnerClass>>,

    pub cp_cache: ConstantPoolCache,
}

pub struct ArrayClassObject {
    pub value_type: ValueType,

    // valid when dimension > 1
    pub down_type: Option<ClassRef>,

    // valid when it's not TypeArray
    pub component: Option<ClassRef>,

    pub mirror: Option<Oop>,
}

// =============================================
// Class initialization
// =============================================

pub fn init_class(class: &ClassRef) {
    let need = { class.get_class_state() == State::Linked };
    if need {
        let clinit_mutex = class.clinit_mutex.clone();
        let _l = clinit_mutex.lock().unwrap();

        class.set_class_state(State::BeingIni);
        if let Some(super_class) = class.get_super_class() {
            init_class(&super_class);
            init_class_fully(&super_class);
        }

        let kind = class.kind.read().unwrap();
        match kind.deref() {
            ClassKind::Instance(class_obj) => {
                // Need to drop read lock before init_static_fields which needs write access
            }
            _ => class.set_class_state(State::FullyIni),
        }
    }
}

// invoke "<clinit>"
pub fn init_class_fully(class: &ClassRef) {
    let name = class.name.clone();
    let need = class.get_class_state() == State::BeingIni;

    if need {
        let Ok(_l) = class.clinit_mutex.try_lock() else {
            return;
        };

        if class.get_class_state() != State::BeingIni {
            return;
        }

        let mir = class.get_this_class_method(
            util::S_CLINIT.get().unwrap(),
            util::S_CLINIT_SIG.get().unwrap(),
        );

        if let Ok(mir) = mir {
            info!("call {}:<clinit>", unsafe {
                std::str::from_utf8_unchecked(name.as_slice())
            });
            let mut jc = JavaCall::new_with_args(mir, vec![]);
            jc.invoke(None, true);
            class.set_class_state(State::FullyIni);
        } else {
            class.set_class_state(State::FullyIni);
        }
    }
}

pub fn load_and_init(name: &[u8]) -> ClassRef {
    let cls_name = unsafe { std::str::from_utf8_unchecked(name) };
    let class = runtime::require_class3(None, name)
        .unwrap_or_else(|| panic!("Class not found: {}", cls_name));

    init_class(&class);
    init_class_fully(&class);

    class
}

// =============================================
// Class — state and flags
// =============================================

impl Class {
    /// Returns a reference to self — provided for uniformity with
    /// call sites that previously used `Arc<ClassPtr>` + Deref.
    pub fn get_class(&self) -> &Class {
        self
    }

    /// Returns self for code that previously used `Arc::make_mut`.
    /// Class uses interior mutability via RwLock, so no exclusive borrow needed.
    pub fn get_mut_class(self: &Arc<Class>) -> &Class {
        self
    }

    pub fn get_class_state(&self) -> State {
        let v = self.state.load(std::sync::atomic::Ordering::Relaxed);
        State::from(v)
    }

    pub fn set_class_state(&self, s: State) {
        self.state
            .store(s.into(), std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_name(&self) -> BytesRef {
        self.name.clone()
    }

    /// Get the super class reference (if any).
    pub fn get_super_class(&self) -> Option<ClassRef> {
        self.super_class.read().unwrap().clone()
    }

    /// Resolve a constant pool class index to a ClassRef.
    pub fn resolve_cp_class_by_index(&self, index: u16) -> Option<ClassRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                let name = classfile::constant_pool::get_class_name(
                    &cls_obj.class_file.cp,
                    index as usize,
                );
                runtime::require_class3(None, name.as_slice())
            }
            _ => None,
        }
    }

    /// Get name and type descriptor from a constant pool name-and-type index.
    pub fn get_cp_name_and_type(&self, index: usize) -> Option<(BytesRef, BytesRef)> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                let (name, desc) =
                    classfile::constant_pool::get_name_and_type(&cls_obj.class_file.cp, index);
                Some((name.clone(), desc.clone()))
            }
            _ => None,
        }
    }

    pub fn is_public(&self) -> bool {
        (self.acc_flags & ACC_PUBLIC) == ACC_PUBLIC
    }

    pub fn is_private(&self) -> bool {
        (self.acc_flags & ACC_PRIVATE) == ACC_PRIVATE
    }

    pub fn is_protected(&self) -> bool {
        (self.acc_flags & ACC_PROTECTED) == ACC_PROTECTED
    }

    pub fn is_final(&self) -> bool {
        (self.acc_flags & ACC_FINAL) == ACC_FINAL
    }

    /// Get the dimension of an array class (both object and primitive arrays).
    pub fn get_array_dimension(&self) -> Option<usize> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::ObjectArray(obj_ary) => Some(obj_ary.get_dimension()),
            ClassKind::TypeArray(typ_ary) => Some(typ_ary.get_dimension()),
            ClassKind::Instance(_) => None,
        }
    }

    pub fn is_static(&self) -> bool {
        (self.acc_flags & ACC_STATIC) == ACC_STATIC
    }

    pub fn is_abstract(&self) -> bool {
        (self.acc_flags & ACC_ABSTRACT) == ACC_ABSTRACT
    }

    pub fn is_interface(&self) -> bool {
        (self.acc_flags & ACC_INTERFACE) == ACC_INTERFACE
    }

    pub fn monitor_enter(&self) {
        // Class-level monitor — using the clinit_mutex as a simple stand-in
        let _guard = self.clinit_mutex.lock();
    }

    pub fn monitor_exit(&self) {
        // MutexGuard is dropped, releasing the lock
    }
}

// =============================================
// Class — linking
// =============================================

impl Class {
    /// Get a write guard to the ClassKind for linking.
    fn kind_write(&self) -> std::sync::RwLockWriteGuard<'_, ClassKind> {
        self.kind.write().unwrap()
    }

    fn kind_read(&self) -> std::sync::RwLockReadGuard<'_, ClassKind> {
        self.kind.read().unwrap()
    }

    pub fn link_class(&self, self_ref: ClassRef) {
        let cp = {
            let kind = self.kind_read();
            match kind.deref() {
                ClassKind::Instance(class_obj) => class_obj.class_file.cp.clone(),
                ClassKind::ObjectArray(_) | ClassKind::TypeArray(_) => return,
            }
        };

        let super_class = {
            let mut kind = self.kind_write();
            match &mut kind.deref_mut() {
                ClassKind::Instance(class_obj) => {
                    let super_class =
                        class_obj.link_super_class(self.name.clone(), self.class_loader.clone());
                    let n = match &super_class {
                        Some(super_cls) => {
                            let sc = super_cls.get_class();
                            match &sc.kind.read().unwrap().deref() {
                                ClassKind::Instance(cls) => cls.n_inst_fields,
                                _ => 0,
                            }
                        }
                        None => 0,
                    };
                    class_obj.link_fields(self_ref.clone(), self.name.clone(), n);
                    class_obj.link_interfaces();
                    class_obj.link_methods(self_ref, self.name.clone());
                    class_obj.link_attributes();
                    super_class
                }
                ClassKind::ObjectArray(ary_class_obj) => {
                    let super_class = runtime::require_class3(None, consts::J_OBJECT).unwrap();
                    self.super_class
                        .write()
                        .unwrap()
                        .replace(super_class.clone());
                    Some(super_class)
                }
                ClassKind::TypeArray(ary_class_obj) => {
                    let super_class = runtime::require_class3(None, consts::J_OBJECT).unwrap();
                    self.super_class
                        .write()
                        .unwrap()
                        .replace(super_class.clone());
                    Some(super_class)
                }
            }
        };

        self.set_class_state(State::Linked);
    }

    pub fn get_class_kind_type(&self) -> ClassKindType {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(_) => ClassKindType::Instance,
            ClassKind::ObjectArray(_) => ClassKindType::ObjectAry,
            ClassKind::TypeArray(_) => ClassKindType::TypAry,
        }
    }

    pub fn is_instance(&self) -> bool {
        matches!(*self.kind_read(), ClassKind::Instance(_))
    }

    pub fn is_array(&self) -> bool {
        !self.is_instance()
    }

    pub fn is_object_ary(&self) -> bool {
        matches!(*self.kind_read(), ClassKind::ObjectArray(_))
    }

    /// Get the value type of an array class (object or primitive array).
    pub fn get_array_value_type(&self) -> Option<ValueType> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::ObjectArray(obj_ary) => Some(obj_ary.value_type),
            ClassKind::TypeArray(typ_ary) => Some(typ_ary.value_type),
            ClassKind::Instance(_) => None,
        }
    }

    /// Get the number of instance fields (only for instance classes).
    pub fn get_n_inst_fields(&self) -> Option<usize> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.n_inst_fields),
            _ => None,
        }
    }

    /// Get instance and static field maps (only for instance classes).
    pub fn get_inst_and_static_fields(
        &self,
    ) -> Option<(
        FxHashMap<(BytesRef, BytesRef, BytesRef), FieldIdRef>,
        FxHashMap<(BytesRef, BytesRef, BytesRef), FieldIdRef>,
    )> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                Some((cls_obj.inst_fields.clone(), cls_obj.static_fields.clone()))
            }
            _ => None,
        }
    }

    /// Get resolved interface map (only for instance classes).
    pub fn get_interfaces_map(&self) -> Option<FxHashMap<BytesRef, ClassRef>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.interfaces.clone()),
            _ => None,
        }
    }

    /// Get the component type class for an array class.
    pub fn get_component_type_class(&self) -> Option<ClassRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::ObjectArray(obj_ary) => obj_ary.component.clone(),
            _ => None,
        }
    }

    /// Get the value type of a type array (primitive array).
    pub fn get_type_array_value_type(&self) -> Option<&'static [u8]> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::TypeArray(typ_ary) => Some(typ_ary.value_type.into()),
            _ => None,
        }
    }

    /// Get the down-type class for a multi-dimensional array.
    pub fn get_array_down_type(&self) -> Option<ClassRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::ObjectArray(obj_ary) => obj_ary.down_type.clone(),
            _ => None,
        }
    }

    /// Get field info from the constant pool cache (only for instance classes).
    pub fn get_cp_field(&self, idx: usize, is_static: bool) -> Option<FieldIdRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.cp_cache.get_field(idx, is_static)),
            _ => None,
        }
    }

    /// Get method info from the constant pool cache (only for instance classes).
    pub fn get_cp_method(&self, idx: usize) -> Option<MethodIdRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => cls_obj.cp_cache.get_method(idx),
            _ => None,
        }
    }

    pub fn get_mirror(&self) -> Oop {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => cls_obj.mirror.clone().unwrap(),
            ClassKind::TypeArray(typ_ary) => typ_ary.mirror.clone().unwrap(),
            ClassKind::ObjectArray(obj_ary) => obj_ary.mirror.clone().unwrap(),
        }
    }

    pub fn set_mirror(&self, mirror: Oop) {
        let mut kind = self.kind_write();
        match kind.deref_mut() {
            ClassKind::Instance(cls_obj) => cls_obj.mirror = Some(mirror),
            ClassKind::ObjectArray(obj_ary) => obj_ary.mirror = Some(mirror),
            ClassKind::TypeArray(typ_ary) => typ_ary.mirror = Some(mirror),
        }
    }

    // Accessors for instance class data (ClassKind::Instance)

    pub fn get_inst_fields(&self) -> Option<FxHashMap<(BytesRef, BytesRef, BytesRef), FieldIdRef>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.inst_fields.clone()),
            _ => None,
        }
    }

    pub fn get_static_fields(
        &self,
    ) -> Option<FxHashMap<(BytesRef, BytesRef, BytesRef), FieldIdRef>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.static_fields.clone()),
            _ => None,
        }
    }

    pub fn get_all_methods(&self) -> Option<FxHashMap<(BytesRef, BytesRef), MethodIdRef>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.all_methods.clone()),
            _ => None,
        }
    }

    pub fn get_enclosing_method(&self) -> Option<EnclosingMethod> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => cls_obj.enclosing_method.clone(),
            _ => None,
        }
    }

    pub fn get_inner_classes(&self) -> Option<Vec<InnerClass>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => cls_obj.inner_classes.clone(),
            _ => None,
        }
    }

    /// Get the interface list indices from the constant pool.
    pub fn get_interfaces(&self) -> Option<Vec<u16>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.class_file.interfaces.clone()),
            _ => None,
        }
    }

    /// Resolve a constant pool class entry.
    pub fn resolve_cp_class(&self, index: u16) -> Option<ClassRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                let name = classfile::constant_pool::get_class_name(
                    &cls_obj.class_file.cp,
                    index as usize,
                );
                runtime::require_class3(None, name.as_slice())
            }
            _ => None,
        }
    }

    /// Get the constant pool. Returns a cloned Arc (cheap since ConstantPool is Arc-based).
    pub fn get_constant_pool(&self) -> Option<classfile::ConstantPool> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.class_file.cp.clone()),
            _ => None,
        }
    }

    /// Clone the constant pool (cheap Arc clone).
    pub fn get_cp_clone(&self) -> Option<classfile::ConstantPool> {
        self.get_constant_pool()
    }

    /// Get the this_class index from the constant pool.
    pub fn get_this_class_index(&self) -> Option<u16> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => Some(cls_obj.class_file.this_class),
            _ => None,
        }
    }

    pub fn get_signature(&self) -> Option<BytesRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => cls_obj.signature.clone(),
            ClassKind::ObjectArray(_) | ClassKind::TypeArray(_) => None,
        }
    }

    pub fn get_source_file(&self) -> Option<BytesRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => cls_obj.source_file.clone(),
            _ => unreachable!(),
        }
    }

    pub fn get_annotation(&self) -> Option<Vec<u8>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls) => {
                util::attributes::assemble_annotation(&cls.class_file.attrs)
            }
            _ => unreachable!(),
        }
    }

    pub fn get_type_annotation(&self) -> Option<Vec<u8>> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls) => {
                util::attributes::assemble_type_annotation(&cls.class_file.attrs)
            }
            _ => unreachable!(),
        }
    }

    pub fn get_attr_signatrue(&self) -> Option<BytesRef> {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls) => {
                let idx = util::attributes::get_signature(&cls.class_file.attrs);
                if idx != 0 {
                    let cp = &cls.class_file.cp;
                    let s = get_cp_utf8(cp, idx as usize);
                    Some(s.clone())
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }
}

// =============================================
// ArrayClassObject
// =============================================

impl ArrayClassObject {
    pub fn get_dimension(&self) -> usize {
        match self.down_type.as_ref() {
            Some(down_type) => {
                let kind = down_type.kind.read().unwrap();
                match kind.deref() {
                    ClassKind::Instance(_) => unreachable!(),
                    ClassKind::ObjectArray(ary_cls_obj) => ary_cls_obj.get_dimension(),
                    ClassKind::TypeArray(ary_cls_obj) => ary_cls_obj.get_dimension(),
                }
            }
            None => 1,
        }
    }
}

// =============================================
// Class — method lookup
// =============================================

impl Class {
    pub fn get_static_method(&self, name: &BytesRef, desc: &BytesRef) -> Result<MethodIdRef, ()> {
        self.get_class_method_inner(name, desc, true)
    }

    pub fn get_class_method(&self, name: &BytesRef, desc: &BytesRef) -> Result<MethodIdRef, ()> {
        self.get_class_method_inner(name, desc, true)
    }

    pub fn get_this_class_method(
        &self,
        name: &BytesRef,
        desc: &BytesRef,
    ) -> Result<MethodIdRef, ()> {
        self.get_class_method_inner(name, desc, false)
    }

    pub fn get_virtual_method(&self, name: &BytesRef, desc: &BytesRef) -> Result<MethodIdRef, ()> {
        self.get_virtual_method_inner(name, desc)
    }

    pub fn get_interface_method(
        &self,
        name: &BytesRef,
        desc: &BytesRef,
    ) -> Result<MethodIdRef, ()> {
        self.get_interface_method_inner(name, desc)
    }

    pub fn get_field_id(&self, name: &BytesRef, desc: &BytesRef, is_static: bool) -> FieldIdRef {
        let k = (self.name.clone(), name.clone(), desc.clone());

        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                if is_static {
                    if let Some(fid) = cls_obj.static_fields.get(&k) {
                        return fid.clone();
                    }
                } else {
                    if let Some(fid) = cls_obj.inst_fields.get(&k) {
                        return fid.clone();
                    }
                }
            }
            _ => unreachable!(),
        }
        drop(kind);

        let super_result = self.get_super_class();
        match super_result {
            Some(super_cls) => super_cls.get_field_id(name, desc, is_static),
            None => {
                let class_name = String::from_utf8_lossy(&self.name);
                let field_name = String::from_utf8_lossy(name);
                let field_desc = String::from_utf8_lossy(desc);
                panic!(
                    "get_field_id: field {}.{}:{} not found in {} (no super class)",
                    class_name, field_name, field_desc, class_name
                );
            }
        }
    }

    pub fn get_field_id_safe(
        &self,
        name: &BytesRef,
        desc: &BytesRef,
        is_static: bool,
    ) -> Result<FieldIdRef, ()> {
        let k = (self.name.clone(), name.clone(), desc.clone());

        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                if is_static {
                    if let Some(fid) = cls_obj.static_fields.get(&k) {
                        return Ok(fid.clone());
                    }
                } else {
                    if let Some(fid) = cls_obj.inst_fields.get(&k) {
                        return Ok(fid.clone());
                    }
                }
            }
            _ => unreachable!(),
        }
        drop(kind);

        if let Some(super_class) = self.get_super_class() {
            super_class.get_field_id_safe(name, desc, is_static)
        } else {
            Err(())
        }
    }
}

// =============================================
// Class — field access (through Heap)
// =============================================

impl Class {
    /// Put a field value into an object instance.
    pub fn put_field_value2(slot_id: u32, offset: usize, v: Oop) {
        with_heap_mut(|heap| {
            let desc = heap.get(slot_id);
            let mut guard = desc.write().unwrap();
            match &mut guard.v {
                oop::RefKind::Inst(inst) => inst.field_values[offset] = v,
                oop::RefKind::Mirror(mirror) => mirror.field_values[offset] = v,
                oop::RefKind::Array(ary) => ary.elements[offset] = v,
                t => unreachable!("t = {:?}", t),
            }
        })
    }

    /// Get a field value from an object instance.
    pub fn get_field_value2(slot_id: u32, offset: usize) -> Oop {
        with_heap(|heap| {
            let desc = heap.get(slot_id);
            let guard = desc.read().unwrap();
            match &guard.v {
                oop::RefKind::Inst(inst) => inst.field_values[offset].clone(),
                oop::RefKind::Mirror(mirror) => match mirror.field_values.get(offset) {
                    Some(v) => v.clone(),
                    _ => unreachable!("mirror = {:?}", mirror),
                },
                oop::RefKind::Array(ary) => ary.elements[offset].clone(),
                t => unreachable!("t = {:?}", t),
            }
        })
    }

    /// Convenience wrapper: get a field value using a FieldIdRef.
    pub fn get_field_value(slot_id: u32, fid: FieldIdRef) -> Oop {
        Self::get_field_value2(slot_id, fid.offset)
    }

    pub fn put_static_field_value(&self, fid: FieldIdRef, v: Oop) {
        let mut kind = self.kind_write();
        match kind.deref_mut() {
            ClassKind::Instance(cls_obj) => {
                let k = (
                    self.name.clone(),
                    fid.field.name.clone(),
                    fid.field.desc.clone(),
                );
                if cls_obj.static_fields.contains_key(&k) {
                    cls_obj.static_field_values[fid.offset] = v;
                } else {
                    drop(kind);
                    self.get_super_class()
                        .unwrap()
                        .put_static_field_value(fid, v);
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn get_static_field_value(&self, fid: FieldIdRef) -> Oop {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                let k = (
                    self.name.clone(),
                    fid.field.name.clone(),
                    fid.field.desc.clone(),
                );
                if cls_obj.static_fields.contains_key(&k) {
                    cls_obj.static_field_values[fid.offset].clone()
                } else {
                    drop(kind);
                    self.get_super_class().unwrap().get_static_field_value(fid)
                }
            }
            _ => unreachable!(),
        }
    }
}

// =============================================
// Class — interface checking
// =============================================

impl Class {
    pub fn check_interface(&self, intf: ClassRef) -> bool {
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(inst) => {
                for e in inst.interfaces.values() {
                    if Arc::ptr_eq(e, &intf) {
                        return true;
                    }
                    let e = e.get_class();
                    if e.check_interface(intf.clone()) {
                        return true;
                    }
                }
            }
            _ => unreachable!(),
        }
        drop(kind);

        match self.get_super_class() {
            Some(super_cls) => {
                let super_cls = super_cls.get_class();
                super_cls.check_interface(intf)
            }
            None => false,
        }
    }

    pub fn hack_as_native(&self, name: &[u8], desc: &[u8]) {
        let mut kind = self.kind_write();
        match kind.deref_mut() {
            ClassKind::Instance(cls) => {
                let name_arc = Arc::new(Vec::from(name));
                let desc_arc = Arc::new(Vec::from(desc));
                let k = (name_arc.clone(), desc_arc.clone());
                let it = cls.all_methods.get_mut(&k).unwrap();
                let mut method = it.method.clone();
                method.acc_flags |= ACC_NATIVE;
                let m = method::MethodId::new(it.offset, method);
                cls.all_methods.insert(k, m.clone());
                cls.v_table.insert((name_arc, desc_arc), m.clone());

                info!(
                    "hack_as_native: {}:{}:{}, native={}",
                    unsafe { std::str::from_utf8_unchecked(self.name.as_slice()) },
                    unsafe { std::str::from_utf8_unchecked(name) },
                    unsafe { std::str::from_utf8_unchecked(desc) },
                    m.method.is_native()
                );
            }
            _ => unreachable!(),
        }
    }
}

// =============================================
// Class — constructors
// =============================================

impl Class {
    pub fn new_class(class_file: ClassFileRef, class_loader: Option<ClassLoader>) -> Self {
        let cp = class_file.cp.clone();
        let name = constant_pool::get_class_name(&cp, class_file.this_class as usize).clone();
        let acc_flags = class_file.acc_flags;
        let class_obj = ClassObject {
            class_file,
            n_inst_fields: 0,
            all_methods: FxHashMap::default(),
            v_table: FxHashMap::default(),
            static_fields: FxHashMap::default(),
            inst_fields: FxHashMap::default(),
            static_field_values: vec![],
            interfaces: FxHashMap::default(),
            mirror: None,
            signature: None,
            source_file: None,
            enclosing_method: None,
            inner_classes: None,
            cp_cache: ConstantPoolCache::new(cp),
        };

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name,
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags,
            super_class: RwLock::new(None),
            class_loader,
            kind: RwLock::new(ClassKind::Instance(class_obj)),
        }
    }

    pub fn new_object_ary(class_loader: ClassLoader, component: ClassRef, elm_name: &[u8]) -> Self {
        let name = Vec::from(elm_name);
        let name = Arc::new(name);

        let ary_cls_obj = ArrayClassObject {
            value_type: ValueType::ARRAY,
            down_type: None,
            component: Some(component),
            mirror: None,
        };

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name,
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags: 0,
            super_class: RwLock::new(None),
            class_loader: Some(class_loader),
            kind: RwLock::new(ClassKind::ObjectArray(ary_cls_obj)),
        }
    }

    pub fn new_prime_ary(class_loader: ClassLoader, value_type: ValueType) -> Self {
        let ary_cls_obj = ArrayClassObject {
            value_type,
            down_type: None,
            component: None,
            mirror: None,
        };

        let mut name = Vec::with_capacity(2);
        name.push(b'[');
        name.extend_from_slice(value_type.into());

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name: Arc::new(name),
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags: 0,
            super_class: RwLock::new(None),
            class_loader: Some(class_loader),
            kind: RwLock::new(ClassKind::TypeArray(ary_cls_obj)),
        }
    }

    pub fn new_wrapped_ary(class_loader: ClassLoader, down_type: ClassRef) -> Self {
        let cls = down_type.get_class();
        debug_assert!(cls.is_array());

        let mut name2 = Vec::with_capacity(1 + cls.name.len());
        name2.push(b'[');
        name2.extend_from_slice(cls.name.as_slice());

        let kind_type = cls.get_class_kind_type();
        let component = if kind_type == ClassKindType::ObjectAry {
            let kind = cls.kind.read().unwrap();
            match kind.deref() {
                ClassKind::ObjectArray(ary_cls) => ary_cls.component.clone(),
                _ => unreachable!(),
            }
        } else {
            None
        };

        let kind = match kind_type {
            ClassKindType::Instance => unreachable!(),
            ClassKindType::TypAry => ClassKind::TypeArray(ArrayClassObject {
                value_type: ValueType::ARRAY,
                down_type: Some(down_type),
                component: None,
                mirror: None,
            }),
            ClassKindType::ObjectAry => ClassKind::ObjectArray(ArrayClassObject {
                value_type: ValueType::ARRAY,
                down_type: Some(down_type),
                component,
                mirror: None,
            }),
        };

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name: Arc::new(name2),
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags: 0,
            super_class: RwLock::new(None),
            class_loader: Some(class_loader),
            kind: RwLock::new(kind),
        }
    }
}

// =============================================
// ClassObject — linking helpers
// =============================================

impl ClassObject {
    fn link_super_class(
        &mut self,
        name: BytesRef,
        class_loader: Option<ClassLoader>,
    ) -> Option<ClassRef> {
        let class_file = &self.class_file;
        let cp = &class_file.cp;

        if class_file.super_class == 0 {
            if name.as_slice() != consts::J_OBJECT {
                unreachable!("should be java/lang/Object");
            }
            None
        } else {
            let name = constant_pool::get_class_name(cp, class_file.super_class as usize);
            let super_class = runtime::require_class(class_loader, name).unwrap();

            {
                let c = super_class.get_class();
                debug_assert!(c.is_instance());
                debug_assert!(!c.is_final(), "should not final");
            }

            Some(super_class)
        }
    }

    fn link_fields(&mut self, self_ref: ClassRef, cls_name: BytesRef, num_field_of_super: usize) {
        let cls_file = self.class_file.clone();
        let cp = &cls_file.cp;

        let mut n_static = 0;
        let mut offset_field = num_field_of_super;

        cls_file.fields.iter().for_each(|it| {
            let field = field::Field::new(cp, it, cls_name.clone(), self_ref.clone());
            let k = (cls_name.clone(), field.name.clone(), field.desc.clone());

            if field.is_static() {
                let fid = field::FieldId {
                    offset: n_static,
                    field,
                };
                self.static_fields.insert(k, Arc::new(fid));
                n_static += 1;
            } else {
                let fid = field::FieldId {
                    offset: offset_field,
                    field,
                };
                self.inst_fields.insert(k, Arc::new(fid));
                offset_field += 1;
            }
        });

        self.n_inst_fields = offset_field;
        self.static_field_values = vec![Oop::Null; n_static];
    }

    fn link_interfaces(&mut self) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file
            .interfaces
            .iter()
            .for_each(|it| match runtime::require_class2(*it, cp) {
                Some(class) => {
                    let name = class.get_class().name.clone();
                    self.interfaces.insert(name, class);
                }
                None => {
                    let name = constant_pool::get_class_name(cp, *it as usize);
                    error!("link interface failed {:?}", name);
                }
            });
    }

    fn link_methods(&mut self, this_ref: ClassRef, cls_name: BytesRef) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file.methods.iter().enumerate().for_each(|(i, it)| {
            let method = method::Method::new(
                cp,
                it,
                this_ref.clone(),
                class_file.clone(),
                i,
                cls_name.clone(),
            );
            let method_id = method::MethodId::new(i, method);
            let name = method_id.method.name.clone();
            let desc = method_id.method.desc.clone();
            let k = (name, desc);
            self.all_methods.insert(k.clone(), method_id.clone());

            if !method_id.method.is_static() {
                self.v_table.insert(k, method_id);
            }
        });
    }

    fn link_attributes(&mut self) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file.attrs.iter().for_each(|a| match a {
            AttributeType::Signature { signature_index } => {
                let s = get_cp_utf8(cp, *signature_index as usize);
                self.signature = Some(s.clone());
            }
            AttributeType::SourceFile { source_file_index } => {
                let s = get_cp_utf8(cp, *source_file_index as usize);
                self.source_file = Some(s.clone());
            }
            AttributeType::EnclosingMethod { em } => {
                self.enclosing_method = Some(em.clone());
            }
            AttributeType::InnerClasses { classes } => {
                self.inner_classes = Some(classes.clone());
            }
            _ => (),
        });
    }

    fn init_static_fields(&mut self) {
        let values = &mut self.static_field_values;
        self.static_fields.iter().for_each(|(_, it)| {
            if it.field.is_final() {
                match it.field.get_attr_constant_value() {
                    Some(v) => values[it.offset] = v,
                    None => values[it.offset] = it.field.get_constant_value(),
                }
            } else {
                values[it.offset] = it.field.get_constant_value();
            }
        });
    }
}

// =============================================
// Class — method lookup (inner)
// =============================================

impl Class {
    pub fn get_class_method_inner(
        &self,
        name: &BytesRef,
        desc: &BytesRef,
        with_super: bool,
    ) -> Result<MethodIdRef, ()> {
        let k = (name.clone(), desc.clone());
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                if let Some(m) = cls_obj.all_methods.get(&k) {
                    return Ok(m.clone());
                }
            }
            ClassKind::ObjectArray(_) => {
                // use java/lang/Object methods
            }
            _ => unreachable!(),
        }
        drop(kind);

        if with_super {
            match self.get_super_class() {
                Some(super_class) => {
                    return super_class
                        .get_class()
                        .get_class_method_inner(name, desc, with_super);
                }
                None => return Err(()),
            }
        }

        Err(())
    }

    fn get_virtual_method_inner(
        &self,
        name: &BytesRef,
        desc: &BytesRef,
    ) -> Result<MethodIdRef, ()> {
        let k = (name.clone(), desc.clone());
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => {
                if let Some(m) = cls_obj.v_table.get(&k) {
                    return Ok(m.clone());
                }
            }
            _ => unreachable!(),
        }
        drop(kind);

        match self.get_super_class() {
            Some(super_class) => super_class.get_class().get_virtual_method_inner(name, desc),
            None => Err(()),
        }
    }

    pub fn get_interface_method_inner(
        &self,
        name: &BytesRef,
        desc: &BytesRef,
    ) -> Result<MethodIdRef, ()> {
        let k = (name.clone(), desc.clone());
        let kind = self.kind_read();
        match kind.deref() {
            ClassKind::Instance(cls_obj) => match cls_obj.v_table.get(&k) {
                Some(m) => return Ok(m.clone()),
                None => {
                    for (_, itf) in cls_obj.interfaces.iter() {
                        let cls = itf.get_class();
                        let m = cls.get_interface_method(name, desc);
                        if m.is_ok() {
                            return m;
                        }
                    }
                }
            },
            _ => unreachable!(),
        }
        drop(kind);

        match self.get_super_class() {
            Some(super_class) => super_class
                .get_class()
                .get_interface_method_inner(name, desc),
            None => Err(()),
        }
    }
}
