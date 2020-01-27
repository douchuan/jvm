use crate::classfile::{access_flags::*, attr_info::AttrType, constant_pool, consts, types::*};
use crate::oop::{
    consts as oop_consts, field, method, ClassRef, ClassFileRef, FieldIdRef, MethodIdRef, Oop,
    OopDesc, ValueType,
};
use crate::runtime::{self, require_class2, ClassLoader, JavaThread, JavaCall, Stack};
use crate::util::{self, PATH_DELIMITER};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
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

#[derive(Debug)]
pub enum ClassKind {
    Instance(ClassObject),
    ObjectArray(ArrayKlassObject, ClassFileRef),
    TypeArray(ArrayKlassObject)
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

    all_methods: HashMap<BytesRef, MethodIdRef>,
    v_table: HashMap<BytesRef, MethodIdRef>,

    static_fields: HashMap<BytesRef, FieldIdRef>,
    inst_fields: HashMap<BytesRef, FieldIdRef>,

    static_filed_values: Vec<Arc<OopDesc>>,

    interfaces: HashMap<BytesRef, ClassRef>,

    pub signature: Option<BytesRef>,
    pub source_file: Option<BytesRef>,
}

#[derive(Debug)]
pub struct ArrayKlassObject {
   //valid when dimension == 1
   elm_type: Option<ValueType>,
   //valid when dimension > 1
   down_type: Option<ClassRef>
}

pub fn init_class_fully(thread: &mut JavaThread, class: ClassRef) {
    let need = {
        class.lock().unwrap().state == State::BeingIni
    };

    if need {
        let mir = {
            let class = class.lock().unwrap();
//            trace!("init_class_fully name={}, state={:?}",
//                   String::from_utf8_lossy(class.name.as_slice()),
//                   class.state);
            class.get_this_class_method(b"()V", b"<clinit>")
        };

        {
            let mut class = class.lock().unwrap();
            class.state = State::FullyIni;
        }

        {
            match mir {
                Ok(mir) => {
                    trace!("call <clinit>");
                    let mut stack = Stack::new(0);
                    let jc = JavaCall::new(thread, &mut stack, mir);
                    jc.unwrap().invoke(thread, &mut stack);
                }
                _ => {
                    let class = class.lock().unwrap();
                    trace!("init_class_fully name={}, no <clinit> ",
                           String::from_utf8_lossy(class.name.as_slice()));
                },
            }
        }
    }
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
        //todo: java mirror
        //        java::lang::Class::createMirror(this, _javaLoader);

        match &mut self.kind {
            ClassKind::Instance(class_obj) => {
                self.super_class = class_obj.link_super_class(self.name.clone(), self.class_loader.clone());
                class_obj.link_fields(self_ref.clone(), self.name.clone());
                class_obj.link_interfaces();
                class_obj.link_methods(self_ref);
                class_obj.link_attributes();
            }

            ClassKind::ObjectArray(ary_class_obj, _) => {
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
                        super_class.lock().unwrap().init_class(thread);
                    }

                    class_obj.init_static_fields();
                }
            }

            ClassKind::ObjectArray(_, _) => (), //skip for Array
            ClassKind::TypeArray(_) => ()
        }
    }

    pub fn is_array(&self) -> bool {
        match &self.kind {
            ClassKind::Instance(_) => false,
            _ => true,
        }
    }
}

impl ArrayKlassObject {
    pub fn get_dimension(&self) -> Option<usize> {
        match self.down_type.as_ref() {
            Some(down_type) => {
                let down_type = down_type.lock().unwrap();
                let n = match &down_type.kind {
                    ClassKind::Instance(_) => unreachable!(),
                    ClassKind::ObjectArray(ary_cls_obj, _) => ary_cls_obj.get_dimension(),
                    ClassKind::TypeArray(ary_cls_obj) => ary_cls_obj.get_dimension(),
                };
                Some(1 + n.unwrap())
            },
            None => Some(1)
        }
    }
}

//open api
impl Class {
    //todo: confirm static method
    pub fn get_static_method(&self, desc: &[u8], name: &[u8]) -> Result<MethodIdRef, ()> {
        self.get_this_class_method(desc, name)
    }

    pub fn get_this_class_method(&self, desc: &[u8], name: &[u8]) -> Result<MethodIdRef, ()> {
        let id = Arc::new(vec![desc, name].join(PATH_DELIMITER));
        self.get_this_class_method_inner(id)
    }

    pub fn get_virtual_method(&self, desc: &[u8], name: &[u8]) -> Result<MethodIdRef, ()> {
        let id = Arc::new(vec![desc, name].join(PATH_DELIMITER));
        self.get_virtual_method_inner(id)
    }

    pub fn get_field_id(&self, id: BytesRef, is_static: bool) -> FieldIdRef {
        if is_static {
            match &self.kind {
                ClassKind::Instance(cls_obj) => {
                    match cls_obj.static_fields.get(&id) {
                        Some(fid) => return fid.clone(),
                        None => (),
                    }
                }
                _ => unreachable!()
            }

        } else {
            match &self.kind {
                ClassKind::Instance(cls_obj) => {
                    match cls_obj.inst_fields.get(&id) {
                        Some(fid) => return fid.clone(),
                        None => (),
                    }
                }
                _ => unreachable!()
            }
        }

        let super_class = self.super_class.clone();
        super_class
            .unwrap()
            .lock()
            .unwrap()
            .get_field_id(id, is_static)
    }

    pub fn put_field_value(&self, mut receiver: Arc<OopDesc>, fid: FieldIdRef, v: Arc<OopDesc>) {
        let rff = Arc::get_mut(&mut receiver).unwrap();
        match &mut rff.v {
            Oop::Inst(inst) => inst.filed_values[fid.offset] = v,
            _ => unreachable!(),
        }
    }

