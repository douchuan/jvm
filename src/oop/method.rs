use crate::classfile::attr_info::AnnotationEntry;
use crate::classfile::{
    access_flags::*, attr_info::Code, constant_pool, consts, types::*, AttrType, FieldInfo,
    MethodInfo,
};
use crate::oop::{self, ClassRef, ValueType};
use crate::runtime::{self, require_class2, JavaThread};
use crate::util::{self, PATH_DELIMITER};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub type MethodIdRef = Arc<MethodId>;

pub fn get_method_ref(
    thread: &mut JavaThread,
    cp: &ConstantPool,
    idx: usize,
) -> Result<MethodIdRef, ()> {
    let (tag, class_index, name_and_type_index) = constant_pool::get_method_ref(cp, idx);

    //load Method's Class, then init it
    let class = require_class2(class_index, cp).unwrap();

    {
        let mut class = class.lock().unwrap();
        class.init_class(thread);
    }

    let (name, typ) = {
        let (name, typ) = constant_pool::get_name_and_type(cp, name_and_type_index as usize);
        let name = name.unwrap();
        let typ = typ.unwrap();

        (name, typ)
    };

    oop::class::init_class_fully(thread, class.clone());

    let class = class.lock().unwrap();
    let id = util::new_method_id(name.as_slice(), typ.as_slice());
    let mir = if tag == consts::CONSTANT_METHOD_REF_TAG {
        // invokespecial, invokestatic and invokevirtual
        class.get_class_method(id)
    } else {
        // invokeinterface
        class.get_virtual_method(id)
    };

    mir
}

#[derive(Debug, Clone)]
pub struct MethodId {
    pub offset: usize,
    pub method: Method,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub class: ClassRef,
    pub name: BytesRef,
    pub desc: BytesRef,
    id: BytesRef,
    pub acc_flags: U2,

    pub code: Option<Code>,
    //fixme: more readable name
    pub lnt: HashMap<U2, U2>,

    vis_annos: Vec<AnnotationEntry>,
    vis_param_annos: Vec<AnnotationEntry>,
}

impl Method {
    pub fn new(
        cp: &ConstantPool,
        mi: &MethodInfo,
        class: ClassRef,
        vis_annos: Vec<AnnotationEntry>,
        vis_param_annos: Vec<AnnotationEntry>,
    ) -> Self {
        let name = constant_pool::get_utf8(cp, mi.name_index as usize).unwrap();
        let desc = constant_pool::get_utf8(cp, mi.desc_index as usize).unwrap();
        let id = vec![name.as_slice(), desc.as_slice()].join(PATH_DELIMITER);
        let id = new_ref!(id);
        //        info!("id = {}", String::from_utf8_lossy(id.as_slice()));
        let acc_flags = mi.acc_flags;
        let code = mi.get_code();
        let lnt = mi.get_line_number_table();

        Self {
            class,
            name,
            desc,
            id,
            acc_flags,
            code,
            lnt,
            vis_annos,
            vis_param_annos,
        }
    }

    pub fn get_id(&self) -> BytesRef {
        self.id.clone()
    }

    pub fn find_exception_handler(&self, cp: &ConstantPool, pc: U2, ex: ClassRef) -> Option<U2> {
        match &self.code {
            Some(code) => {
                for e in code.exceptions.iter() {
                    if e.contains(pc) {
                        if e.is_finally() {
                            return Some(e.handler_pc);
                        }

                        if let Some(class) = runtime::require_class2(e.catch_type, cp) {
                            if runtime::cmp::instance_of(ex.clone(), class) {
                                return Some(e.handler_pc);
                            }
                        }
                    }
                }
            }

            _ => (),
        }

        None
    }

    pub fn check_annotation(&self, name: &[u8]) -> bool {
        for it in self.vis_annos.iter() {
            if it.type_name.as_slice() == name {
                return true;
            }
        }

        for it in self.vis_param_annos.iter() {
            if it.type_name.as_slice() == name {
                return true;
            }
        }

        false
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

    pub fn is_synchronized(&self) -> bool {
        (self.acc_flags & ACC_SYNCHRONIZED) == ACC_SYNCHRONIZED
    }

    pub fn is_native(&self) -> bool {
        (self.acc_flags & ACC_NATIVE) == ACC_NATIVE
    }

    pub fn is_abstract(&self) -> bool {
        (self.acc_flags & ACC_ABSTRACT) == ACC_ABSTRACT
    }
}
