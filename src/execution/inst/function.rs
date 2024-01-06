use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::RFuncInst;
use crate::binary::section::CodeSeg;
use crate::binary::types::{FuncType, ValType};
use crate::execution::importer::Importer;
use crate::execution::random_str;

#[derive(Debug)]
pub struct FuncInst {
    id: String,
    type_: FuncType,
    from: String,
    pub kind: FuncInstKind,
}

impl FuncInst {
    pub fn from_wasm(ft: FuncType, code: &CodeSeg, from: &str) -> Self {
        Self {
            id: random_str(7),
            type_: ft,
            from: from.to_string(),
            kind: FuncInstKind::Inner(code as *const CodeSeg),
        }
    }

    pub fn from_importer(ft: FuncType, ctx: Rc<RefCell<dyn Importer>>, name: &str) -> Self {
        let from = ctx.borrow().get_id().to_string();

        Self {
            id: random_str(7),
            type_: ft,
            from,
            kind: FuncInstKind::Outer(ctx, name.to_string()),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_type(&self) -> &FuncType {
        &self.type_
    }

    pub fn arg_types(&self) -> &Vec<ValType> {
        &self.type_.params
    }

    pub fn ret_types(&self) -> &Vec<ValType> {
        &self.type_.results
    }
}

impl FuncInst {
    pub fn as_outer(&self, ctx: Rc<RefCell<dyn Importer>>, name: &str) -> RFuncInst {
        let ft = self.type_.clone();
        let mut func_inst = Self::from_importer(ft, ctx, name);

        func_inst.id = self.id.clone();

        Rc::new(RefCell::new(func_inst))
    }
}

pub enum FuncInstKind {
    Inner(*const CodeSeg),
    Outer(Rc<RefCell<dyn Importer>>, String),
}

impl fmt::Debug for FuncInstKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inner(code) => write!(f, "{:?}", unsafe { &code.as_ref() }),
            Self::Outer(_, v2) => write!(f, "外部函数：{}", v2),
        }
    }
}
