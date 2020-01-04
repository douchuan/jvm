use crate::classfile::{access_flags::*, attr_info::AttrType, constant_pool, consts, types::*};
use crate::oop::{field, consts as oop_consts, ClassFileRef, ClassRef, Field, FieldId, FieldIdRef, Method, MethodId, OopDesc, ValueType};
use crate::runtime::{self, ClassLoader, JavaThread, require_class2};
use crate::util::{self, PATH_DELIMITER};
use std::collections::HashMap;
use std::sync::Arc;

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

#[derive(Debug, Clone)]
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

    n_static_fields: usize,
    n_inst_fields: usize,

    all_methods: HashMap<BytesRef, MethodId>,
    v_table: HashMap<BytesRef, MethodId>,

    static_fields: HashMap<BytesRef, Arc<FieldId>>,
    inst_fields: HashMap<BytesRef, Arc<FieldId>>,

    static_filed_values: Vec<Arc<OopDesc>>,

    interfaces: HashMap<BytesRef, ClassRef>,

    pub source_file: Option<BytesRef>,
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

    pub fn link_class(&mut self) {
        //todo: java mirror
        //        java::lang::Class::createMirror(this, _javaLoader);

        self.link_super_class();
        self.link_fields();
        self.link_interfaces();
        self.link_methods();
        self.link_constant_pool();
        self.link_attributes();

        self.set_class_state(State::Linked);
    }

    pub fn init_class(&mut self, thread: Arc<JavaThread>) {
        if self.state == State::Linked {
            self.state = State::BeingIni;

            if let Some(super_class) = self.super_class.as_ref() {
                super_class.lock().unwrap().init_class(thread);
            }

            self.init_static_fields();

            //todo: JavaCall "<clinit>" "()V"

            self.state = State::FullyIni;
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
        match self.typ {
            Type::PrimeArray => true,
            _ => false,
        }
    }

    pub fn is_object_array(&self) -> bool {
        match self.typ {
            Type::ObjectArray => true,
            _ => false,
        }
    }

    pub fn get_static_method(&self, desc: &str, name: &str) -> Option<&MethodId> {
        self.get_static_method2(desc.as_bytes(), name.as_bytes())
    }

    pub fn get_static_method2(&self, desc: &[u8], name: &[u8]) -> Option<&MethodId> {
        let id = vec![desc, name].join(PATH_DELIMITER);
        self.all_methods.get(&id)
    }

    pub fn get_field_id(&self, id: BytesRef, is_static: bool) -> Arc<FieldId> {
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
        super_class.unwrap().lock().unwrap().get_field_id(id, is_static)
    }

    pub fn put_static_field_value(&mut self, field_id: FieldIdRef, v: Arc<OopDesc>) {
        let id = field_id.field.get_id();
        if self.static_fields.contains_key(&id) {
            self.static_filed_values[field_id.offset] = v;
        } else {
            let super_class = self.super_class.clone();
            super_class.unwrap().lock().unwrap().put_static_field_value(field_id, v);
        }
    }

    pub fn get_static_field_value(&self, field_id: FieldIdRef) -> Arc<OopDesc> {
        let id = field_id.field.get_id();
        if self.static_fields.contains_key(&id) {
            self.static_filed_values[field_id.offset].clone()
        } else {
            let super_class = self.super_class.clone();
            super_class.unwrap().lock().unwrap().get_static_field_value(field_id)
        }
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
            n_static_fields: 0,
            n_inst_fields: 0,
            all_methods: HashMap::new(),
            v_table: HashMap::new(),
            static_fields: HashMap::new(),
            inst_fields: HashMap::new(),
            static_filed_values: vec![],
            interfaces: HashMap::new(),
            source_file: None,
        }
    }

    pub fn new_object_ary(class_loader: ClassLoader, elm: ClassRef) -> Self {
        unimplemented!()
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
        let class_file = self.class_file.clone();
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

    fn link_fields(&mut self) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        let mut n_static = 0;
        let mut n_inst = 0;
        class_file.fields.iter().for_each(|it| {
            let field = Field::new(cp, it, self);
            let id = field.get_id();
            if field.is_static() {
                let fid = FieldId {
                    offset: n_static,
                    field,
                };
                self.static_fields.insert(id, Arc::new(fid));
                n_static += 1;
            } else {
                let fid = FieldId {
                    offset: n_inst,
                    field,
                };
                self.inst_fields.insert(id, Arc::new(fid));
                n_inst += 1;
            }
        });

        self.n_static_fields = n_static;
        self.n_inst_fields = n_inst;

        self.static_filed_values.reserve(n_static);
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

    fn link_methods(&mut self) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file.methods.iter().enumerate().for_each(|(i, it)| {
            let method = Method::new(cp, it, self);
            let id = method.get_id();
            let method_id = MethodId { offset: i, method };

            self.all_methods.insert(id.clone(), method_id.clone());

            if !method_id.method.is_static() {
                self.v_table.insert(id, method_id);
            }
        });
    }

    fn link_constant_pool(&mut self) {
        unimplemented!()
    }

    fn link_attributes(&mut self) {
        let class_file = self.class_file.clone();
        let cp = &class_file.cp;

        class_file.attrs.iter().for_each(|a| {
            match a {
                AttrType::Signature {
                    length,
                    signature_index,
                } => {
                    if let Some(s) = constant_pool::get_utf8(cp, *signature_index as usize) {
                        self.signature = Some(s);
                    }
                }
                AttrType::SourceFile {
                    length,
                    source_file_index,
                } => {
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
        let class_file = self.class_file.clone();
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
