use std::collections::HashSet;

use super::errors::ValidateErr;
use super::instruction::Instruction;
use super::module::Module;
use super::section::{
    CodeSeg, DataMode, DataSeg, ElementMode, ElementSeg, ExportDesc, ExportSeg, Expr, GlobalSeg,
    ImportDesc, ImportSeg, StartSeg, TypeIdx,
};
use super::types::{FuncType, MemType, TableType, ValType};

pub type ValidateResult<T = ()> = Result<T, ValidateErr>;

pub trait Validate {
    fn validate(&self) -> ValidateResult {
        Ok(())
    }

    fn validate_use_module(&self, _module: &Module) -> ValidateResult {
        Ok(())
    }
}

impl Module {
    pub fn validates<T>(secs: &Vec<T>, module: &Module) -> ValidateResult
    where
        T: Validate,
    {
        for sec in secs {
            sec.validate_use_module(module)?;
        }

        Ok(())
    }
}

/// 导入段
impl Validate for ImportSeg {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        match &self.desc {
            ImportDesc::Func(idx) => idx.validate_use_module(module),
            ImportDesc::Table(type_) => type_.validate_use_module(module),
            ImportDesc::Mem(type_) => type_.validate_use_module(module),
            ImportDesc::Global(_) => Ok(()),
        }
    }
}

/// 函数（类型索引）段
impl Validate for TypeIdx {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        match (*self as usize) > module.type_sec.len() {
            true => Err(ValidateErr::FnTypeNotFound(*self))?,
            false => Ok(()),
        }
    }
}

/// 表段
impl Validate for TableType {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        let exit_import_table = module
            .import_sec
            .iter()
            .any(|import| matches!(import.desc, ImportDesc::Table(_)));

        if exit_import_table {
            Err(ValidateErr::ExitImportTable)?;
        }

        let limits = &self.limits;

        match limits.max {
            Some(max) if max < limits.min => Err(ValidateErr::MaxLtMin(max, limits.min))?,
            Some(max) if max > 232 => Err(ValidateErr::MaxTooLarge(max, 232))?,
            _ => Ok(()),
        }
    }
}

/// 内存段
impl Validate for MemType {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        let exit_import_memory = module
            .import_sec
            .iter()
            .any(|import| matches!(import.desc, ImportDesc::Mem(_)));

        if exit_import_memory {
            Err(ValidateErr::ExitImportMem)?;
        }

        match self.max {
            Some(max) if max < self.min => Err(ValidateErr::MaxLtMin(max, self.min))?,
            Some(max) if max > 216 => Err(ValidateErr::MaxTooLarge(max, 216))?,
            _ => Ok(()),
        }
    }
}

/// 全局段
impl Validate for GlobalSeg {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        let val_type = validate_const_expr(&self.init_expr, &module.global_sec)?;

        match val_type != self.type_.val_type {
            true => Err(ValidateErr::ExprRetNotEq(val_type, self.type_.val_type)),
            false => Ok(()),
        }
    }
}

/// 导出段
impl Validate for Vec<ExportSeg> {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        let mut names: HashSet<String> = HashSet::new();
        let mut dups: Vec<String> = vec![];

        for export in self {
            export.desc.validate_use_module(module)?;

            let name = &export.name;

            if !names.insert(name.clone()) {
                dups.push(name.to_string());
            }
        }

        match !dups.is_empty() {
            true => Err(ValidateErr::DuplicateExport(dups))?,
            false => Ok(()),
        }
    }
}

impl Validate for ExportDesc {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        match self {
            ExportDesc::Func(i) if module.func_sec.get(*i as usize).is_none() => {
                Err(ValidateErr::FnNotFound(*i))
            }
            ExportDesc::Table(i) if module.table_sec.get(*i as usize).is_none() => {
                Err(ValidateErr::TableNotFound(*i))
            }
            ExportDesc::Mem(i) if module.mem_sec.get(*i as usize).is_none() => {
                Err(ValidateErr::MemNotFound(*i))
            }
            ExportDesc::Global(i) if module.global_sec.get(*i as usize).is_none() => {
                Err(ValidateErr::GlobalVarNotFound(*i))
            }
            _ => Ok(()),
        }
    }
}

