use super::Interp;

impl<'a> Interp<'a> {
    #[inline]
    pub fn pop(&self) {
        self.frame.area.stack.borrow_mut().drop_top();
    }
    #[inline]
    pub fn pop2(&self) {
        let mut stack = self.frame.area.stack.borrow_mut();
        stack.drop_top();
        stack.drop_top();
    }
    #[inline]
    pub fn dup(&self) {
        self.frame.area.stack.borrow_mut().dup();
    }
    #[inline]
    pub fn dup_x1(&self) {
        self.frame.area.stack.borrow_mut().dup_x1();
    }
    #[inline]
    pub fn dup_x2(&self) {
        self.frame.area.stack.borrow_mut().dup_x2();
    }
    #[inline]
    pub fn dup2(&self) {
        self.frame.area.stack.borrow_mut().dup2();
    }
    #[inline]
    pub fn dup2_x1(&self) {
        self.frame.area.stack.borrow_mut().dup2_x1();
    }
    #[inline]
    pub fn dup2_x2(&self) {
        self.frame.area.stack.borrow_mut().dup2_x2();
    }
    #[inline]
    pub fn swap(&self) {
        self.frame.area.stack.borrow_mut().swap();
    }
}
