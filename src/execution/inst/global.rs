use crate::binary::types::GlobalType;
use crate::execution::errors::{Trap, VMState};
use crate::execution::value::ValInst;

#[derive(Debug)]
pub struct GlobalInst(GlobalType, ValInst);

impl GlobalInst {
    pub fn new(type_: GlobalType, val: ValInst) -> VMState<Self> {
        if type_.val_type != val.get_type() {
            Err(Trap::GlobalTypeNotEq)?
        }

        Ok(Self(type_, val))
    }

    pub fn get_type(&self) -> &GlobalType {
        &self.0
    }

    pub fn value(&self) -> ValInst {
        self.1.clone()
    }

    pub fn set(&mut self, value: ValInst) -> VMState {
        if self.0.is_const() {
            Err(Trap::GlobalVarConst)?;
        }

        self.0.val_type = value.get_type();
        self.1 = value;

        Ok(())
    }
}
