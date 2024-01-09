use std::error::Error;

pub type VMState<T = ()> = Result<T, Box<dyn Error>>;

#[derive(thiserror::Error, Debug)]
pub enum InstError {
    #[error("内存越界访问")]
    OutofBoundMem,

    #[error("表越界访问")]
    OutofBoundTable,

    #[error("Unreachable")]
    Unreachable,
}

#[derive(thiserror::Error, Debug)]
pub enum LinkError {
    #[error("找不到模块：{0}")]
    ModuleNotFound(String),

    #[error("模块 {0} 中找不到导入项：{1}")]
    ItemNotFound(String, String),

    #[error("模块 {0} 中找不到导出项：{1}")]
    ExportNotFound(String, String),

    #[error("导入项类型不匹配")]
    IncompatibleImportType,
}
