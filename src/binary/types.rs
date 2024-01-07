use super::errors::DecodeErr;
use super::instruction::BlockType;
use super::reader::DecodeResult;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum RefType {
    #[default]
    FuncRef = 0x70,
    ExternRef = 0x6f,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValType {
    I32 = 0x7f,
    I64 = 0x7e,
    F32 = 0x7d,
    F64 = 0x7c,     // 数值
    V128 = 0x7b,    // 向量
    FuncRef = 0x70, // 引用
    ExternRef = 0x6f,
    NullRef = 0x6b,
}

impl From<u8> for ValType {
    fn from(v: u8) -> Self {
        match v {
            0x7f => Self::I32,
            0x7e => Self::I64,
            0x7d => Self::F32,
            0x7c => Self::F64,
            0x7b => Self::V128,
            0x70 => Self::FuncRef,
            0x6f => Self::ExternRef,
            0x6b => Self::NullRef,
            _ => panic!("无效的数值类型：{}", v),
        }
    }
}

impl ValType {
    fn is_num_type(&self) -> bool {
        matches!(self, Self::I32 | Self::I64 | Self::F32 | Self::F64)
    }

    fn is_vec_type(&self) -> bool {
        matches!(self, Self::V128)
    }

    fn is_ref_type(&self) -> bool {
        matches!(self, Self::FuncRef | Self::ExternRef | Self::NullRef)
    }
}

pub type ResultType = Vec<ValType>;

#[derive(Debug, Default, Clone)]
pub struct FuncType {
    pub params: ResultType,
    pub results: ResultType,
}

impl FuncType {
    pub fn new_param(param_type: ValType, size: usize) -> Self {
        Self {
            params: vec![param_type; size],
            results: vec![],
        }
    }

    pub fn new_params(params: Vec<ValType>) -> Self {
        Self {
            params,
            results: vec![],
        }
    }

    pub fn new_result(ret_type: ValType) -> Self {
        Self {
            params: vec![],
            results: vec![ret_type],
        }
    }
}

impl From<&BlockType> for FuncType {
    fn from(v: &BlockType) -> Self {
        match v {
            BlockType::I32 => Self::new_result(ValType::I32),
            BlockType::I64 => Self::new_result(ValType::I64),
            BlockType::F32 => Self::new_result(ValType::F32),
            BlockType::F64 => Self::new_result(ValType::F64),
            BlockType::V128 => Self::new_result(ValType::V128),
            BlockType::ExternRef => Self::new_result(ValType::ExternRef),
            BlockType::Empty => Self::default(),
            BlockType::TypeIdx(_) => Self::default(),
        }
    }
}

impl PartialEq for FuncType {
    fn eq(&self, rhs: &Self) -> bool {
        if self.params.len() != rhs.params.len() {
            return false;
        }

        self.params.iter().zip(rhs.params.iter()).all(|(a, b)| a == b)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

pub type MemType = Limits;

#[derive(Debug, Default, Clone)]
pub struct TableType {
    pub elem_type: RefType,
    pub limits: Limits,
}

#[derive(Debug, Clone)]
pub struct GlobalType {
    pub val_type: ValType,
    pub mut_: Mut,
}

impl GlobalType {
    pub fn is_const(&self) -> bool {
        self.mut_ == Mut::Const
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mut {
    Const = 0x00,
    Var = 0x01,
}

impl Mut {
    pub fn from_u8(v: u8) -> DecodeResult<Self> {
        match v {
            0x00 => Ok(Self::Const),
            0x01 => Ok(Self::Var),
            _ => Err(DecodeErr::InvalidMut(v))?,
        }
    }
}
