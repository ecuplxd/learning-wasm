use super::errors::DecodeErr;
use super::instruction::Instruction;
use super::reader::DecodeResult;
use super::types::{GlobalType, MemType, TableType, ValType};
use crate::execution::types::{ValInst, ValInsts};

/// Indices
// 简写 x
pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type TableIdx = u32;
pub type MemIdx = u32;
pub type GlobalIdx = u32;
pub type LocalIdx = u32;
// 简写 l
pub type LabelIdx = u32;

/// Sections
#[repr(u8)]
#[derive(Debug)]
pub enum Section {
    Custom = 0x00,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
}

impl Section {
    pub fn from_u8(v: u8) -> DecodeResult<Self> {
        match v {
            0x00 => Ok(Self::Custom),
            0x01 => Ok(Self::Type),
            0x02 => Ok(Self::Import),
            0x03 => Ok(Self::Function),
            0x04 => Ok(Self::Table),
            0x05 => Ok(Self::Memory),
            0x06 => Ok(Self::Global),
            0x07 => Ok(Self::Export),
            0x08 => Ok(Self::Start),
            0x09 => Ok(Self::Element),
            0x0a => Ok(Self::Code),
            0x0b => Ok(Self::Data),
            0x0c => Ok(Self::DataCount),
            _ => Err(DecodeErr::UnexpectedSection(v))?,
        }
    }
}

/// Custom Section
/// https://webassembly.github.io/spec/core/appendix/custom.html
#[derive(Debug, Default)]
pub struct CustomSeg {
    pub name: String,
    pub data: Vec<u8>,
}

/// Import Section
#[derive(Debug)]
pub struct ImportSeg {
    pub module: String, // 模块名
    pub name: String,   // 成员名
    pub desc: ImportDesc,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx) = 0x00,
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

/// Global Section
#[derive(Debug)]
pub struct GlobalSeg {
    pub type_: GlobalType,
    pub init_expr: Expr,
}

/// Export Section
pub type Expr = Vec<Instruction>;

#[derive(Debug, Clone)]
pub struct ExportSeg {
    pub name: String,
    pub desc: ExportDesc,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ExportDesc {
    Func(FuncIdx) = 0x00,
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

pub enum ExternalKind {
    Func,
    Table,
    Memory,
    Global,
    Tag,
}

/// Element Section 存放表初始化数据
#[derive(Debug)]
pub struct ElementSeg {
    pub flag: u32,
    pub mode: ElementMode,
    pub type_: ValType,
    /// 目前是 0x00，funcref
    pub elem_kind: i32,
    pub func_idxs: Vec<FuncIdx>,
    pub init_expr: Vec<Expr>,
}

impl ElementSeg {
    pub fn init_is_expr(&self) -> bool {
        !self.init_expr.is_empty()
    }

    pub fn drop_(&mut self) {
        self.func_idxs.clear();
        self.init_expr.clear();
    }
}

#[derive(Debug)]
pub enum ElementMode {
    /// table.init
    Passive,
    /// copies its elements into a table during instantiation
    Active { table_idx: TableIdx, offset_expr: Expr },
    /// ref.func
    Declarative,
}

/// Code Section
#[derive(Debug)]
pub struct CodeSeg {
    /// 冗余校验
    pub size: u32,
    pub locals: Vec<Locals>,
    pub body: Expr,
}

impl CodeSeg {
    pub fn local_size(&self) -> u64 {
        self.locals.iter().map(|local| local.n as u64).sum()
    }

    pub fn init_local(&self) -> ValInsts {
        let mut values = vec![];

        for local in &self.locals {
            for _ in 0..local.n {
                values.push(ValInst::from(&local.value_type))
            }
        }

        values
    }
}

#[derive(Debug)]
pub struct Locals {
    pub n: u32,
    pub value_type: ValType,
}

/// Data Section 存放内存初始化数据
#[derive(Debug)]
pub struct DataSeg {
    pub flag: u32,
    pub mode: DataMode,
    pub init: Vec<u8>,
    /// 仅 Active 可用
    pub mem_idx: MemIdx,
    pub offset_expr: Expr,
}

impl DataSeg {
    pub fn drop_(&mut self) {
        self.init.clear();
    }
}

#[derive(Debug)]
pub enum DataMode {
    /// 使用 mem.init 指令初始化
    Passive,
    /// 实例化时拷贝到内存
    Active,
}

pub const ACTIVE_0: u32 = 0;
pub const ACTIVE_2: u32 = 2;
pub const ACTIVE_4: u32 = 4;
pub const ACTIVE_6: u32 = 6;

pub const PASSIVE_1: u32 = 1;
pub const PASSIVE_5: u32 = 5;

pub const DECLARATIVE_3: u32 = 3;
pub const DECLARATIVE_7: u32 = 7;