/// 开始段
impl Validate for StartSeg {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        if let Some(idx) = self {
            let idx = *idx as usize;
            let mut func_type: Option<FuncType> = None;
            let import_idxs = module
                .import_sec
                .iter()
                .filter(|import| matches!(import.desc, ImportDesc::Func(_)))
                .map(|import| match import.desc {
                    ImportDesc::Func(i) => i,
                    _ => panic!(""),
                })
                .collect::<Vec<_>>();
            let total = import_idxs.len();

            if idx < total {
                func_type = match import_idxs.get(idx) {
                    Some(i) => module.type_sec.get(*i as usize).cloned(),
                    _ => None,
                };
            }

            let func_total = (module.func_sec.len()) + total;

            if idx < func_total {
                let i = module.func_sec[idx - total];

                func_type = module.type_sec.get(i as usize).cloned();
            }

            match func_type {
                Some(func_type) => {
                    if !func_type.params.is_empty() {
                        Err(ValidateErr::StartFnNoParam(func_type.params))?;
                    }

                    if !func_type.results.is_empty() {
                        Err(ValidateErr::StartFnNoResult(func_type.results))?;
                    }
                }
                None => Err(ValidateErr::FnTypeNotFound(idx as u32))?,
            }
        }

        Ok(())
    }
}

/// 元素段
impl Validate for ElementSeg {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        match &self.mode {
            ElementMode::Passive => Ok(()),
            ElementMode::Active {
                table_idx,
                offset_expr: offset,
            } => {
                let idx = *table_idx as usize;

                if idx > module.table_sec.len() {
                    Err(ValidateErr::TableNotFound(idx as u32))?;
                }

                let val_type = validate_const_expr(offset, &module.global_sec)?;

                if val_type != ValType::I32 {
                    Err(ValidateErr::OffsetRetNotEqI32(val_type))?;
                }

                let func_total = (module.func_sec.len() + import_func_total(module)) as u32;

                for func_idx in &self.func_idxs {
                    if *func_idx > func_total {
                        Err(ValidateErr::FnTypeNotFound(*func_idx))?;
                    }
                }

                Ok(())
            }
            ElementMode::Declarative => Ok(()),
        }
    }
}

/// 代码段
impl Validate for CodeSeg {
    fn validate(&self) -> ValidateResult {
        Ok(())
    }
}

/// 数据段
impl Validate for DataSeg {
    fn validate_use_module(&self, module: &Module) -> ValidateResult {
        match &self.mode {
            DataMode::Passive => Ok(()),
            DataMode::Active => {
                let idx = self.mem_idx as usize;

                if idx > module.mem_sec.len() {
                    Err(ValidateErr::MemNotFound(self.mem_idx))?;
                }

                let val_type = validate_const_expr(&self.offset_expr, &module.global_sec)?;

                match val_type != ValType::I32 {
                    true => Err(ValidateErr::OffsetRetNotEqI32(val_type))?,
                    false => Ok(()),
                }
            }
        }
    }
}

fn validate_const_expr(expr: &Expr, globals: &[GlobalSeg]) -> ValidateResult<ValType> {
    if expr.len() != 1 {
        Err(ValidateErr::InitExprLen(expr.len()))?;
    }

    match &expr[0] {
        Instruction::I32Const(_) => Ok(ValType::I32),
        Instruction::I64Const(_) => Ok(ValType::I64),
        Instruction::F32Const(_) => Ok(ValType::F32),
        Instruction::F64Const(_) => Ok(ValType::F64),
        Instruction::GlobalGet(idx) => match globals.get(*idx as usize) {
            Some(global) if !global.type_.is_const() => Err(ValidateErr::GlobalVarNotConst(*idx))?,
            Some(global) => Ok(global.type_.val_type),
            None => Err(ValidateErr::GlobalVarNotFound(*idx))?,
        },
        instr => Err(ValidateErr::InitNotConst(instr.discriminant()))?,
    }
}

fn import_func_total(module: &Module) -> usize {
    module
        .import_sec
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Func(_)))
        .count()
}
