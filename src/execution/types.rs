use core::{fmt, simd};
use std::simd::u8x16;

use super::errors::{Trap, VMState};
use super::inst::RFuncInst;
use crate::binary::instruction::Lane16;
use crate::binary::module::Module;
use crate::binary::section::MaybeU32;
use crate::binary::types::ValType;

pub trait ToV128 {
    fn v128(self) -> v128;
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub struct v128(pub i32, pub i32, pub i32, pub i32);

impl v128 {
    pub fn new(bytes: Lane16) -> Self {
        let v = u8x16::from_array(bytes);

        v.v128()
    }

    pub fn all_zero(&self) -> bool {
        self.0 == 0 && self.1 == 0 && self.2 == 0 && self.3 == 0
    }
}

macro_rules! conversions {
    ($(($name:ident = $ty:ty))*) => {
        impl v128 {
            $(
                #[inline(always)]
                pub fn $name(self) -> $ty {
                    unsafe { std::mem::transmute(self) }
                }
            )*
        }
        $(
            impl ToV128 for $ty {
                #[inline(always)]
                fn v128(self) -> v128 {
                    unsafe { std::mem::transmute(self) }
                }
            }
        )*
    }
}

conversions! {
    (as_u8x16 = simd::u8x16)
    (as_u16x8 = simd::u16x8)
    (as_u32x4 = simd::u32x4)
    (as_u64x2 = simd::u64x2)
    (as_i8x16 = simd::i8x16)
    (as_i16x8 = simd::i16x8)
    (as_i32x4 = simd::i32x4)
    (as_i64x2 = simd::i64x2)
    (as_f32x4 = simd::f32x4)
    (as_f64x2 = simd::f64x2)
}

// 目前（2.0）只能是函数引用
pub type RefInst = RFuncInst;

#[derive(Clone)]
pub enum ValInst {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    V128(v128),
    FuncRef(Option<RefInst>),
    ExternRef(MaybeU32),
    NullRef,
}

pub type ValInsts = Vec<ValInst>;

impl ValInst {
    pub fn new_ref_null(v: u8) -> Self {
        let val_type = ValType::from(v);

        match val_type {
            ValType::FuncRef => Self::FuncRef(None),
            ValType::ExternRef => Self::ExternRef(None),
            _ => panic!("不是一个有效的引用：{:?}", val_type),
        }
    }

    pub fn new_ref(val_type: ValType, ref_inst: Option<RefInst>, idx: MaybeU32) -> Self {
        match val_type {
            ValType::FuncRef => Self::new_func_ref(ref_inst.unwrap()),
            ValType::ExternRef => Self::new_extern_ref(idx.unwrap()),
            _ => panic!("不是一个有效的引用：{:?}", val_type),
        }
    }

    pub fn new_func_ref(ref_inst: RefInst) -> Self {
        Self::FuncRef(Some(ref_inst))
    }

    pub fn new_extern_ref(ref_inst: u32) -> Self {
        Self::ExternRef(Some(ref_inst))
    }

    pub fn get_type(&self) -> ValType {
        match self {
            Self::I32(_) => ValType::I32,
            Self::I64(_) => ValType::I64,
            Self::F32(_) => ValType::F32,
            Self::F64(_) => ValType::F64,
            Self::V128(_) => ValType::V128,
            Self::FuncRef(_) => ValType::FuncRef,
            Self::ExternRef(_) => ValType::ExternRef,
            Self::NullRef => ValType::NullRef,
        }
    }
}

/// 值
impl ValInst {
    pub fn as_u32(&self) -> u32 {
        match self {
            Self::I32(v) => *v as u32,
            _ => panic!("不能将 {:?} 转为 U32，类型不匹配", self),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Self::I64(v) => *v as u64,
            _ => panic!("不能将 {:?} 转为 U64，类型不匹配", self),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            Self::I32(v) => *v,
            _ => panic!("不能将 {:?} 转为 I32，类型不匹配", self),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Self::I64(v) => *v,
            _ => panic!("不能将 {:?} 转为 I64，类型不匹配", self),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Self::F32(v) => *v,
            _ => panic!("不能将 {:?} 转为 F32，类型不匹配", self),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Self::F64(v) => *v,
            _ => panic!("不能将 {:?} 转为 F64，类型不匹配", self),
        }
    }

