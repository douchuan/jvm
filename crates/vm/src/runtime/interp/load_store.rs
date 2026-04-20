use super::Interp;
use std::sync::atomic::Ordering;

impl<'a> Interp<'a> {
    pub fn opcode_pos(&mut self) -> usize {
        let pc = &self.frame.pc;
        let codes = &self.code;
        let op_widen = self.op_widen;
        if op_widen {
            self.op_widen = false;
            super::read::read_u2(pc, codes)
        } else {
            super::read::read_u1(pc, codes)
        }
    }

    #[inline]
    pub fn iload(&mut self) {
        let pos = self.opcode_pos();
        let v = self.local.borrow().get_int(pos);
        self.frame.area.stack.borrow_mut().push_int(v);
    }

    #[inline]
    pub fn lload(&mut self) {
        let pos = self.opcode_pos();
        let v = self.local.borrow().get_long(pos);
        self.frame.area.stack.borrow_mut().push_long(v);
    }

    #[inline]
    pub fn fload(&mut self) {
        let pos = self.opcode_pos();
        let v = self.local.borrow().get_float(pos);
        self.frame.area.stack.borrow_mut().push_float(v);
    }

    #[inline]
    pub fn dload(&mut self) {
        let pos = self.opcode_pos();
        let v = self.local.borrow().get_double(pos);
        self.frame.area.stack.borrow_mut().push_double(v);
    }

    #[inline]
    pub fn aload(&mut self) {
        let pos = self.opcode_pos();
        let v = self.local.borrow().get_ref(pos);
        self.frame.area.stack.borrow_mut().push_ref(v, false);
    }

    #[inline]
    pub fn iload_0(&self) {
        let v = self.local.borrow().get_int(0);
        self.frame.area.stack.borrow_mut().push_int(v);
    }
    #[inline]
    pub fn iload_1(&self) {
        let v = self.local.borrow().get_int(1);
        self.frame.area.stack.borrow_mut().push_int(v);
    }
    #[inline]
    pub fn iload_2(&self) {
        let v = self.local.borrow().get_int(2);
        self.frame.area.stack.borrow_mut().push_int(v);
    }
    #[inline]
    pub fn iload_3(&self) {
        let v = self.local.borrow().get_int(3);
        self.frame.area.stack.borrow_mut().push_int(v);
    }

    #[inline]
    pub fn lload_0(&self) {
        let v = self.local.borrow().get_long(0);
        self.frame.area.stack.borrow_mut().push_long(v);
    }
    #[inline]
    pub fn lload_1(&self) {
        let v = self.local.borrow().get_long(1);
        self.frame.area.stack.borrow_mut().push_long(v);
    }
    #[inline]
    pub fn lload_2(&self) {
        let v = self.local.borrow().get_long(2);
        self.frame.area.stack.borrow_mut().push_long(v);
    }
    #[inline]
    pub fn lload_3(&self) {
        let v = self.local.borrow().get_long(3);
        self.frame.area.stack.borrow_mut().push_long(v);
    }

    #[inline]
    pub fn fload_0(&self) {
        let v = self.local.borrow().get_float(0);
        self.frame.area.stack.borrow_mut().push_float(v);
    }
    #[inline]
    pub fn fload_1(&self) {
        let v = self.local.borrow().get_float(1);
        self.frame.area.stack.borrow_mut().push_float(v);
    }
    #[inline]
    pub fn fload_2(&self) {
        let v = self.local.borrow().get_float(2);
        self.frame.area.stack.borrow_mut().push_float(v);
    }
    #[inline]
    pub fn fload_3(&self) {
        let v = self.local.borrow().get_float(3);
        self.frame.area.stack.borrow_mut().push_float(v);
    }

    #[inline]
    pub fn dload_0(&self) {
        let v = self.local.borrow().get_double(0);
        self.frame.area.stack.borrow_mut().push_double(v);
    }
    #[inline]
    pub fn dload_1(&self) {
        let v = self.local.borrow().get_double(1);
        self.frame.area.stack.borrow_mut().push_double(v);
    }
    #[inline]
    pub fn dload_2(&self) {
        let v = self.local.borrow().get_double(2);
        self.frame.area.stack.borrow_mut().push_double(v);
    }
    #[inline]
    pub fn dload_3(&self) {
        let v = self.local.borrow().get_double(3);
        self.frame.area.stack.borrow_mut().push_double(v);
    }

