use super::memory::MAX_PAGE_SIZE;
use super::RFuncInst;
use crate::binary::types::TableType;
use crate::execution::types::{ValInst, ValInsts};

/// 表
#[derive(Debug, Default)]
pub struct TableInst {
    type_: TableType,
    elems: ValInsts,
}

impl TableInst {
    pub fn new(type_: TableType) -> Self {
        let init_val = ValInst::new_ref_null(type_.elem_type as u8);

        Self {
            elems: vec![init_val; type_.limits.min as usize], // 不能用 with_capacity 进行初始化，会取不到值
            type_,
        }
    }

    pub fn get_type(&self) -> &TableType {
        &self.type_
    }

    pub fn size(&self) -> u32 {
        self.elems.len() as u32
    }

    pub fn grow(&mut self, size: u32, ref_val: ValInst) -> i32 {
        let old_size = self.size();

        if size == 0 {
            return old_size as i32;
        }

        let (new_size, overflow) = size.overflowing_add(old_size);

        if overflow {
            return -1;
        }

        let max = self.type_.limits.max;

        match max {
            Some(max) if max < new_size => {
                println!("{} 超过该表可以分配的总上限：{}", new_size, max);

                return -1;
            }
            None if new_size > MAX_PAGE_SIZE => {
                println!("{} 超过可以分配的总上限：{}", new_size, MAX_PAGE_SIZE);

                return -1;
            }
            _ => self.elems.resize(new_size as usize, ref_val),
        }

        old_size as i32
    }

    pub fn get_func_inst(&self, idx: u32) -> &RFuncInst {
        let ref_val = &self.elems[idx as usize];

        ref_val.as_func_inst()
    }

    pub fn get_elem(&self, idx: u32) -> &ValInst {
        &self.elems[idx as usize]
    }

    pub fn set_elem(&mut self, idx: u32, ref_val: ValInst) {
        self.elems[idx as usize] = ref_val;
    }

    pub fn get_elems(&self, src: u32, size: u32) -> &[ValInst] {
        let src = src as usize;
        let size = size as usize;

        &self.elems[src..src + size]
    }

    pub fn set_elems(&mut self, offset: u32, refs: &[ValInst]) {
        let offset = offset as usize;

        for (i, ref_val) in refs.iter().enumerate() {
            self.elems[i + offset] = ref_val.clone();
        }
    }

    pub fn drop_(&mut self) {
        self.elems.clear();
    }
}
