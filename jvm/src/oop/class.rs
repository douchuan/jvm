use class_parser::{
    AttributeType,
    constant_pool::get_utf8 as get_cp_utf8,
    types::{BytesRef, U2},
    attributes::EnclosingMethod, attributes::InnerClass, constant_pool, consts,
    flags::*,
};
use crate::oop::method::MethodId;
use crate::oop::{self, consts as oop_consts, field, method, Oop, ValueType};
use crate::runtime::{self, require_class2, ClassLoader, JavaCall, JavaThread};
use crate::types::*;
use crate::util;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

pub struct Class {
    pub name: BytesRef,
    pub state: State,
    pub acc_flags: U2,

    // None for java.lang.Object
    pub super_class: Option<ClassRef>,

    // None for the "bootstrap" loader
    pub class_loader: Option<ClassLoader>,

    monitor: Mutex<usize>,

    pub kind: ClassKind,
}

pub enum ClassKind {
    Instance(ClassObject),
    ObjectArray(ArrayClassObject),
    TypeArray(ArrayClassObject),
}

impl fmt::Debug for ClassKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClassKind::Instance(cls) => write!(f, "ClassKind::Instance"),
            ClassKind::ObjectArray(obj_ary) => write!(f, "ClassKind::ObjectArray"),
            ClassKind::TypeArray(typ_ar) => write!(f, "ClassKind::TypeArray"),
        }
    }
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Class({})",
            String::from_utf8_lossy(self.name.as_slice())
        )
    }
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

#[derive(Debug)]
pub struct ClassObject {
    pub class_file: ClassFileRef,

    pub n_inst_fields: usize,

    pub all_methods: HashMap<BytesRef, MethodIdRef>,
    v_table: HashMap<BytesRef, MethodIdRef>,

    pub static_fields: HashMap<BytesRef, FieldIdRef>,
    pub inst_fields: HashMap<BytesRef, FieldIdRef>,

    static_field_values: Vec<Oop>,

    interfaces: HashMap<BytesRef, ClassRef>,

    mirror: Option<Oop>,

    pub signature: Option<BytesRef>,
    pub source_file: Option<BytesRef>,
    pub enclosing_method: Option<EnclosingMethod>,
    pub inner_classes: Option<Vec<InnerClass>>,
}

#[derive(Debug)]
pub struct ArrayClassObject {
    pub value_type: ValueType,

    //valid when dimension > 1
    pub down_type: Option<ClassRef>,

    //valid when it's not TypeArray
    pub component: Option<ClassRef>,

    pub mirror: Option<Oop>,
}

//invoke "<clinit>"
pub fn init_class_fully(thread: &mut JavaThread, class: ClassRef) {
    let need = { class.read().unwrap().state == State::BeingIni };

    if need {
        let (mir, name) = {
            let mut class = class.write().unwrap();
            class.state = State::FullyIni;

            let id = util::new_method_id(b"<clinit>", b"()V");
            let mir = class.get_this_class_method(id);
            (mir, class.name.clone())
        };

        match mir {
            Ok(mir) => {
                info!("call {}:<clinit>", String::from_utf8_lossy(name.as_slice()));
                let area = runtime::DataArea::new(0, 0);
                let mut jc = JavaCall::new_with_args(thread, mir, vec![]);
                jc.invoke(thread, Some(&area), true);
            }
            _ => (),
        }
    }
}

pub fn load_and_init(jt: &mut JavaThread, name: &[u8]) -> ClassRef {
    // trace!("load_and_init 1 name={}", String::from_utf8_lossy(name));
    let cls_name = unsafe { std::str::from_utf8_unchecked(name) };
    let class = runtime::require_class3(None, name)
        .unwrap_or_else(|| panic!("Class not found: {}", cls_name));
    // trace!("load_and_init 2 name={}", String::from_utf8_lossy(name));
    {
        let mut class = class.write().unwrap();
        class.init_class(jt);
        //                trace!("finish init_class: {}", String::from_utf8_lossy(*c));
    }

    // trace!("load_and_init 3, name={}", String::from_utf8_lossy(name));
    init_class_fully(jt, class.clone());
    //            trace!("finish init_class_fully: {}", String::from_utf8_lossy(*c));
    // trace!("load_and_init 4, name={}", String::from_utf8_lossy(name));

    class
}

impl Class {
    pub fn get_class_state(&self) -> State {
        self.state
    }