    pub fn as_v128(&self) -> v128 {
        match self {
            Self::V128(v) => *v,
            _ => panic!("不能将 {:?} 转为 V128，类型不匹配", self),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::I32(v) => *v != 0,
            Self::I64(v) => *v != 0,
            Self::F32(v) => *v != 0.0,
            Self::F64(v) => *v != 0.0,
            Self::V128(v) => !v.all_zero(),
            Self::FuncRef(v) => v.is_some(),
            Self::ExternRef(v) => v.is_some(),
            Self::NullRef => false,
        }
    }

    pub fn as_ref_inst(&self) -> VMState<&RefInst> {
        match self {
            ValInst::FuncRef(v) => match v {
                Some(ref_inst) => Ok(ref_inst),
                None => Err(Trap::NullRef)?,
            },
            _ => Err(Trap::InvalidRef)?,
        }
    }

    pub fn as_func_inst(&self) -> VMState<&RFuncInst> {
        self.as_ref_inst()
    }

    pub fn as_mem_addr(&self) -> u64 {
        match self {
            ValInst::I32(v) => *v as i64 as u64,
            ValInst::I64(v) => *v as u64,
            _ => panic!("无效的地址：{:?}", self),
        }
    }
}

impl From<u32> for ValInst {
    fn from(v: u32) -> Self {
        Self::I32(v as i32)
    }
}

impl From<u64> for ValInst {
    fn from(v: u64) -> Self {
        Self::I64(v as i64)
    }
}

impl From<i32> for ValInst {
    fn from(v: i32) -> Self {
        Self::I32(v)
    }
}

impl From<i64> for ValInst {
    fn from(v: i64) -> Self {
        Self::I64(v)
    }
}

impl From<f32> for ValInst {
    fn from(v: f32) -> Self {
        Self::F32(v)
    }
}

impl From<f64> for ValInst {
    fn from(v: f64) -> Self {
        Self::F64(v)
    }
}

impl From<v128> for ValInst {
    fn from(v: v128) -> Self {
        Self::V128(v)
    }
}

impl From<&ValType> for ValInst {
    fn from(type_: &ValType) -> Self {
        match type_ {
            ValType::I32 => Self::I32(0),
            ValType::I64 => Self::I64(0),
            ValType::F32 => Self::F32(0.0),
            ValType::F64 => Self::F64(0.0),
            ValType::V128 => Self::V128(v128(0, 0, 0, 0)),
            ValType::FuncRef => Self::FuncRef(None),
            ValType::ExternRef => Self::ExternRef(None),
            ValType::NullRef => Self::NullRef,
        }
    }
}

impl fmt::Debug for ValInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32(v) => write!(f, "I32({:?})", v),
            Self::I64(v) => write!(f, "I64({:?})", v),
            Self::F32(v) => write!(f, "F32({:?})", v),
            Self::F64(v) => write!(f, "F64({:?})", v),
            Self::V128(v) => write!(f, "V128({:?})", v),
            Self::FuncRef(None) => write!(f, "Null FuncRef"),
            Self::FuncRef(Some(v)) => write!(f, "FuncRef({:?})", v),
            Self::ExternRef(None) => write!(f, "Null ExternRef"),
            Self::ExternRef(Some(v)) => write!(f, "ExternRef({:?})", v),
            Self::NullRef => write!(f, "NullRef"),
        }
    }
}

impl PartialEq for ValInst {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::I32(a), Self::I32(b)) => a == b,
            (Self::I64(a), Self::I64(b)) => a == b,
            (Self::F32(a), Self::F32(b)) => match a.is_nan() && b.is_nan() {
                true => true,
                false => a == b,
            },
            (Self::F64(a), Self::F64(b)) => match a.is_nan() && b.is_nan() {
                true => true,
                false => a == b,
            },
            (Self::V128(a), Self::V128(b)) => a == b,
            (Self::FuncRef(a), Self::FuncRef(b)) => a == b,
            (Self::ExternRef(a), Self::ExternRef(b)) => a == b,
            (Self::NullRef, Self::NullRef) => true,
            _ => false,
        }
    }
}

pub enum LoadFrom<'a> {
    File(&'a str),
    Data(Vec<u8>),
    Module(Module),
}