    #[inline]
    pub fn aload_0(&self) {
        let v = self.local.borrow().get_ref(0);
        self.frame.area.stack.borrow_mut().push_ref(v, false);
    }
    #[inline]
    pub fn aload_1(&self) {
        let v = self.local.borrow().get_ref(1);
        self.frame.area.stack.borrow_mut().push_ref(v, false);
    }
    #[inline]
    pub fn aload_2(&self) {
        let v = self.local.borrow().get_ref(2);
        self.frame.area.stack.borrow_mut().push_ref(v, false);
    }
    #[inline]
    pub fn aload_3(&self) {
        let v = self.local.borrow().get_ref(3);
        self.frame.area.stack.borrow_mut().push_ref(v, false);
    }

    #[inline]
    pub fn istore(&mut self) {
        let pos = self.opcode_pos();
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.local.borrow_mut().set_int(pos, v);
    }
    #[inline]
    pub fn lstore(&mut self) {
        let pos = self.opcode_pos();
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.local.borrow_mut().set_long(pos, v);
    }
    #[inline]
    pub fn fstore(&mut self) {
        let pos = self.opcode_pos();
        let v = self.frame.area.stack.borrow_mut().pop_float();
        self.local.borrow_mut().set_float(pos, v);
    }
    #[inline]
    pub fn dstore(&mut self) {
        let pos = self.opcode_pos();
        let v = self.frame.area.stack.borrow_mut().pop_double();
        self.local.borrow_mut().set_double(pos, v);
    }
    #[inline]
    pub fn astore(&mut self) {
        let pos = self.opcode_pos();
        let v = self.frame.area.stack.borrow_mut().pop_ref();
        self.local.borrow_mut().set_ref(pos, v);
    }

    #[inline]
    pub fn istore_0(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.local.borrow_mut().set_int(0, v);
    }
    #[inline]
    pub fn istore_1(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.local.borrow_mut().set_int(1, v);
    }
    #[inline]
    pub fn istore_2(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.local.borrow_mut().set_int(2, v);
    }
    #[inline]
    pub fn istore_3(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_int();
        self.local.borrow_mut().set_int(3, v);
    }

    #[inline]
    pub fn lstore_0(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.local.borrow_mut().set_long(0, v);
    }
    #[inline]
    pub fn lstore_1(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.local.borrow_mut().set_long(1, v);
    }
    #[inline]
    pub fn lstore_2(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.local.borrow_mut().set_long(2, v);
    }
    #[inline]
    pub fn lstore_3(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_long();
        self.local.borrow_mut().set_long(3, v);
    }

    #[inline]
    pub fn fstore_0(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_float();
        self.local.borrow_mut().set_float(0, v);
    }
    #[inline]
    pub fn fstore_1(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_float();
        self.local.borrow_mut().set_float(1, v);
    }
    #[inline]
    pub fn fstore_2(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_float();
        self.local.borrow_mut().set_float(2, v);
    }
    #[inline]
    pub fn fstore_3(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_float();
        self.local.borrow_mut().set_float(3, v);
    }

    #[inline]
    pub fn dstore_0(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_double();
        self.local.borrow_mut().set_double(0, v);
    }
    #[inline]
    pub fn dstore_1(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_double();
        self.local.borrow_mut().set_double(1, v);
    }
    #[inline]
    pub fn dstore_2(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_double();
        self.local.borrow_mut().set_double(2, v);
    }
    #[inline]
    pub fn dstore_3(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_double();
        self.local.borrow_mut().set_double(3, v);
    }

    #[inline]
    pub fn astore_0(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_ref();
        self.local.borrow_mut().set_ref(0, v);
    }
    #[inline]
    pub fn astore_1(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_ref();
        self.local.borrow_mut().set_ref(1, v);
    }
    #[inline]
    pub fn astore_2(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_ref();
        self.local.borrow_mut().set_ref(2, v);
    }
    #[inline]
    pub fn astore_3(&self) {
        let v = self.frame.area.stack.borrow_mut().pop_ref();
        self.local.borrow_mut().set_ref(3, v);
    }
}
