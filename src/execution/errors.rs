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

#[derive(thiserror::Error, Debug)]
pub enum Trap {
    #[error("Unreachable")]
    Unreachable,

    #[error("找不到函数")]
    FnNotFound,

    #[error("frame 找不到可以执行的指令")]
    NoOpcode,

    #[error("该全局变量不可变，不能进行修改")]
    GlobalVarConst,

    #[error("外部函数没有可供执行的函数体")]
    FnNoBody,

    #[error("找不到 call 调用栈帧")]
    CallFrameNotFount,

    #[error("间接调用参数不匹配")]
    ArgNotEq,

    #[error("类型不匹配")]
    ValTypeNotEq,

    #[error("Global 类型不匹配")]
    GlobalTypeNotEq,

    #[error("未初始化")]
    UnInitTableElem,

    #[error("不是一个有效的引用")]
    InvalidRef,

    #[error("OutofRange")]
    OutofRange,

    #[error("IntegerOverflow")]
    IntegerOverflow,

    #[error("DivZero")]
    DivZero,
}
