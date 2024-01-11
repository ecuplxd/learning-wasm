use crate::binary::types::ValType;
use crate::execution::value::ValInsts;

#[derive(Debug)]
pub struct ElemInst {
    pub type_: ValType,
    pub refs: ValInsts,
}

impl ElemInst {
    pub fn new(type_: ValType, refs: ValInsts) -> Self {
        Self { type_, refs }
    }

    pub fn drop_(&mut self) {
        self.refs.clear();
    }
}
