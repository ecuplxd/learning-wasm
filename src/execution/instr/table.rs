use std::rc::Rc;

use crate::execution::stack::operand::Operand;
use crate::execution::vm::VM;

impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-table-get
    pub fn table_get(&mut self, idx: u32) {
        let table = Rc::clone(&self.tables[idx as usize]);

        {
            let table = table.borrow_mut();
            let idx = self.pop_u32();
            let ref_val = table.get_elem(idx);

            self.push(ref_val.clone());
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-table-set
    pub fn table_set(&mut self, idx: u32) {
        let ref_val = self.pop();
        let i = self.pop_u32();

        self.tables[idx as usize].borrow_mut().set_elem(i, ref_val);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-table-init
    pub fn table_init(&mut self, elem_idx: u32, table_idx: u32) {
        let size = self.pop_u32() as usize;
        let src = self.pop_u32() as usize;
        let offset = self.pop_u32();

        let elem_inst = &self.elements[elem_idx as usize];
        let refs = &elem_inst.refs[src..src + size];

        self.tables[table_idx as usize]
            .borrow_mut()
            .set_elems(offset, refs);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-elem-drop
    pub fn elem_drop(&mut self, idx: u32) {
        self.elements[idx as usize].drop_();
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-table-copy
    /// 复制另一张表的数据
    pub fn table_copy(&mut self, dst_idx: u32, src_idx: u32) {
        let size = self.pop_u32();
        let src = self.pop_u32();
        let offset = self.pop_u32();

        let table = self.tables[src_idx as usize].borrow();
        let src_elems = table.get_elems(src, size).to_vec();

        // 两次借用
        drop(table);

        self.tables[dst_idx as usize]
            .borrow_mut()
            .set_elems(offset, &src_elems);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-table-grow
    pub fn table_grow(&mut self, idx: u32) {
        let size = self.pop_u32();
        let ref_val = self.pop();
        let table = Rc::clone(&self.tables[idx as usize]);

        {
            let mut table = table.borrow_mut();
            let old_size = table.grow(size, ref_val);

            self.push_i32(old_size);
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-table-size
    pub fn table_size(&mut self, idx: u32) {
        let table = Rc::clone(&self.tables[idx as usize]);

        {
            let table = table.borrow();
            let size = table.size();

            self.push_u32(size);
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-table-fill
    pub fn table_fill(&mut self, idx: u32) {
        let size = self.pop_u32() as usize;
        let ref_val = self.pop();
        // 从 offset 处开始
        let offset = self.pop_u32();
        let refs = vec![ref_val; size];

        self.tables[idx as usize].borrow_mut().set_elems(offset, &refs);
    }
}
