use super::Interp;
use crate::oop::{self, Oop};
use std::sync::atomic::Ordering;
use tracing::{debug, error, info, trace, warn};

impl<'a> Interp<'a> {
    pub fn try_handle_exception(&self, ex: Oop) -> Result<(), Oop> {
        let ex_cls = {
            let rf = ex.extract_ref();
            oop::with_heap(|heap| {
                let desc = heap.get(rf);
                let guard = desc.read().unwrap();
                guard.v.extract_inst().class.clone()
            })
        };
        let handler = {
            let pc = self.frame.pc.load(Ordering::Relaxed);
            self.frame
                .mir
                .method
                .find_exception_handler(&self.cp, pc as u16, ex_cls)
        };
        match handler {
            Some(pc) => {
                let mut stack = self.frame.area.stack.borrow_mut();
                stack.clear();
                stack.push_ref(ex, false);
                drop(stack);
                let line_num = self.frame.mir.method.get_line_num(pc);
                info!(
                    "Found Exception Handler: line={}, frame_id={}, {:?}",
                    line_num, self.frame.frame_id, self.frame.mir.method
                );
                self.goto_abs(pc as i32);
                Ok(())
            }
            None => {
                let pc = self.frame.pc.load(Ordering::Relaxed);
                let line_num = self.frame.mir.method.get_line_num(pc as u16);
                info!(
                    "NotFound Exception Handler: line={}, frame_id={}, {:?}",
                    line_num, self.frame.frame_id, self.frame.mir.method,
                );
                Err(ex)
            }
        }
    }
}
