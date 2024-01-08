use std::io;

use super::section::Section;
use super::types::ValType;

#[derive(thiserror::Error, Debug)]
pub enum DecodeErr {
    #[error("文件读取失败")]
    FileRead(#[from] io::Error),

    #[error("超出 LEB128 可编码的长度")]
    LEBDecodeTooLong,

    #[error("超出 LEB128 可编码的值")]
    IntTooLarge,

    #[error("LEB128 编码意外结束")]
    LEBUnexpectedEnd,

    #[error("wasm 文件头错误：{0:02X}")]
    MagicUnMatch(u32),

    #[error("wasm 版本错误：{0:02X}")]
    VersionUnMatch(u32),

    #[error("无效的 Section ID：{0:02X}")]
    UnexpectedSection(u8),

    #[error("无效的类型段值：{0:02X}")]
    InvalidType(u8),

    #[error("无效的可变性描述：{0:02X}")]
    InvalidMut(u8),

    #[error("未终结的表达式块")]
    ExprUnexpectedEnd,

    #[error("未知的指令码前缀：{0:02X}")]
    UnknownOpcodePrefix(u8),

    #[error("{0:02X} 段，未知的指令码：{1:02X}")]
    UnknownOpcode(u8, u8),

    #[error("无效的块表达式")]
    InvalidBlock,

    #[error("无效的 else 块表达式")]
    InvalidElseBlock,

    #[error("无效的表元素类型：{0:02X}")]
    InvalidTableElemType(u8),

    #[error("无效的导入类型：{0:02X}")]
    InvalidImportKind(u8),

    #[error("无效的导出类型：{0:02X}")]
    InvalidExportKind(u8),

    #[error("无效的元素模式：{0:02X}")]
    InvalidElemMode(u32),

    #[error("无效的数据模式：{0:02X}")]
    InvalidDataMode(u32),

    #[error("local count must be < 0x10000000")]
    LocalsTooLarge,

    #[error("表元素应为引用类型")]
    TableElemNotARef,

    #[error("section size mismatch")]
    SectionSizeMismatch,

    #[error("无效的 Limit 模式：{0:02X}")]
    InvalidLimitMode(u8),

    #[error("段 {0:?} 出现了 {1} 次，最多只能出现 1 次")]
    MultipleSection(Section, usize),

    #[error("代码块数量 {0} 和函数段数量 {1} 不一致")]
    FuncAndCodeNotEq(usize, usize),

    #[error("数据段数量 {0} 和 DataCount {1} 不一致")]
    DataAndDataCountNotEq(usize, usize),

    #[error("{0} 应为 0，现为 {1}")]
    NotZero(String, u8),

    #[error("{0} 指令需要 DataCount 段")]
    LossDataCount(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ValidateErr {
    #[error("找不到索引 {0} 对应的函数类型")]
    FuncIdxNotFound(usize),

    #[error("数据段 offset 初始表达式返回值应为 I32：{0:?}")]
    DataOffset(ValType),

    #[error("找不到索引 {0} 对应的内存块")]
    NotFoundMemByIdx(u32),

    #[error("初始表达式的长度应为 1，现为 {0}")]
    InitExprLen(usize),

    #[error("索引 {0} 对应的全局变量不是常量表达式")]
    GlobalVarNotConst(u32),

    #[error("找不到索引 {0} 对应的全局变量")]
    GlobalVarNotFound(u32),

    #[error("只能用常量表达式进行初始化操作：{0:02X?}")]
    InitNotByConst(u16),
}
