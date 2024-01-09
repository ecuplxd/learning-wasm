use std::cell::RefCell;
use std::rc::Rc;

use super::errors::{InstError, LinkError, VMState};
use super::importer::{Importer, MImporter};
use super::inst::element::ElemInst;
use super::inst::function::FuncInst;
use super::inst::global::GlobalInst;
use super::inst::memory::{MemInst, Memory};
use super::inst::table::TableInst;
use super::inst::{ExportMap, RFuncInst, RGlobalInst, RMemInst, RTableInst};
use super::random_str;
use super::stack::frame::{CallStack, Frame};
use super::stack::operand::Operand;
use super::types::{LoadFrom, RefInst, ValInst, ValInsts};
use crate::binary::module::Module;
use crate::binary::section::{DataMode, ElementMode, ExportDesc, ImportDesc, ImportSeg};

#[derive(Debug, Default)]
pub struct VM {
    id: String,
    name: String,
    pub module: Rc<Module>,

    pub operands: ValInsts,
    pub frames: Vec<Frame>,

    pub funcs: Vec<RFuncInst>,
    pub tables: Vec<RTableInst>,
    pub mems: Vec<RMemInst>,
    pub globals: Vec<RGlobalInst>,

    pub exports: ExportMap,
    pub datas: Vec<Vec<u8>>,
    pub elements: Vec<ElemInst>,

    pub local_idx: usize,
    pub mem_idx: usize,
}

/// 构造函数
impl VM {
    pub fn new(name: &str, module: Module, maps: Option<MImporter>) -> VMState<Self> {
        let mut vm = Self {
            id: name.to_string() + "-" + &random_str(10),
            name: name.to_string(),
            module: Rc::new(module),
            ..Default::default()
        };

        if let Some(maps) = maps {
            if !maps.is_empty() {
                vm.resolve_imports(maps)?;
            }
        }

        vm.init()?;
        vm.call_start();

        Ok(vm)
    }

    pub fn from_file(name: &str, path: &str, importers: Option<MImporter>) -> VMState<Self> {
        let module = Module::from_file(path).expect("模块解析错误");

        Self::new(name, module, importers)
    }

    pub fn from_data(name: &str, data: Vec<u8>, importers: Option<MImporter>) -> VMState<Self> {
        let module = Module::from_data(data).expect("模块解析错误");

        Self::new(name, module, importers)
    }

    pub fn load_and_run(name: &str, kind: LoadFrom, importers: Option<MImporter>) -> VMState<Self> {
        match kind {
            LoadFrom::Data(data) => Self::from_data(name, data, importers),
            LoadFrom::File(path) => Self::from_file(name, path, importers),
            LoadFrom::Module(module) => Self::new(name, module, importers),
        }
    }
}

impl Importer for VM {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn resolve_func(&self, name: &str) -> Option<RFuncInst> {
        match self.exports.get(name) {
            Some(export) => match export.desc {
                ExportDesc::Func(idx) => Some(Rc::clone(&self.funcs[idx as usize])),
                _ => None,
            },
            None => None,
        }
    }

    fn resolve_table(&self, name: &str) -> Option<RTableInst> {
        match self.exports.get(name) {
            Some(export) => match export.desc {
                ExportDesc::Table(idx) => Some(Rc::clone(&self.tables[idx as usize])),
                _ => None,
            },
            None => None,
        }
    }

    fn resolve_mem(&self, name: &str) -> Option<RMemInst> {
        match self.exports.get(name) {
            Some(export) => match export.desc {
                ExportDesc::Mem(idx) => Some(Rc::clone(&self.mems[idx as usize])),
                _ => None,
            },
            None => None,
        }
    }

    fn resolve_global(&self, name: &str) -> Option<RGlobalInst> {
        match self.exports.get(name) {
            Some(export) => match export.desc {
                ExportDesc::Global(idx) => Some(Rc::clone(&self.globals[idx as usize])),
                _ => None,
            },
            None => None,
        }
    }

    fn call_by_name(&mut self, name: &str, args: ValInsts) -> ValInsts {
        match self.resolve_func(name) {
            Some(ptr) => {
                let func_inst = ptr.borrow();

                self.invoke(&func_inst, Some(args))
            }
            None => panic!("找不到函数：{}", name),
        }
    }
}

/// 实现操作数栈
impl Operand for VM {
    fn pop(&mut self) -> ValInst {
        self.operands.pop().expect("栈空")
    }

    fn push(&mut self, value: ValInst) {
        self.operands.push(value);
    }

