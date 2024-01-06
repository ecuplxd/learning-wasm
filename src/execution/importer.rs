use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::inst::{RFuncInst, RGlobalInst, RMemInst, RTableInst};
use super::types::ValInsts;

pub trait Importer {
    fn get_name(&self) -> &str;

    fn get_id(&self) -> &str {
        self.get_name()
    }

    fn resolve_func(&self, _name: &str) -> Option<RFuncInst> {
        None
    }

    fn resolve_table(&self, _name: &str) -> Option<RTableInst> {
        None
    }

    fn resolve_mem(&self, _name: &str) -> Option<RMemInst> {
        None
    }

    fn resolve_global(&self, _name: &str) -> Option<RGlobalInst> {
        None
    }

    fn call_by_name(&mut self, name: &str, args: ValInsts) -> ValInsts;
}

pub type MImporter = HashMap<String, Rc<RefCell<dyn Importer>>>;

impl PartialEq for dyn Importer {
    fn eq(&self, rhs: &Self) -> bool {
        self.get_id() == rhs.get_id()
    }
}
