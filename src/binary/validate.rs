use std::collections::HashSet;

use super::instruction::Instruction;
use super::module::Module;
use super::section::{
    CodeSeg, DataMode, DataSeg, ElementMode, ElementSeg, ExportSeg, Expr, FuncIdx, GlobalSeg,
    ImportDesc, ImportSeg, TypeIdx,
};
use super::types::{FuncType, MemType, RefType, TableType, ValType};

pub trait Validate {
    fn validate_use_module(&self, _module: &Module) {}
}

pub trait ValidateSelf {
    fn validate(&self);
}

/// 导入段
impl Validate for ImportSeg {
    fn validate_use_module(&self, module: &Module) {
        match &self.desc {
            ImportDesc::Func(idx) => idx.validate_use_module(module),
            ImportDesc::Table(type_) => type_.validate_use_module(module),
            ImportDesc::Mem(type_) => type_.validate_use_module(module),
            ImportDesc::Global(_) => (),
        }
    }
}

/// 函数（类型索引）段
impl Validate for TypeIdx {
    fn validate_use_module(&self, module: &Module) {
        let idx = *self as usize;

        if idx > module.type_sec.len() {
            panic!("找不到索引 {} 对应的函数类型", idx);
        }
    }
}

/// 表段
impl Validate for TableType {
    fn validate_use_module(&self, module: &Module) {
        let exit_import_table = module
            .import_sec
            .iter()
            .any(|import| matches!(import.desc, ImportDesc::Table(_)));

        if exit_import_table {
            panic!("导入外部表段，则不能在模块内再次定义");
        }

        let limits = &self.limits;

        match limits.max {
            Some(max) if max < limits.min => panic!("指定的上限小于下限：{} < {}", max, limits.min),
            Some(max) if max > 232 => panic!("上限不能大于 {}", 232),
            _ => (),
        }

        if self.elem_type != RefType::FuncRef && self.elem_type != RefType::ExternRef {
            panic!("表元素只是是引用类型：{:?}", self.elem_type);
        }
    }
}

/// 内存段
impl Validate for MemType {
    fn validate_use_module(&self, module: &Module) {
        let exit_import_memory = module
            .import_sec
            .iter()
            .any(|import| matches!(import.desc, ImportDesc::Mem(_)));

        if exit_import_memory {
            panic!("导入外部内存，则不能在模块内再次定义");
        }

        match self.max {
            Some(max) if max < self.min => panic!("指定的上限小于下限：{} < {}", max, self.min),
            Some(max) if max > 216 => panic!("上限不能大于 {}", 216),
            _ => (),
        }
    }
}

/// 全局段
impl Validate for GlobalSeg {
    fn validate_use_module(&self, module: &Module) {
        if validate_const_expr(&self.init_expr, &module.global_sec) != self.type_.val_type {
            panic!(
                "表达式的结果 {:?} 和全局变量类型不一致 {:?}",
                self.type_.val_type, module.global_sec
            )
        }
    }
}

/// 导出段
impl ValidateSelf for Vec<ExportSeg> {
    fn validate(&self) {
        let mut names: HashSet<String> = HashSet::new();
        let mut dups: Vec<String> = vec![];

        for export in self {
            let name = &export.name;

            if !names.insert(name.clone()) {
                dups.push(name.to_string());
            }
        }

        if !dups.is_empty() {
            panic!("存在重复的导出项：{:?}", dups);
        }
    }
}

/// 开始段
impl Validate for Option<FuncIdx> {
    fn validate_use_module(&self, module: &Module) {
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
                        panic!("main 函数不接收任何参数：{:?}", func_type.params);
                    }

                    if !func_type.results.is_empty() {
                        panic!("main 函数没有返回值：{:?}", func_type.results);
                    }
                }
                None => panic!("找不到索引 {} 对应的函数类型声明", idx),
            }
        }
    }
}

/// 元素段
impl Validate for ElementSeg {
    fn validate_use_module(&self, module: &Module) {
        match &self.mode {
            ElementMode::Passive => todo!("validate ElementMode::Passive"),
            ElementMode::Active {
                table_idx,
                offset_expr: offset,
            } => {
                let idx = *table_idx as usize;

                if idx > module.table_sec.len() {
                    panic!("找不到索引 {} 对应的表", table_idx);
                }

                if validate_const_expr(offset, &module.global_sec) != ValType::I32 {
                    panic!("元素段 offset 初始表达式返回值应为 I32");
                }

                let func_total = (module.func_sec.len() + import_func_total(module)) as u32;

                for func_idx in &self.func_idxs {
                    if *func_idx > func_total {
                        panic!("找不到索引 {} 对应的函数", func_idx);
                    }
                }
            }
            ElementMode::Declarative => todo!("validate ElementMode::Declarative"),
        }
    }
}

/// 代码段
impl Validate for Vec<CodeSeg> {
    fn validate_use_module(&self, module: &Module) {
        if self.len() != module.func_sec.len() {
            panic!(
                "代码块数量 {} 和函数段数量 {} 不一致",
                self.len(),
                module.func_sec.len()
            );
        }

        for code in self {
            code.validate();
        }
    }
}

impl ValidateSelf for CodeSeg {
    fn validate(&self) {}
}

/// 数据段
impl Validate for DataSeg {
    fn validate_use_module(&self, module: &Module) {
        match &self.mode {
            DataMode::Passive => todo!("validate DataMode::Passive"),
            DataMode::Active => {
                let idx = self.mem_idx as usize;

                if idx > module.mem_sec.len() {
                    panic!("找不到索引 {} 对应的内存块", self.mem_idx);
                }

                if validate_const_expr(&self.offset_expr, &module.global_sec) != ValType::I32 {
                    panic!("数据段 offset 初始表达式返回值应为 I32");
                }
            }
        }
    }
}

fn validate_const_expr(expr: &Expr, globals: &[GlobalSeg]) -> ValType {
    if expr.len() != 1 {
        panic!("初始表达式的长度应为 1，现为 {}", expr.len());
    }

    match &expr[0] {
        Instruction::I32Const(_) => ValType::I32,
        Instruction::I64Const(_) => ValType::I64,
        Instruction::F32Const(_) => ValType::F32,
        Instruction::F64Const(_) => ValType::F64,
        Instruction::GlobalGet(idx) => match globals.get(*idx as usize) {
            Some(global) if !global.type_.is_const() => {
                panic!("索引 {} 对应的全局变量不是常量表达式", idx)
            }
            Some(global) => global.type_.val_type,
            None => panic!("找不到索引 {} 对应的全局变量", idx),
        },
        instr => panic!("只能用常量表达式进行初始化操作：{:?}", instr),
    }
}

fn import_func_total(module: &Module) -> usize {
    module
        .import_sec
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Func(_)))
        .count()
}
