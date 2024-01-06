use crate::binary::types::ValType;
use crate::execution::stack::operand::Operand;
use crate::execution::vm::VM;

impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-drop
    pub fn drop_(&mut self) {
        self.pop();
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-select
    pub fn select(&mut self) {
        let v3 = self.pop_bool();
        let v2 = self.pop();
        let v1 = self.pop();

        match v3 {
            true => self.push(v1),
            false => self.push(v2),
        }
    }

    pub fn select2(&mut self, x: u8, type_: &ValType) {
        self.select();
    }
}
