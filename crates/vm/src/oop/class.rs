use std::fmt::{self, Debug, Error, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap;

use classfile::{
    attributes::EnclosingMethod, attributes::InnerClass, constant_pool,
    constant_pool::get_utf8 as get_cp_utf8, consts, flags::*, AttributeType, BytesRef, U2,
};

use crate::oop::{self, consts as oop_consts, field, Oop, OopPtr, RefKindDesc, ValueType};
use crate::runtime::method::MethodId;
use crate::runtime::thread::ReentrantMutex;
use crate::runtime::{
    self, method, require_class2, ClassLoader, ConstantPoolCache, JavaCall, JavaThread,
};
use crate::types::*;
use crate::{native, util};

pub struct ClassPtr(u64);

impl ClassPtr {
    pub fn new(v: Class) -> ClassRef {
        let v = Box::new(v);
        let ptr = Box::into_raw(v) as u64;
        Arc::new(ClassPtr(ptr))
    }
}

impl Drop for ClassPtr {
    fn drop(&mut self) {
        let _v = unsafe { Box::from_raw(self.0 as *mut Class) };
    }
}

impl ClassPtr {
    fn raw_ptr(&self) -> *const Class {
        self.0 as *const Class
    }

    fn raw_mut_ptr(&self) -> *mut Class {
        self.0 as *mut Class
    }
}

impl ClassPtr {
    pub fn name(&self) -> BytesRef {
        let ptr = self.raw_ptr();
        unsafe { (*ptr).name.clone() }
    }

    pub fn get_class(&self) -> &Class {
        let ptr = self.raw_ptr();
        unsafe { &(*ptr) }
    }

    pub fn get_mut_class(&self) -> &mut Class {
        let ptr = self.raw_mut_ptr();
        unsafe { &mut (*ptr) }
    }

    pub fn extract_inst(&self) -> &ClassObject {
        let class = self.get_class();
        match &class.kind {
            oop::ClassKind::Instance(cls_obj) => cls_obj,
            _ => unreachable!(),
        }
    }
}

impl Debug for ClassPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cls = self.get_class();
        let cls_name = unsafe { std::str::from_utf8_unchecked(cls.name.as_slice()) };
        let cls_name = cls_name.to_string();
        let cls_kind_type = format!("{:?}", cls.get_class_kind_type());
        let cls_state = format!("{:?}", cls.get_class_state());
        f.debug_struct("ClassPtr")
            .field("name", &cls_name)
            .field("state", &cls_state)
            .field("kind", &cls_kind_type)
            .finish()
    }
}

/////////////////////////////////////////////

pub struct Class {
    clinit_mutex: Arc<std::sync::Mutex<()>>,
    mutex: ReentrantMutex,
    state: std::sync::atomic::AtomicU8,

    pub name: BytesRef,
    pub acc_flags: U2,

    // None for java.lang.Object
    pub super_class: Option<ClassRef>,

    // None for the "bootstrap" loader
    pub class_loader: Option<ClassLoader>,

    pub kind: ClassKind,
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

    //  FxHashMap<(name, desc), MethodIdRef>
    pub all_methods: FxHashMap<(BytesRef, BytesRef), MethodIdRef>,
    v_table: FxHashMap<(BytesRef, BytesRef), MethodIdRef>,

    //  FxHashMap<(package, name, desc), FieldIdRef>
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

    //valid when dimension > 1
    pub down_type: Option<ClassRef>,

    //valid when it's not TypeArray
    pub component: Option<ClassRef>,

    pub mirror: Option<Oop>,
}

pub fn init_class(class: &ClassRef) {
    let need = { class.get_class().get_class_state() == State::Linked };
    if need {
        let mut cls = class.get_mut_class();
        let clinit_mutex = cls.clinit_mutex.clone();
        let l = clinit_mutex.lock().unwrap();

        cls.set_class_state(State::BeingIni);
        if let Some(super_class) = &cls.super_class {
            init_class(super_class);
            init_class_fully(super_class);
        }

        match &mut cls.kind {
            ClassKind::Instance(class_obj) => {
                class_obj.init_static_fields();
            }

            _ => cls.set_class_state(State::FullyIni),
        }
    }
}