    pub fn get_field_value(&self, receiver: Arc<OopDesc>, fid: FieldIdRef) -> Arc<OopDesc> {
        match &receiver.v {
            Oop::Inst(inst) => inst.filed_values[fid.offset].clone(),
            _ => unreachable!(),
        }
    }

    pub fn put_static_field_value(&mut self, field_id: FieldIdRef, v: Arc<OopDesc>) {
        match &mut self.kind {
            ClassKind::Instance(cls_obj) => {
                let id = field_id.field.get_id();
                if cls_obj.static_fields.contains_key(&id) {
                    cls_obj.static_filed_values[field_id.offset] = v;
                } else {
                    let super_class = self.super_class.clone();
                    super_class
                        .unwrap()
                        .lock()
                        .unwrap()
                        .put_static_field_value(field_id, v);
                }
            }
            _ => unreachable!()
        }

    }

    pub fn get_static_field_value(&self, field_id: FieldIdRef) -> Arc<OopDesc> {
        match &self.kind {
            ClassKind::Instance(cls_obj) => {
                let id = field_id.field.get_id();
                if cls_obj.static_fields.contains_key(&id) {
                    cls_obj.static_filed_values[field_id.offset].clone()
                } else {
                    let super_class = self.super_class.clone();
                    super_class
                        .unwrap()
                        .lock()
                        .unwrap()
                        .get_static_field_value(field_id)
                }
            }
            _ => unreachable!()
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
            static_filed_values: vec![],
            interfaces: HashMap::new(),
            signature: None,
            source_file: None,
        };

        Self {
            name,
            state: State::Allocated,
            acc_flags,
            super_class: None,
            class_loader,
            monitor: Mutex::new(0),

            kind: ClassKind::Instance(class_obj)
        }
    }

    pub fn new_object_ary(class_loader: ClassLoader, elm: ClassRef, elm_name: &[u8]) -> Self {
        let name = Arc::new(Vec::from(elm_name));
        let class_file = {
            let class = elm.lock().unwrap();
            match &class.kind {
                ClassKind::Instance(cls_obj) => cls_obj.class_file.clone(),
                _ => unreachable!()
            }
        };

        let array_klass_obj = ArrayKlassObject {
            elm_type: Some(ValueType::OBJECT),
            down_type: None,
        };

        Self {
            name,
            state: State::Allocated,
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            monitor: Mutex::new(0),
            kind: ClassKind::ObjectArray(array_klass_obj, class_file)
        }
    }

    pub fn new_prime_ary(class_loader: ClassLoader, elm: ValueType) -> Self {
        unimplemented!()
        /*
        let array_klass_obj = ArrayKlassObject {
            elm_type: Some(ValueType::OBJECT),
            down_type: None,
        };

        Self {
            name,
            state: State::Allocated,
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            signature: None,
            source_file: None,
            monitor: Mutex::new(0),
            kind: ClassKind::TypeArray(array_klass_obj)
        }
        */
    }

    pub fn new_wrapped_ary(class_loader: ClassLoader, down_type: ClassRef) -> Self {
        unimplemented!()
    }
}

//inner api for link
impl ClassObject {
    fn link_super_class(&mut self, name: BytesRef, class_loader: Option<ClassLoader>) -> Option<ClassRef> {
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
            util::sync_call_ctx(&super_class, |c| {
                assert!(!c.is_final(), "should not final");
            });

            Some(super_class)
        }
    }

    fn link_fields(&mut self, self_ref: ClassRef, name: BytesRef) {
        let cls_file = self.class_file.clone();
        let cp = &cls_file.cp;

        let mut n_static = 0;
        let mut n_inst = 0;
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

        self.static_filed_values.reserve(n_static);
        //todo: avoid this?
        for _ in 0..self.static_filed_values.capacity() {
            self.static_filed_values.push(oop_consts::get_null());
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
                    let name = class.lock().unwrap().name.clone();
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
            let method = method::Method::new(cp, it, this_ref.clone());
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

        class_file.attrs.iter().for_each(|a| {
            match a {
                AttrType::Signature { signature_index } => {
                    if let Some(s) = constant_pool::get_utf8(cp, *signature_index as usize) {
                        self.signature = Some(s);
                    }
                }
                AttrType::SourceFile { source_file_index } => {
                    if let Some(s) = constant_pool::get_utf8(cp, *source_file_index as usize) {
                        self.source_file = Some(s);
                    }
                }
                //todo: ATTRIBUTE_InnerClasses, ATTRIBUTE_EnclosingMethod, ATTRIBUTE_BootstrapMethods
                _ => (),
            }
        });
    }

    fn init_static_fields(&mut self) {
        let class_file = &self.class_file;
        let cp = &class_file.cp;
        let values = &mut self.static_filed_values;
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
    pub fn get_this_class_method_inner(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        match &self.kind {
            ClassKind::Instance(cls_obj) => {
                match cls_obj.all_methods.get(&id) {
                    Some(m) => return Ok(m.clone()),
                    None => (),
                }
            }
            _ => unreachable!()
        }

        match self.super_class.as_ref() {
            Some(super_class) => {
                return super_class.lock().unwrap().get_this_class_method_inner(id);
            }
            None => return Err(())
        }
    }

    pub fn get_virtual_method_inner(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        match &self.kind {
            ClassKind::Instance(cls_obj) => {
                match cls_obj.v_table.get(&id) {
                    Some(m) => return Ok(m.clone()),
                    None => (),
                }
            }
            _ => unreachable!()
        }

        match self.super_class.as_ref() {
            Some(super_class) => {
                return super_class.lock().unwrap().get_virtual_method_inner(id);
            }
            None => return Err(())
        }
    }
}