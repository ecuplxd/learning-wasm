use std::io;

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

    #[error("函数段和代码段数量不匹配")]
    FuncAndCodeNotEq,

    #[error("数据段和 DataCount 不匹配")]
    DataAndDataCountNotEq
}
