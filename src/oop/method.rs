use crate::classfile::{
    access_flags::*, attr_info::Code, constant_pool, consts, types::*, AttrType, FieldInfo,
    MethodInfo,
};
use crate::oop::{ClassObject, ClassRef, ValueType};
use crate::runtime;
use crate::util::{self, PATH_DELIMITER};

#[derive(Debug, Clone)]
pub struct MethodId {
    pub offset: usize,
    pub method: Method,
}

#[derive(Debug, Clone)]
pub struct Method {
    name: BytesRef,
    desc: BytesRef,
    id: String,
    acc_flags: U2,

    pub code: Code,
}

impl Method {
    pub fn new(cp: &ConstantPool, mi: &MethodInfo, class: &ClassObject) -> Self {
        let name = constant_pool::get_utf8(mi.name_index, cp).unwrap();
        let desc = constant_pool::get_utf8(mi.desc_index, cp).unwrap();
        let p1 = String::from_utf8_lossy(desc.as_slice());
        let p2 = String::from_utf8_lossy(name.as_slice());
        let id = vec![p1, p2].join(PATH_DELIMITER);
        let acc_flags = mi.acc_flags;
        let code = mi.get_code().clone();

        Self {
            name,
            desc,
            id,
            acc_flags,
            code,
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn find_exception_handler(&self, class: &ClassObject, pc: U2, ex: ClassRef) -> Option<U2> {
        let class_file = class.class_file.clone();
        let cp = &class_file.cp;

        for e in self.code.exceptions.iter() {
            if e.contains(pc) {
                if e.is_finally() {
                    return Some(e.handler_pc);
                }

                if let Some(class) = runtime::require_class2(e.catch_type, cp) {
                    if runtime::instance_of(ex.clone(), class) {
                        return Some(e.handler_pc);
                    }
                }
            }
        }

        None
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
}
