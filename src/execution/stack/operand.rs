use crate::execution::value::{v128, ValInst, ValInsts};

pub trait Operand {
    fn stack_size(&self) -> usize;
    fn get_value(&self, n: usize) -> &ValInst;
    fn set_value(&mut self, n: usize, v: ValInst);

    fn pop(&mut self) -> ValInst;
    fn push(&mut self, v: ValInst);

    fn pop_u64(&mut self) -> u64 {
        self.pop().as_u64()
    }

    fn push_u64(&mut self, v: u64) {
        self.push(ValInst::from(v));
    }

    fn pop_u32(&mut self) -> u32 {
        self.pop().as_u32()
    }

    fn push_u32(&mut self, v: u32) {
        self.push(ValInst::from(v));
    }

    fn pop_i32(&mut self) -> i32 {
        self.pop().as_i32()
    }

    fn push_i32(&mut self, v: i32) {
        self.push(ValInst::from(v));
    }

    fn pop_i64(&mut self) -> i64 {
        self.pop().as_i64()
    }

    fn push_i64(&mut self, v: i64) {
        self.push(ValInst::from(v));
    }

    fn pop_f32(&mut self) -> f32 {
        self.pop().as_f32()
    }

    fn push_f32(&mut self, v: f32) {
        self.push(ValInst::from(v));
    }

    fn pop_f64(&mut self) -> f64 {
        self.pop().as_f64()
    }

    fn push_f64(&mut self, v: f64) {
        self.push(ValInst::from(v));
    }

    fn pop_v128(&mut self) -> v128 {
        self.pop().as_v128()
    }

    fn push_v128(&mut self, v: v128) {
        self.push(ValInst::from(v));
    }

    fn pop_bool(&mut self) -> bool {
        self.pop().as_bool()
    }

    fn push_bool(&mut self, v: bool) {
        self.push_i32(v as i32);
    }

    fn pop_n(&mut self, n: usize) -> ValInsts {
        let mut vals: ValInsts = (0..n).map(|_| self.pop()).collect();

        vals.reverse();
        vals
    }

    fn push_n(&mut self, vals: ValInsts) {
        vals.into_iter().for_each(|val| self.push(val));
    }
}