//invoke "<clinit>"
pub fn init_class_fully(class: &ClassRef) {
    let need = { class.get_class().get_class_state() == State::BeingIni };

    if need {
        let l = class.get_class().clinit_mutex.lock();

        let (mir, name) = {
            let mut class = class.get_mut_class();
            class.set_class_state(State::FullyIni);

            let mir = class.get_this_class_method(&util::S_CLINIT, &util::S_CLINIT_SIG);
            (mir, class.name.clone())
        };

        if let Ok(mir) = mir {
            info!("call {}:<clinit>", unsafe {
                std::str::from_utf8_unchecked(name.as_slice())
            });
            let mut jc = JavaCall::new_with_args(mir, vec![]);
            jc.invoke(None, true);
        }
    }
}

pub fn load_and_init(name: &[u8]) -> ClassRef {
    // trace!("load_and_init 1 name={}", String::from_utf8_lossy(name));
    let cls_name = unsafe { std::str::from_utf8_unchecked(name) };
    let class = runtime::require_class3(None, name)
        .unwrap_or_else(|| panic!("Class not found: {}", cls_name));

    init_class(&class);
    init_class_fully(&class);

    class
}

impl Class {
    pub fn get_class_state(&self) -> State {
        let v = self.state.load(Ordering::Relaxed);
        State::from(v)
    }

    pub fn set_class_state(&mut self, s: State) {
        self.state.store(s.into(), Ordering::Relaxed);
    }

    pub fn get_name(&self) -> BytesRef {
        self.name.clone()
    }

    pub fn get_super_class(&self) -> Option<ClassRef> {
        self.super_class.clone()
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
        unsafe {
            self.mutex.lock();
        }
    }

    pub fn monitor_exit(&self) {
        unsafe {
            self.mutex.unlock();
        }
    }

