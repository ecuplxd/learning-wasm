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