    fn stack_size(&self) -> usize {
        self.operands.len()
    }

    fn get_value(&self, n: usize) -> &ValInst {
        &self.operands[n]
    }

    fn set_value(&mut self, n: usize, value: ValInst) {
        self.operands[n] = value;
    }
}

/// 实现调用栈
impl CallStack for VM {
    fn pop_frame(&mut self) -> Frame {
        self.frames.pop().expect("调用栈帧空")
    }

    fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame)
    }

    fn depth(&self) -> usize {
        self.frames.len()
    }

    fn top_frame(&self) -> &Frame {
        self.frames.last().expect("调用栈帧空")
    }

    fn top_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("调用栈帧空")
    }

    fn get_frame(&self, n: usize) -> &Frame {
        &self.frames[n]
    }
}

/// 实现内存
impl Memory for VM {
    fn mem_reads(&self, addr: u64, n: u64) -> Vec<u8> {
        self.mems[self.mem_idx].borrow().mem_reads(addr, n)
    }

    fn mem_writes(&mut self, addr: u64, bytes: &[u8]) -> VMState {
        self.mems[self.mem_idx].borrow_mut().mem_writes(addr, bytes)
    }

    fn mem_size(&self) -> u32 {
        self.mems[self.mem_idx].borrow().mem_size()
    }

    fn mem_grow(&mut self, size: u32) -> i32 {
        self.mems[self.mem_idx].borrow_mut().mem_grow(size)
    }
}

impl VM {
    fn reset(&mut self) {
        self.operands = vec![];
        self.frames = vec![];
    }

    pub fn start_loop(&mut self) {
        let depth = self.depth();

        while self.depth() >= depth {
            let frame = self.top_mut();

            match unsafe { frame.expr.as_ref() } {
                Some(expr) if frame.pc == expr.len() => self.exit_block(),
                Some(expr) => {
                    let instr = &expr[frame.pc];

                    frame.pc += 1;

                    self.exec_instr(instr);
                }
                None => panic!("frame {:?} 找不到可以执行的指令", frame),
            };
        }
    }

    // 执行入口函数
    pub fn call_start(&mut self) {
        if let Some(idx) = self.module.start_sec {
            let func_inst = Rc::clone(&self.funcs[idx as usize]);

            {
                let func_inst = func_inst.borrow();

                self.invoke(&func_inst, Some(vec![]));
            }
        }
    }
}

/// 初始化的所有逻辑
impl VM {
    // 处理导入
    fn resolve_imports(&mut self, importers: MImporter) -> VMState {
        let module = Rc::clone(&self.module);

        for import in &module.import_sec {
            match importers.get(&import.module) {
                Some(importer) => self.resolve_import(import, Rc::clone(importer))?,
                _ => Err(LinkError::ModuleNotFound(import.module.clone()))?,
            }
        }

        Ok(())
    }

    fn resolve_import(&mut self, import: &ImportSeg, importer_: Rc<RefCell<dyn Importer>>) -> VMState {
        let importer = importer_.borrow();
        let module_name = importer.get_name().to_string();
        let item_name = import.name.clone();

        match &import.desc {
            ImportDesc::Func(idx) => match importer.resolve_func(&import.name) {
                Some(func_inst) => {
                    let func_inst = func_inst.borrow();
                    let sig = &self.module.type_sec[*idx as usize];

                    if func_inst.get_type() != sig {
                        Err(LinkError::IncompatibleImportType)?;
                    }

                    {
                        let func_inst = func_inst.as_outer(Rc::clone(&importer_), &import.name);

                        self.funcs.push(func_inst);

                        Ok(())
                    }
                }
                _ => Err(LinkError::ItemNotFound(module_name, item_name))?,
            },
            ImportDesc::Table(type_) => match importer.resolve_table(&import.name) {
                Some(inst) => {
                    if inst.borrow().get_type().incompatible(type_) {
                        Err(LinkError::IncompatibleImportType)?;
                    }

                    self.tables.push(inst);

                    Ok(())
                }
                _ => Err(LinkError::ItemNotFound(module_name, item_name))?,
            },
            ImportDesc::Mem(type_) => match importer.resolve_mem(&import.name) {
                Some(inst) => {
                    if inst.borrow().get_type().incompatible(type_) {
                        println!("{:?} {:?}", inst.borrow().get_type(), type_);
                        Err(LinkError::IncompatibleImportType)?;
                    }

                    self.mems.push(inst);

                    Ok(())
                }
                _ => Err(LinkError::ItemNotFound(module_name, item_name))?,
            },
            ImportDesc::Global(type_) => match importer.resolve_global(&import.name) {
                Some(inst) => {
                    if inst.borrow().get_type() != type_ {
                        Err(LinkError::IncompatibleImportType)?;
                    }

                    self.globals.push(inst);

                    Ok(())
                }
                _ => Err(LinkError::ItemNotFound(module_name, item_name))?,
            },
        }
    }

