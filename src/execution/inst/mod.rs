use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use self::function::FuncInst;
use self::global::GlobalInst;
use self::memory::MemInst;
use self::table::TableInst;
use crate::binary::section::ExportSeg;

pub mod element;
pub mod function;
pub mod global;
pub mod memory;
pub mod table;

pub type ExportMap = HashMap<String, ExportSeg>;

pub type RFuncInst = Rc<RefCell<FuncInst>>;
pub type RTableInst = Rc<RefCell<TableInst>>;
pub type RMemInst = Rc<RefCell<MemInst>>;
pub type RGlobalInst = Rc<RefCell<GlobalInst>>;

pub enum ExportInst {
    Func(RFuncInst),
    Table(RTableInst),
    Mem(RMemInst),
    Global(RGlobalInst),
}
