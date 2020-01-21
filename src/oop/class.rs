use crate::classfile::{access_flags::*, attr_info::AttrType, constant_pool, consts, types::*};
use crate::oop::{
    consts as oop_consts, field, method, ClassFileRef, ClassRef, FieldIdRef, MethodIdRef, Oop,
    OopDesc, ValueType,
};
use crate::runtime::{self, require_class2, ClassLoader, JavaThread, JavaCall, Stack};
use crate::util::{self, PATH_DELIMITER};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type {
    InstanceClass,
    ObjectArray,
    PrimeArray,
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

pub fn init_class_fully(thread: &mut JavaThread, class: ClassRef) {
    let need = {
        class.lock().unwrap().state != State::FullyIni
    };

    if need {
        let mir = {
            let class = class.lock().unwrap();
            class.get_this_class_method(b"()V", b"<clinit>")
        };

        {
            match mir {
                Ok(mir) => {
                    let mut stack = Stack::new(0);
                    let jc = JavaCall::new(thread, &mut stack, mir);
                    jc.unwrap().invoke(thread, &mut stack);
                }
                _ => (),
            }
        }

        {
            let mut class = class.lock().unwrap();
            class.state = State::FullyIni;
        }
    }

}

#[derive(Debug)]
pub struct ClassObject {
    pub name: BytesRef,

    pub typ: Type,

    pub state: State,

    pub acc_flags: U2,

    //superclass, or None if this is java.lang.Object
    pub super_class: Option<ClassRef>,

    //defining class loader, or None for the "bootstrap" system loader
    pub class_loader: Option<ClassLoader>,

    pub class_file: ClassFileRef,

    pub signature: Option<BytesRef>,

    //valid when dimension == 1
    elm_type: Option<ValueType>,
    //valid when dimension > 1
    down_type: Option<ClassRef>,

    pub n_inst_fields: usize,

    all_methods: HashMap<BytesRef, MethodIdRef>,
    v_table: HashMap<BytesRef, MethodIdRef>,

    static_fields: HashMap<BytesRef, FieldIdRef>,
    inst_fields: HashMap<BytesRef, FieldIdRef>,

    static_filed_values: Vec<Arc<OopDesc>>,

    interfaces: HashMap<BytesRef, ClassRef>,

    pub source_file: Option<BytesRef>,

    monitor: Mutex<usize>,
}

//open api
impl ClassObject {
    pub fn get_class_state(&self) -> State {
        self.state
    }

    pub fn set_class_state(&mut self, s: State) {
        self.state = s;
    }

    pub fn get_name(&self) -> BytesRef {
        self.name.clone()
    }

    pub fn get_class_type(&self) -> Type {
        self.typ
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

    pub fn link_class(&mut self, self_ref: ClassRef) {
        //todo: java mirror
        //        java::lang::Class::createMirror(this, _javaLoader);

        match self.typ {
            Type::InstanceClass => {
                self.link_super_class();
                self.link_fields(self_ref.clone());
                self.link_interfaces();
                self.link_methods(self_ref);
                self.link_constant_pool();
                self.link_attributes();
            }

            Type::ObjectArray | Type::PrimeArray => {
                let super_class = runtime::require_class3(None, consts::J_OBJECT).unwrap();
                self.super_class = Some(super_class);
            }
        }

        self.set_class_state(State::Linked);
    }

    pub fn init_class(&mut self, thread: &mut JavaThread) {
        match self.typ {
            Type::InstanceClass => {
                if self.state == State::Linked {
                    self.state = State::BeingIni;

                    if let Some(super_class) = self.super_class.as_ref() {
                        super_class.lock().unwrap().init_class(thread);
                    }

                    self.init_static_fields();
                }
            }

            Type::ObjectArray | Type::PrimeArray => (), //skip for Array
        }
    }

    pub fn is_array(&self) -> bool {
        match self.typ {
            Type::PrimeArray | Type::ObjectArray => true,
            _ => false,
        }
    }

    pub fn get_dimension(&self) -> Option<usize> {
        match self.down_type.as_ref() {
            Some(down_type) => Some(1 + down_type.lock().unwrap().get_dimension().unwrap()),
            None => {
                if self.is_array() {
                    Some(1)
                } else {
                    None
                }
            }
        }
    }

    pub fn is_prime_array(&self) -> bool {
        self.typ == Type::PrimeArray
    }

    pub fn is_object_array(&self) -> bool {
        self.typ == Type::ObjectArray
    }

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
            match self.static_fields.get(&id) {
                Some(fid) => return fid.clone(),
                None => (),
            }
        } else {
            match self.inst_fields.get(&id) {
                Some(fid) => return fid.clone(),
                None => (),
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
        let id = field_id.field.get_id();
        if self.static_fields.contains_key(&id) {
            self.static_filed_values[field_id.offset] = v;
        } else {
            let super_class = self.super_class.clone();
            super_class
                .unwrap()
                .lock()
                .unwrap()
                .put_static_field_value(field_id, v);
        }
    }

    pub fn get_static_field_value(&self, field_id: FieldIdRef) -> Arc<OopDesc> {
        let id = field_id.field.get_id();
        if self.static_fields.contains_key(&id) {
            self.static_filed_values[field_id.offset].clone()
        } else {
            let super_class = self.super_class.clone();
            super_class
                .unwrap()
                .lock()
                .unwrap()
                .get_static_field_value(field_id)
        }
    }

    pub fn monitor_enter(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v += 1;
    }

    pub fn monitor_exit(&mut self) {
        let mut v = self.monitor.lock().unwrap();
        *v -= 1;
    }
}

//open api new
impl ClassObject {
    pub fn new_class(class_file: ClassFileRef, class_loader: Option<ClassLoader>) -> Self {
        let cp = &class_file.cp;
        let name = constant_pool::get_class_name(cp, class_file.this_class as usize).unwrap();

        Self {
            name,
            typ: Type::InstanceClass,
            state: State::Allocated,
            acc_flags: class_file.acc_flags,
            super_class: None,
            class_loader,
            class_file,
            signature: None,
            elm_type: None,
            down_type: None,
            n_inst_fields: 0,
            all_methods: HashMap::new(),
            v_table: HashMap::new(),
            static_fields: HashMap::new(),
            inst_fields: HashMap::new(),
            static_filed_values: vec![],
            interfaces: HashMap::new(),
            source_file: None,
            monitor: Mutex::new(0),
        }
    }

    pub fn new_object_ary(class_loader: ClassLoader, elm: ClassRef, elm_name: &[u8]) -> Self {
        let name = Arc::new(Vec::from(elm_name));
        let class_file = {
            let class = elm.lock().unwrap();
            class.class_file.clone()
        };
        Self {
            name,
            typ: Type::ObjectArray,
            state: State::Allocated,
            acc_flags: 0, //todo: should be 0?
            super_class: None,
            class_loader: Some(class_loader),
            class_file,
            signature: None,
            elm_type: Some(ValueType::OBJECT),
            down_type: None,
            n_inst_fields: 0,
            all_methods: HashMap::new(),
            v_table: HashMap::new(),
            static_fields: HashMap::new(),
            inst_fields: HashMap::new(),
            static_filed_values: vec![],
            interfaces: HashMap::new(),
            source_file: None,
            monitor: Mutex::new(0),
        }
    }

    pub fn new_prime_ary(class_loader: ClassLoader, elm: ValueType) -> Self {
        unimplemented!()
    }

    pub fn new_wrapped_ary(class_loader: ClassLoader, down_type: ClassRef) -> Self {
        unimplemented!()
    }
}

//inner api for link
impl ClassObject {
    fn link_super_class(&mut self) {
        let class_file = &self.class_file;
        let cp = &class_file.cp;

        if class_file.super_class == 0 {
            if self.name.as_slice() != consts::J_OBJECT {
                self.set_class_state(State::IniErr);
                assert!(false, format!("should be {:?}", consts::J_OBJECT));
            }
            self.super_class = None;
        } else {
            let name = constant_pool::get_class_name(cp, class_file.super_class as usize).unwrap();
            let super_class = runtime::require_class(self.class_loader, name).unwrap();
            util::sync_call_ctx(&super_class, |c| {
                assert!(!c.is_final(), "should not final");
            });
            self.super_class = Some(super_class);
        }
    }

    fn link_fields(&mut self, self_ref: ClassRef) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        let mut n_static = 0;
        let mut n_inst = 0;
        let class_name = self.name.clone();
        let class_name = class_name.as_slice();
        class_file.fields.iter().for_each(|it| {
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

    fn link_constant_pool(&mut self) {
        //todo: impl
        //        unimplemented!()
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

impl ClassObject {
    pub fn get_this_class_method_inner(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        match self.all_methods.get(&id) {
            Some(m) => return Ok(m.clone()),
            None => (),
        }

        if self.super_class.is_none() {
            return Err(());
        } else {
            let super_class = self.super_class.clone();
            super_class
                .unwrap()
                .lock()
                .unwrap()
                .get_this_class_method_inner(id)
        }
    }

    pub fn get_virtual_method_inner(&self, id: BytesRef) -> Result<MethodIdRef, ()> {
        match self.v_table.get(&id) {
            Some(m) => return Ok(m.clone()),
            None => (),
        }

        if self.super_class.is_none() {
            return Err(());
        } else {
            let super_class = self.super_class.clone();
            super_class.unwrap().lock().unwrap().get_virtual_method_inner(id)
        }
    }
}