    pub fn set_class_state(&mut self, s: State) {
        self.state = s;
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

    pub fn monitor_enter(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v += 1;
    }

    pub fn monitor_exit(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v -= 1;
    }

    pub fn link_class(&mut self, self_ref: ClassRef) {
        match &mut self.kind {
            ClassKind::Instance(class_obj) => {
                self.super_class =
                    class_obj.link_super_class(self.name.clone(), self.class_loader.clone());

                let n_super_inst = {
                    match &self.super_class {
                        Some(super_cls) => {
                            let super_cls = super_cls.read().unwrap();
                            match &super_cls.kind {
                                ClassKind::Instance(cls) => cls.n_inst_fields,
                                _ => 0,
                            }
                        }
                        None => 0,
                    }
                };

                class_obj.link_fields(self_ref.clone(), self.name.clone(), n_super_inst);

                class_obj.link_interfaces();
                class_obj.link_methods(self_ref);
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

    pub fn init_class(&mut self, thread: &mut JavaThread) {
        match &mut self.kind {
            ClassKind::Instance(class_obj) => {
                if self.state == State::Linked {
                    self.state = State::BeingIni;

                    if let Some(super_class) = self.super_class.as_ref() {
                        {
                            super_class.write().unwrap().init_class(thread);
                        }

                        init_class_fully(thread, super_class.clone());
                    }

                    class_obj.init_static_fields();
                }
            }

            _ => self.state = State::FullyIni,
        }
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

    pub fn get_attr_signatrue(&self) -> Option<BytesRef> {
        match &self.kind {
            ClassKind::Instance(cls) => {
                let idx = util::attributes::get_signature(&cls.class_file.attrs);
                if idx != 0 {
                    let cp = &cls.class_file.cp;
                    get_cp_utf8(cp, idx as usize)
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
                let down_type = down_type.read().unwrap();
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
    pub fn get_static_method(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        self.get_class_method_inner(id, true)
    }

    pub fn get_class_method(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        self.get_class_method_inner(id, true)
    }

    pub fn get_this_class_method(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        self.get_class_method_inner(id, false)
    }

    pub fn get_virtual_method(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        self.get_virtual_method_inner(id)
    }

    pub fn get_interface_method(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        self.get_interface_method_inner(id)
    }

    pub fn get_field_id(&self, name: &[u8], desc: &[u8], is_static: bool) -> FieldIdRef {
        let field_id = util::new_field_id(self.name.as_slice(), name, desc);
        //        error!("get_field_id = {}", String::from_utf8_lossy(field_id.as_slice()));

        if is_static {
            match &self.kind {
                ClassKind::Instance(cls_obj) => match cls_obj.static_fields.get(&field_id) {
                    Some(fid) => return fid.clone(),
                    None => (),
                },
                _ => unreachable!(),
            }
        } else {
            match &self.kind {
                ClassKind::Instance(cls_obj) => match cls_obj.inst_fields.get(&field_id) {
                    Some(fid) => return fid.clone(),
                    None => (),
                },
                _ => unreachable!(),
            }
        }

        let super_class = self.super_class.clone();
        super_class
            .unwrap()
            .read()
            .unwrap()
            .get_field_id(name, desc, is_static)
    }

    //todo: receiver change to ref
    pub fn put_field_value(&self, receiver: Oop, fir: FieldIdRef, v: Oop) {
        match receiver {
            Oop::Ref(rf) => {
                let mut rf = rf.write().unwrap();
                match &mut rf.v {
                    oop::RefKind::Inst(inst) => inst.field_values[fir.offset] = v,
                    oop::RefKind::Mirror(mirror) => mirror.field_values[fir.offset] = v,
                    t => unreachable!("t = {:?}", t),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn get_field_value(&self, receiver: &Oop, fid: FieldIdRef) -> Oop {
        match receiver {
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::Inst(inst) => inst.field_values[fid.offset].clone(),
                    oop::RefKind::Mirror(mirror) => match mirror.field_values.get(fid.offset) {
                        Some(v) => v.clone(),
                        _ => unreachable!("mirror = {:?}", mirror),
                    },
                    t => unreachable!("t = {:?}", t),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn get_field_value2(&self, receiver: &Oop, offset: usize) -> Oop {
        match receiver {
            Oop::Ref(rf) => {
                let rf = rf.read().unwrap();
                match &rf.v {
                    oop::RefKind::Inst(inst) => inst.field_values[offset].clone(),
                    oop::RefKind::Mirror(mirror) => match mirror.field_values.get(offset) {
                        Some(v) => v.clone(),
                        _ => unreachable!("mirror = {:?}", mirror),
                    },
                    t => unreachable!("t = {:?}", t),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn put_static_field_value(&mut self, field_id: FieldIdRef, v: Oop) {
        match &mut self.kind {
            ClassKind::Instance(cls_obj) => {
                let id = field_id.field.get_id();
                if cls_obj.static_fields.contains_key(&id) {
                    cls_obj.static_field_values[field_id.offset] = v;
                } else {
                    let super_class = self.super_class.clone();
                    super_class
                        .unwrap()
                        .write()
                        .unwrap()
                        .put_static_field_value(field_id, v);
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn get_static_field_value(&self, field_id: FieldIdRef) -> Oop {
        match &self.kind {
            ClassKind::Instance(cls_obj) => {
                let id = field_id.field.get_id();
                if cls_obj.static_fields.contains_key(&id) {
                    cls_obj.static_field_values[field_id.offset].clone()
                } else {
                    let super_class = self.super_class.clone();
                    super_class
                        .unwrap()
                        .read()
                        .unwrap()
                        .get_static_field_value(field_id)
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn check_interface(&self, intf: ClassRef) -> bool {
        match &self.kind {
            ClassKind::Instance(inst) => {
                for (_, e) in &inst.interfaces {
                    if Arc::ptr_eq(e, &intf) {
                        return true;
                    }

                    let e = e.read().unwrap();
                    if e.check_interface(intf.clone()) {
                        return true;
                    }
                }
            }
            _ => unreachable!(),
        }

        match &self.super_class {
            Some(super_cls) => {
                let super_cls = super_cls.read().unwrap();
                super_cls.check_interface(intf)
            }
            None => false,
        }
    }

    pub fn hack_as_native(&mut self, id: BytesRef) {
        let cls_name = self.name.clone();
        match &mut self.kind {
            ClassKind::Instance(cls) => {
                {
                    let m = cls.all_methods.get(&id).unwrap();
                    let mut method = m.method.clone();
                    method.acc_flags |= ACC_NATIVE;
                    let m = Arc::new(method::MethodId {
                        offset: m.offset,
                        method,
                    });
                    cls.all_methods.insert(id.clone(), m);
                }

                let m = cls.all_methods.get(&id).unwrap();
                info!(
                    "hack_as_native: {}:{}, native={}",
                    String::from_utf8_lossy(cls_name.as_slice()),
                    String::from_utf8_lossy(id.as_slice()),
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
        let cp = &class_file.cp;
        let name = constant_pool::get_class_name(cp, class_file.this_class as usize).unwrap();
        let acc_flags = class_file.acc_flags;
        let class_obj = ClassObject {
            class_file,
            n_inst_fields: 0,
            all_methods: HashMap::new(),
            v_table: HashMap::new(),
            static_fields: HashMap::new(),
            inst_fields: HashMap::new(),
            static_field_values: vec![],
            interfaces: HashMap::new(),
            mirror: None,
            signature: None,
            source_file: None,
            enclosing_method: None,
            inner_classes: None,
        };

        Self {
            name,
            state: State::Allocated,
            acc_flags,
            super_class: None,
            class_loader,
            monitor: Mutex::new(0),

            kind: ClassKind::Instance(class_obj),
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
            name,
            state: State::Allocated,
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            monitor: Mutex::new(0),
            kind: ClassKind::ObjectArray(ary_cls_obj),
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
            name: Arc::new(name),
            state: State::Allocated,
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            monitor: Mutex::new(0),
            kind: ClassKind::TypeArray(ary_cls_obj),
        }
    }

    pub fn new_wrapped_ary(class_loader: ClassLoader, down_type: ClassRef) -> Self {
        let (name, cls_kind) = {
            let cls = down_type.read().unwrap();
            assert!(cls.is_array());
            (cls.name.clone(), cls.get_class_kind_type())
        };

        //build name
        let mut name2 = Vec::with_capacity(1 + name.len());
        name2.push(b'[');
        name2.extend_from_slice(&name);

        let kind = match cls_kind {
            ClassKindType::Instance => unreachable!(),
            ClassKindType::TypAry => ClassKind::TypeArray(ArrayClassObject {
                value_type: ValueType::ARRAY,
                down_type: Some(down_type.clone()),
                component: None,
                mirror: None,
            }),
            ClassKindType::ObjectAry => {
                let component = {
                    let cls = down_type.read().unwrap();
                    match &cls.kind {
                        ClassKind::ObjectArray(ary_cls) => ary_cls.component.clone(),
                        _ => unreachable!(),
                    }
                };
                ClassKind::ObjectArray(ArrayClassObject {
                    value_type: ValueType::ARRAY,
                    down_type: Some(down_type.clone()),
                    component,
                    mirror: None,
                })
            }
        };

        Self {
            name: Arc::new(name2),
            state: State::Allocated,
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            monitor: Mutex::new(0),
            kind,
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
            let name = constant_pool::get_class_name(cp, class_file.super_class as usize).unwrap();
            let super_class = runtime::require_class(class_loader, name).unwrap();

            {
                let c = super_class.read().unwrap();
                assert!(c.is_instance());
                assert!(!c.is_final(), "should not final");
            }

            Some(super_class)
        }
    }

    fn link_fields(&mut self, self_ref: ClassRef, name: BytesRef, n_super_inst: usize) {
        let cls_file = self.class_file.clone();
        let cp = &cls_file.cp;

        let mut n_static = 0;
        let mut n_inst = n_super_inst;
        let class_name = name.clone();
        let class_name = class_name.as_slice();
        cls_file.fields.iter().for_each(|it| {
            let field = field::Field::new(cp, it, class_name, self_ref.clone());
            let id = field.get_id();
            if field.is_static() {
                let fid = field::FieldId {
                    offset: n_static,
                    field,
                };
                self.static_fields.insert(id, Arc::new(fid));
                n_static += 1;
            } else {
                let fid = field::FieldId {
                    offset: n_inst,
                    field,
                };
                self.inst_fields.insert(id, Arc::new(fid));
                n_inst += 1;
            }
        });

        self.n_inst_fields = n_inst;

        //todo: avoid this
        //  sort static_fields by offset, then static_field_values.push
        for _ in 0..n_static {
            self.static_field_values.push(oop_consts::get_null());
        }
    }

    fn link_interfaces(&mut self) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file
            .interfaces
            .iter()
            .for_each(|it| match runtime::require_class2(*it, cp) {
                Some(class) => {
                    let name = class.read().unwrap().name.clone();
                    self.interfaces.insert(name, class);
                }
                None => {
                    let name = constant_pool::get_class_name(cp, *it as usize);
                    error!("link interface failed {:?}", name);
                }
            });
    }

    fn link_methods(&mut self, this_ref: ClassRef) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file.methods.iter().enumerate().for_each(|(i, it)| {
            let method = method::Method::new(cp, it, this_ref.clone(), class_file.clone(), i);
            let id = method.get_id();
            let method_id = Arc::new(method::MethodId { offset: i, method });

            self.all_methods.insert(id.clone(), method_id.clone());

            if !method_id.method.is_static() {
                self.v_table.insert(id, method_id);
            }
        });
    }

    fn link_attributes(&mut self) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file.attrs.iter().for_each(|a| match a {
            AttributeType::Signature { signature_index } => {
                if let Some(s) = get_cp_utf8(cp, *signature_index as usize) {
                    self.signature = Some(s);
                }
            }
            AttributeType::SourceFile { source_file_index } => {
                if let Some(s) = get_cp_utf8(cp, *source_file_index as usize) {
                    self.source_file = Some(s);
                }
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
        id: BytesRef,
        with_super: bool,
    ) -> Result<MethodIdRef, ()> {
        match &self.kind {
            ClassKind::Instance(cls_obj) => match cls_obj.all_methods.get(&id) {
                Some(m) => return Ok(m.clone()),
                None => (),
            },

            ClassKind::ObjectArray(ary) => {
                //use java/lang/Object, methods
            }
            _ => unreachable!(),
        }

        if with_super {
            match self.super_class.as_ref() {
                Some(super_class) => {
                    return super_class
                        .read()
                        .unwrap()
                        .get_class_method_inner(id, with_super);
                }
                None => return Err(()),
            }
        }

        Err(())
    }

    fn get_virtual_method_inner(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        match &self.kind {
            ClassKind::Instance(cls_obj) => match cls_obj.v_table.get(&id) {
                Some(m) => return Ok(m.clone()),
                None => (),
            },
            _ => unreachable!(),
        }

        match self.super_class.as_ref() {
            Some(super_class) => {
                return super_class.read().unwrap().get_virtual_method_inner(id);
            }
            None => return Err(()),
        }
    }

    pub fn get_interface_method_inner(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        match &self.kind {
            ClassKind::Instance(cls_obj) => match cls_obj.v_table.get(&id) {
                Some(m) => return Ok(m.clone()),
                None => {
                    for (_, itf) in cls_obj.interfaces.iter() {
                        let cls = itf.read().unwrap();
                        match cls.get_interface_method(id.clone()) {
                            Ok(m) => return Ok(m.clone()),
                            _ => (),
                        }
                    }
                }
            },
            _ => unreachable!(),
        }

        match self.super_class.as_ref() {
            Some(super_class) => {
                return super_class.read().unwrap().get_interface_method_inner(id);
            }
            None => return Err(()),
        }
    }
}
