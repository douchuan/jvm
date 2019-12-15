use crate::classfile::{access_flags::*, attr_info::AttrType, constant_pool, consts, types::*};
use crate::oop::{ClassFileRef, ClassRef, Field, FieldId, Method, MethodId, Oop, ValueType};
use crate::runtime::{self, ClassLoader};
use crate::util;

use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
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
    pub name: String,

    pub typ: Type,

    pub state: State,

    pub acc_flags: U2,

    //superclass, or None if this is java.lang.Object
    pub super_class: Option<ClassRef>,

    //defining class loader, or None for the "bootstrap" system loader
    pub class_loader: Option<ClassLoader>,

    pub class_file: ClassFileRef,

    pub signature: Option<String>,

    //valid when dimension == 1
    elm_type: Option<ValueType>,
    //valid when dimension > 1
    down_type: Option<ClassRef>,

    n_static_fields: usize,
    n_inst_fields: usize,

    all_methods: HashMap<String, MethodId>,
    v_table: HashMap<String, MethodId>,

    static_fields: HashMap<String, FieldId>,
    inst_fields: HashMap<String, FieldId>,

    static_filed_values: Vec<Oop>,

    interfaces: HashMap<String, ClassRef>,

    pub source_file: Option<String>,
}

//open api
impl ClassObject {
    pub fn get_class_state(&self) -> State {
        self.state
    }

    pub fn set_class_state(&mut self, s: State) {
        self.state = s;
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
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

    pub fn init_class(&mut self) {
        if self.state == State::Linked {
            self.state = State::BeingIni;

            if let Some(super_class) = self.super_class.as_ref() {
                super_class.lock().unwrap().init_class();
            }

            self.init_static_fields();

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
        let id = vec![desc, name].join(":");
        self.all_methods.get(&id)
    }
}

//open api new
impl ClassObject {
    pub fn new_class(class_file: ClassFileRef, class_loader: Option<ClassLoader>) -> Self {
        let cp = &class_file.cp;
        let name = constant_pool::get_class_name2(class_file.this_class, cp).unwrap();

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
            if self.name != consts::JAVA_LANG_OBJECT {
                self.set_class_state(State::IniErr);
                assert!(false, format!("should be {}", consts::JAVA_LANG_OBJECT));
            }
            self.super_class = None;
        } else {
            let name = constant_pool::get_class_name(class_file.super_class, cp).unwrap();
            let name = std::str::from_utf8(name).unwrap();
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
                self.static_fields.insert(id, fid);
                n_static += 1;
            } else {
                let fid = FieldId {
                    offset: n_inst,
                    field,
                };
                self.inst_fields.insert(id, fid);
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
                    let name = constant_pool::get_class_name2(*it, cp);
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
                    if let Some(s) = constant_pool::get_utf8(*signature_index, cp) {
                        self.signature = Some(String::from_utf8_lossy(s).to_string());
                    }
                }
                AttrType::SourceFile {
                    length,
                    source_file_index,
                } => {
                    if let Some(s) = constant_pool::get_utf8(*source_file_index, cp) {
                        self.source_file = Some(String::from_utf8_lossy(s).to_string());
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
