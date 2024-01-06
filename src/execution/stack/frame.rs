use crate::binary::section::Expr;

#[derive(Debug, PartialEq, Clone)]
pub enum LabelKind {
    Call,
    If,
    Loop,
    Block,
}

#[derive(Debug)]
pub struct Frame {
    pub kind: LabelKind,
    pub pc: usize,
    pub sp: usize,
    pub expr: *const Expr,
    pub arity: usize,   // 返回值数量
    pub arg_num: usize, // 参数数量
}

impl Frame {
    pub fn reset_pc(&mut self) {
        self.pc = 0;
    }

    pub fn basic_info(&self) -> Self {
        Self {
            expr: std::ptr::null(),
            pc: self.pc,
            sp: self.sp,
            arity: self.arity,
            arg_num: self.arg_num,
            kind: self.kind.clone(),
        }
    }
}

pub trait CallStack {
    fn pop_frame(&mut self) -> Frame;
    fn push_frame(&mut self, frame: Frame);
    fn depth(&self) -> usize;
    fn top_frame(&self) -> &Frame;
    fn top_mut(&mut self) -> &mut Frame;
    fn get_frame(&self, n: usize) -> &Frame;

    fn top_call(&self) -> (Option<&Frame>, i32) {
        for n in (0..self.depth()).rev() {
            let frame = self.get_frame(n);

            if frame.kind == LabelKind::Call {
                return (Some(frame), (self.depth() - 1 - n) as i32);
            }
        }

        (None, -1)
    }
}