    fn init(&mut self) -> VMState {
        let module = Rc::clone(&self.module);

        self.init_funcs(module.as_ref());
        self.init_table_and_elem(module.as_ref())?;
        self.init_mem_and_data(module.as_ref())?;
        self.init_global(module.as_ref());

        for export in &module.export_sec {
            self.exports.insert(export.name.clone(), export.clone());
        }

        Ok(())
    }

    // 初始化内存：定义了内存才能使用 data 段，下表、元素段同理
    fn init_mem_and_data(&mut self, module: &Module) -> VMState {
        for mem in &module.mem_sec {
            let mem_inst = MemInst::new(mem.clone());

            self.mems.push(Rc::new(RefCell::new(mem_inst)));
        }

        // 初始化 data
        for (i, data) in module.data_sec.iter().enumerate() {
            self.datas.push(data.init.to_vec());

            if matches!(data.mode, DataMode::Active) {
                data.offset_expr.iter().for_each(|instr| self.exec_instr(instr));

                // 初始化完成后，此时栈顶就是内存起始地址
                let addr = self.pop().as_mem_addr();
                let mut mem = self.mems[data.mem_idx as usize].borrow_mut();

                mem.mem_writes(addr, &self.datas[i])?;

                self.datas[i].clear();
            }
        }

        Ok(())
    }

    // 初始化函数段
    fn init_funcs(&mut self, module: &Module) {
        // 内部函数
        for (i, ft_idx) in module.func_sec.iter().enumerate() {
            let ft = &module.type_sec[*ft_idx as usize];
            let code = &module.code_sec[i];
            let func_inst = FuncInst::from_wasm(ft.clone(), code, self.get_id());

            self.funcs.push(Rc::new(RefCell::new(func_inst)));
        }
    }

    // 初始化全局段
    fn init_global(&mut self, module: &Module) {
        for global in &module.global_sec {
            global.init_expr.iter().for_each(|instr| self.exec_instr(instr));

            let global_inst = GlobalInst::new(global.type_.clone(), self.pop());

            self.globals.push(Rc::new(RefCell::new(global_inst)));
        }
    }

    // 初始化表
    fn init_table_and_elem(&mut self, module: &Module) -> VMState {
        for table_type in &module.table_sec {
            let table = TableInst::new(table_type.clone());

            self.tables.push(Rc::new(RefCell::new(table)));
        }

        for elem in &module.elem_sec {
            let refs = match elem.init_is_expr() {
                true => elem
                    .init_expr
                    .iter()
                    .map(|expr| {
                        // 其实只有 1 个指令
                        expr.iter().for_each(|instr| self.exec_instr(instr));

                        self.pop()
                    })
                    .collect(),
                false => elem
                    .func_idxs
                    .iter()
                    .map(|idx| {
                        let func_inst = &self.funcs[*idx as usize];
                        let ref_inst = RefInst(*idx, Rc::clone(func_inst));

                        ValInst::new_ref(elem.type_, Some(ref_inst), Some(*idx))
                    })
                    .collect::<Vec<_>>(),
            };

            let elem_inst = ElemInst::new(elem.type_, refs);

            self.elements.push(elem_inst);
        }

        // 初始化元素段
        for (i, elem) in module.elem_sec.iter().enumerate() {
            match &elem.mode {
                ElementMode::Active {
                    table_idx,
                    offset_expr: offset,
                } => {
                    offset.iter().for_each(|instr| self.exec_instr(instr));

                    let offset = self.pop_u32();
                    let elem_inst = &mut self.elements[i];
                    let mut table = self.tables[*table_idx as usize].borrow_mut();

                    for (i, ref_val) in elem_inst.refs.iter().enumerate() {
                        table.set_elem(offset + (i as u32), ref_val.clone())?;
                    }

                    if offset > table.size() {
                        Err(InstError::OutofBoundTable)?;
                    }

                    elem_inst.drop_();
                }
                ElementMode::Declarative => self.elements[i].drop_(),
                ElementMode::Passive => continue,
            }
        }

        Ok(())
    }
}
