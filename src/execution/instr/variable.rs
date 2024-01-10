use std::rc::Rc;

use crate::execution::errors::VMState;
use crate::execution::stack::operand::Operand;
use crate::execution::vm::VM;

impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-local-get
    pub fn local_get(&mut self, idx: u32) {
        let idx = idx as usize;
        let val = self.get_value(self.local_idx + idx);

        self.push(val.clone());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-local-set
    pub fn local_set(&mut self, idx: u32) {
        let idx = idx as usize;
        let v1 = self.pop();

        self.set_value(self.local_idx + idx, v1);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-local-tee
    pub fn local_tee(&mut self, idx: u32) {
        let v1 = self.pop();
        let idx = idx as usize;

        self.push(v1.clone());
        self.set_value(self.local_idx + idx, v1);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-global-get
    pub fn global_get(&mut self, idx: u32) {
        let global = Rc::clone(&self.globals[idx as usize]);

        {
            let global = global.borrow();
            let value = global.value();

            self.push(value);
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-global-set
    pub fn global_set(&mut self, idx: u32) -> VMState {
        let v1 = self.pop();
        let global = &self.globals[idx as usize];

        global.borrow_mut().set(v1)
    }
}
