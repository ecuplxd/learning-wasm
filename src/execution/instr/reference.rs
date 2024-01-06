use crate::execution::stack::operand::Operand;
use crate::execution::types::{RefInst, ValInst};
use crate::execution::vm::VM;

impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-ref-null
    pub fn ref_null(&mut self, v: u64) {
        self.push(ValInst::new_ref_null(v as u8));
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-ref-is-null
    pub fn ref_is_null(&mut self) {
        let is_true = self.pop_bool();

        self.push_bool(!is_true);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-ref-func
    pub fn ref_func(&mut self, idx: u32) {
        let func_inst = &self.funcs[idx as usize];
        let ref_inst = RefInst(idx, func_inst.clone());

        self.push(ValInst::new_func_ref(ref_inst));
    }
}