    pub fn link_class(&mut self, self_ref: ClassRef) {
        match &mut self.kind {
            ClassKind::Instance(class_obj) => {
                self.super_class = class_obj.link_super_class(self.name.clone(), self.class_loader);
                let n = match &self.super_class {
                    Some(super_cls) => {
                        let super_cls = super_cls.get_class();
                        match &super_cls.kind {
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
            }

            ClassKind::ObjectArray(ary_class_obj) => {
                let super_class = runtime::require_class3(None, consts::J_OBJECT).unwrap();
                self.super_class = Some(super_class);
            }

            ClassKind::TypeArray(ary_class_obj) => {
                let super_class = runtime::require_class3(None, consts::J_OBJECT).unwrap();
                self.super_class = Some(super_class);
            }
        }

        self.set_class_state(State::Linked);
    }

    pub fn get_class_kind_type(&self) -> ClassKindType {
        match &self.kind {
            ClassKind::Instance(_) => ClassKindType::Instance,
            ClassKind::ObjectArray(_) => ClassKindType::ObjectAry,
            ClassKind::TypeArray(_) => ClassKindType::TypAry,
        }
    }

    pub fn is_instance(&self) -> bool {
        match &self.kind {
            ClassKind::Instance(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match &self.kind {
            ClassKind::Instance(_) => false,
            _ => true,
        }
    }

    pub fn is_object_ary(&self) -> bool {
        match &self.kind {
            ClassKind::Instance(_) => false,
            ClassKind::TypeArray(_) => false,
            ClassKind::ObjectArray(_) => true,
        }
    }

    pub fn get_mirror(&self) -> Oop {
        match &self.kind {
            ClassKind::Instance(cls_obj) => cls_obj.mirror.clone().unwrap(),
            //[J
            ClassKind::TypeArray(typ_ary) => typ_ary.mirror.clone().unwrap(),
            //[Ljava/lang/Object;
            ClassKind::ObjectArray(obj_ary) => obj_ary.mirror.clone().unwrap(),
            _ => unreachable!(),
        }
    }

    pub fn set_mirror(&mut self, mirror: Oop) {
        match &mut self.kind {
            ClassKind::Instance(cls_obj) => cls_obj.mirror = Some(mirror),
            ClassKind::ObjectArray(obj_ary) => obj_ary.mirror = Some(mirror),
            ClassKind::TypeArray(typ_ary) => typ_ary.mirror = Some(mirror),
        }
    }

    pub fn get_source_file(&self) -> Option<BytesRef> {
        match &self.kind {
            ClassKind::Instance(cls_obj) => cls_obj.source_file.clone(),
            _ => unreachable!(),
        }
    }

    pub fn get_annotation(&self) -> Option<Vec<u8>> {
        match &self.kind {
            ClassKind::Instance(cls) => {
                util::attributes::assemble_annotation(&cls.class_file.attrs)
            }
            _ => unreachable!(),
        }
    }

    pub fn get_type_annotation(&self) -> Option<Vec<u8>> {
        match &self.kind {
            ClassKind::Instance(cls) => {
                util::attributes::assemble_type_annotation(&cls.class_file.attrs)
            }
            _ => unreachable!(),
        }
    }

    pub fn get_attr_signatrue(&self) -> Option<&BytesRef> {
        match &self.kind {
            ClassKind::Instance(cls) => {
                let idx = util::attributes::get_signature(&cls.class_file.attrs);
                if idx != 0 {
                    let cp = &cls.class_file.cp;
                    let s = get_cp_utf8(cp, idx as usize);
                    Some(s)
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }
}

impl ArrayClassObject {
    pub fn get_dimension(&self) -> usize {
        match self.down_type.as_ref() {
            Some(down_type) => {
                let down_type = down_type.get_class();
                let n = match &down_type.kind {
                    ClassKind::Instance(_) => unreachable!(),
                    ClassKind::ObjectArray(ary_cls_obj) => ary_cls_obj.get_dimension(),
                    ClassKind::TypeArray(ary_cls_obj) => ary_cls_obj.get_dimension(),
                };
                1 + n
            }
            None => 1,
        }
    }
}

//open api
impl Class {
    //todo: confirm static method
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

        if is_static {
            match &self.kind {
                ClassKind::Instance(cls_obj) => {
                    if let Some(fid) = cls_obj.static_fields.get(&k) {
                        return fid.clone();
                    }
                }
                _ => unreachable!(),
            }
        } else {
            match &self.kind {
                ClassKind::Instance(cls_obj) => {
                    if let Some(fid) = cls_obj.inst_fields.get(&k) {
                        return fid.clone();
                    }
                }
                _ => unreachable!(),
            }
        }

        let super_class = self.super_class.clone();
        super_class
            .unwrap()
            .get_class()
            .get_field_id(name, desc, is_static)
    }

    pub fn put_field_value(rf: Arc<OopPtr>, fir: FieldIdRef, v: Oop) {
        Self::put_field_value2(rf, fir.offset, v);
    }

    pub fn put_field_value2(rf: Arc<OopPtr>, offset: usize, v: Oop) {
        let ptr = rf.get_mut_raw_ptr();
        unsafe {
            match &mut (*ptr).v {
                oop::RefKind::Inst(inst) => inst.field_values[offset] = v,
                oop::RefKind::Mirror(mirror) => mirror.field_values[offset] = v,
                oop::RefKind::Array(ary) => ary.elements[offset] = v,
                t => unreachable!("t = {:?}", t),
            }
        }
    }

    pub fn get_field_value(rf: Arc<OopPtr>, fid: FieldIdRef) -> Oop {
        Self::get_field_value2(rf, fid.offset)
    }

    pub fn get_field_value2(rf: Arc<OopPtr>, offset: usize) -> Oop {
        unsafe {
            let ptr = rf.get_raw_ptr();
            match &(*ptr).v {
                oop::RefKind::Inst(inst) => inst.field_values[offset].clone(),
                oop::RefKind::Mirror(mirror) => match mirror.field_values.get(offset) {
                    Some(v) => v.clone(),
                    _ => unreachable!("mirror = {:?}", mirror),
                },
                oop::RefKind::Array(ary) => ary.elements[offset].clone(),
                t => unreachable!("t = {:?}", t),
            }
        }
    }

    pub fn put_static_field_value(&mut self, fid: FieldIdRef, v: Oop) {
        match &mut self.kind {
            ClassKind::Instance(cls_obj) => {
                let k = (
                    self.name.clone(),
                    fid.field.name.clone(),
                    fid.field.desc.clone(),
                );
                if cls_obj.static_fields.contains_key(&k) {
                    cls_obj.static_field_values[fid.offset] = v;
                } else {
                    let super_class = self.super_class.clone();
                    super_class
                        .unwrap()
                        .get_mut_class()
                        .put_static_field_value(fid, v);
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn get_static_field_value(&self, fid: FieldIdRef) -> Oop {
        match &self.kind {
            ClassKind::Instance(cls_obj) => {
                let k = (
                    self.name.clone(),
                    fid.field.name.clone(),
                    fid.field.desc.clone(),
                );
                if cls_obj.static_fields.contains_key(&k) {
                    cls_obj.static_field_values[fid.offset].clone()
                } else {
                    let super_class = self.super_class.clone();
                    super_class.unwrap().get_class().get_static_field_value(fid)
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn check_interface(&self, intf: ClassRef) -> bool {
        match &self.kind {
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

        match &self.super_class {
            Some(super_cls) => {
                let super_cls = super_cls.get_class();
                super_cls.check_interface(intf)
            }
            None => false,
        }
    }

    pub fn hack_as_native(&mut self, name: &[u8], desc: &[u8]) {
        match &mut self.kind {
            ClassKind::Instance(cls) => {
                let m = {
                    let name = Arc::new(Vec::from(name));
                    let desc = Arc::new(Vec::from(desc));
                    let k = (name, desc);
                    let it = cls.all_methods.get_mut(&k).unwrap();
                    let mut method = it.method.clone();
                    method.acc_flags |= ACC_NATIVE;
                    let m = method::MethodId::new(it.offset, method);
                    cls.all_methods.insert(k, m.clone());

                    m
                };

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

//open api new
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

        let mutex = unsafe {
            let mut mutex = ReentrantMutex::uninitialized();
            mutex.init();
            mutex
        };

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name,
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags,
            super_class: None,
            class_loader,
            kind: ClassKind::Instance(class_obj),
            mutex,
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

        let mutex = unsafe {
            let mut mutex = ReentrantMutex::uninitialized();
            mutex.init();
            mutex
        };

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name,
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            kind: ClassKind::ObjectArray(ary_cls_obj),
            mutex,
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

        let mutex = unsafe {
            let mut mutex = ReentrantMutex::uninitialized();
            mutex.init();
            mutex
        };

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name: Arc::new(name),
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            kind: ClassKind::TypeArray(ary_cls_obj),
            mutex,
        }
    }

    pub fn new_wrapped_ary(class_loader: ClassLoader, down_type: ClassRef) -> Self {
        let cls = down_type.get_class();
        debug_assert!(cls.is_array());

        //build name
        let mut name2 = Vec::with_capacity(1 + cls.name.len());
        name2.push(b'[');
        name2.extend_from_slice(cls.name.as_slice());

        let kind = match cls.get_class_kind_type() {
            ClassKindType::Instance => unreachable!(),
            ClassKindType::TypAry => ClassKind::TypeArray(ArrayClassObject {
                value_type: ValueType::ARRAY,
                down_type: Some(down_type),
                component: None,
                mirror: None,
            }),
            ClassKindType::ObjectAry => {
                let component = {
                    let cls = down_type.get_class();
                    match &cls.kind {
                        ClassKind::ObjectArray(ary_cls) => ary_cls.component.clone(),
                        _ => unreachable!(),
                    }
                };
                ClassKind::ObjectArray(ArrayClassObject {
                    value_type: ValueType::ARRAY,
                    down_type: Some(down_type),
                    component,
                    mirror: None,
                })
            }
        };

        let mutex = unsafe {
            let mut mutex = ReentrantMutex::uninitialized();
            mutex.init();
            mutex
        };

        Self {
            clinit_mutex: Arc::new(Mutex::new(())),
            name: Arc::new(name2),
            state: std::sync::atomic::AtomicU8::new(State::Allocated.into()),
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            kind,
            mutex,
        }
    }
}

//inner api for link
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

impl Class {
    pub fn get_class_method_inner(
        &self,
        name: &BytesRef,
        desc: &BytesRef,
        with_super: bool,
    ) -> Result<MethodIdRef, ()> {
        let k = (name.clone(), desc.clone());
        match &self.kind {
            ClassKind::Instance(cls_obj) => {
                if let Some(m) = cls_obj.all_methods.get(&k) {
                    return Ok(m.clone());
                }
            }
            ClassKind::ObjectArray(ary) => {
                //use java/lang/Object, methods
            }
            _ => unreachable!(),
        }

        if with_super {
            match self.super_class.as_ref() {
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
        match &self.kind {
            ClassKind::Instance(cls_obj) => {
                if let Some(m) = cls_obj.v_table.get(&k) {
                    return Ok(m.clone());
                }
            }
            _ => unreachable!(),
        }

        match self.super_class.as_ref() {
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
        match &self.kind {
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

        match self.super_class.as_ref() {
            Some(super_class) => super_class
                .get_class()
                .get_interface_method_inner(name, desc),
            None => Err(()),
        }
    }
}
