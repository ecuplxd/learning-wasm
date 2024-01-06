use crate::binary::types::GlobalType;
use crate::execution::types::ValInst;

#[derive(Debug)]
pub struct GlobalInst(GlobalType, ValInst);

impl GlobalInst {
    pub fn new(type_: GlobalType, val: ValInst) -> Self {
        if type_.val_type != val.get_type() {
            panic!(
                "Global 类型 {:?} 和值 {:?} 类型 {:?} 不匹配",
                type_.val_type,
                val,
                val.get_type()
            );
        }

        Self(type_, val)
    }

    pub fn value(&self) -> ValInst {
        self.1.clone()
    }

    pub fn set(&mut self, value: ValInst) {
        if self.0.is_const() {
            panic!("该全局变量不可变，不能进行修改");
        }

        self.0.val_type = value.get_type();
        self.1 = value;
    }
}
