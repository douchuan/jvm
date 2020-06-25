use crate::oop;
use crate::runtime::DataArea;
use crate::types::*;
use classfile::{types::U1, ConstantPool};
use std::sync::Arc;

pub struct Frame {
    pub frame_id: usize, //for debug
    pub class: ClassRef,
    //avoid lock class to access cp
    pub cp: ConstantPool,
    pub mir: MethodIdRef,
    pub code: Arc<Vec<U1>>,

    pub pc: std::sync::atomic::AtomicI32,
    pub op_widen: std::sync::atomic::AtomicBool,
    pub ex_here: std::sync::atomic::AtomicBool,

    // The variable part of Frame is placed here
    pub area: DataArea,
}

// unsafe impl Sync for Frame {}

//new
impl Frame {
    pub fn new(mir: MethodIdRef, frame_id: usize) -> Self {
        let class = mir.method.class.clone();
        let cp = {
            let cls_obj = class.extract_inst();
            cls_obj.class_file.cp.clone()
        };

        // trace!("method.code.is_some = {}", mir.method.code.is_some());
        match &mir.method.code {
            Some(code) => {
                // trace!("max_locals = {}, max_stack = {}", code.max_locals, code.max_stack);
                let area = DataArea::new(code.max_locals as usize, code.max_stack as usize);
                let code = code.code.clone();

                Self {
                    frame_id,
                    class,
                    cp,
                    mir,
                    code,
                    pc: std::sync::atomic::AtomicI32::new(0),
                    op_widen: std::sync::atomic::AtomicBool::new(false),
                    ex_here: std::sync::atomic::AtomicBool::new(false),
                    area,
                }
            }

            None => Self {
                frame_id,
                class,
                cp: Arc::new(Vec::new()),
                mir,
                code: Arc::new(vec![]),
                pc: std::sync::atomic::AtomicI32::new(0),
                op_widen: std::sync::atomic::AtomicBool::new(false),
                ex_here: std::sync::atomic::AtomicBool::new(false),
                area: DataArea::new(0, 0),
            },
        }
    }
}
