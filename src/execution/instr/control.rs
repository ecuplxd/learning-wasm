use std::rc::Rc;

use crate::binary::instruction::{Block, BlockType, BrTableArg, IfBlock};
use crate::binary::section::{CodeSeg, Expr, LabelIdx};
use crate::binary::types::{FuncType, ValType};
use crate::execution::importer::Importer;
use crate::execution::inst::function::{FuncInst, FuncInstKind};
use crate::execution::stack::frame::{CallStack, Frame, LabelKind};
use crate::execution::stack::operand::Operand;
use crate::execution::types::ValInsts;
use crate::execution::vm::VM;

/// 实现块逻辑
impl VM {
    pub fn enter_block(&mut self, kind: LabelKind, func_type: &FuncType, expr: &Expr) {
        let frame = Frame {
            pc: 0,
            kind,
            sp: self.stack_size() - func_type.params.len(),
            expr: expr as *const Expr,
            arg_num: func_type.params.len(),
            arity: func_type.results.len(),
        };

        if frame.kind == LabelKind::Call {
            self.local_idx = frame.sp;
        }

        self.push_frame(frame);
    }

    pub fn exit_block(&mut self) {
        let frame = self.pop_frame();

        self.clear_block(&frame);
    }

    pub fn reset_block(&mut self, frame: Frame) {
        let results = self.pop_n(frame.arg_num);

        self.pop_n(self.stack_size() - frame.sp);
        self.push_n(results);
    }

    pub fn clear_block(&mut self, frame: &Frame) {
        let results = self.pop_n(frame.arity);

        self.pop_n(self.stack_size() - frame.sp);
        self.push_n(results);

        if frame.kind == LabelKind::Call && self.depth() > 0 {
            let (call_frame, _) = self.top_call();

            if let Some(call_frame) = call_frame {
                self.local_idx = call_frame.sp;
            } else {
                panic!("找不到 call 调用栈帧");
            }
        }
    }

    pub fn block_type_to_func_type(&mut self, block_type: &BlockType) -> FuncType {
        let module = Rc::clone(&self.module);
        let func_type = match block_type {
            BlockType::TypeIdx(idx) => module.type_sec[*idx as usize].clone(),
            _ => FuncType::from(block_type),
        };

        func_type
    }
}

/// 实现函数调用逻辑
impl VM {
    /// args 存在（外部手动调用的情况），则要将参数压栈，结果出栈
    pub fn invoke(&mut self, func_inst: &FuncInst, args: Option<ValInsts>) -> ValInsts {
        let pop_push = args.is_some();
        let type_ = func_inst.get_type().clone();

        if pop_push {
            self.push_n_and_check_type(&type_.params, args.unwrap());
        }

        match &func_inst.kind {
            FuncInstKind::Inner(code) => unsafe { self.invoke_inner_func(*code, &type_) },
            FuncInstKind::Outer(ctx, name) => {
                // 存在嵌套调用，使用指针而不是 borrow_mut 绕过检查
                let ptr = ctx.as_ptr();
                let importer = unsafe { ptr.as_mut().unwrap() };

                self.invoke_outer_func(importer, name, &type_);
            }
        }

        match pop_push {
            true => self.pop_n_and_check_type(&type_.results),
            false => vec![],
        }
    }

    /// # Safety
    ///
    /// 调用内部函数 codeseg 必存在
    pub unsafe fn invoke_inner_func(&mut self, code_ptr: *const CodeSeg, type_: &FuncType) {
        match unsafe { code_ptr.as_ref() } {
            Some(code) => {
                self.enter_block(LabelKind::Call, type_, &code.body);
                self.push_n(code.init_local());

                self.start_loop();
            }
            None => panic!("{:?} 没有可供执行的函数体", code_ptr),
        }
    }

    pub fn invoke_outer_func(&mut self, importer: &mut dyn Importer, name: &str, type_: &FuncType) {
        let args = self.pop_n_and_check_type(&type_.params);
        let rets = importer.call_by_name(name, args);

        self.push_n_and_check_type(&type_.results, rets);
    }

    pub fn pop_n_and_check_type(&mut self, val_types: &[ValType]) -> ValInsts {
        // Todo: check
        let vals = self.pop_n(val_types.len());

        vals
    }

    pub fn push_n_and_check_type(&mut self, val_types: &[ValType], vals: ValInsts) {
        // Todo: check
        if val_types.len() == vals.len() {
            self.push_n(vals);
        }
    }
}

/// 实现指令逻辑
impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-unreachable
    pub fn unreachable(&mut self) {
        panic!("unreachable");
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-nop
    pub fn nop(&mut self) {}

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-block
    pub fn block(&mut self, block: &Block) {
        let ft = self.block_type_to_func_type(&block.type_);

        self.enter_block(LabelKind::Block, &ft, &block.expr);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-loop
    pub fn loop_(&mut self, block: &Block) {
        let ft = self.block_type_to_func_type(&block.type_);

        self.enter_block(LabelKind::Loop, &ft, &block.expr);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-if
    pub fn if_(&mut self, block: &IfBlock) {
        let ft = self.block_type_to_func_type(&block.type_);
        let expr = match self.pop_bool() {
            true => &block.if_expr,
            false => &block.else_expr,
        };

        self.enter_block(LabelKind::If, &ft, expr);
    }

    pub fn else_(&mut self) {}

    pub fn end(&mut self) {}

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-br
    pub fn br(&mut self, l: LabelIdx) {
        for _ in 0..l {
            self.pop_frame();
        }

        let frame = self.top_frame();

        match frame.kind {
            LabelKind::Loop => {
                self.reset_block(frame.basic_info());
                self.top_mut().reset_pc();
            }
            _ => self.exit_block(),
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-br-if
    pub fn br_if(&mut self, l: LabelIdx) {
        if self.pop_bool() {
            self.br(l);
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-br-table
    pub fn br_table(&mut self, arg: &BrTableArg) {
        let idx = self.pop_u32() as usize;

        match idx < arg.labels.len() {
            true => self.br(arg.labels[idx]),
            false => self.br(arg.default),
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-return
    pub fn return_(&mut self) {
        let (_, label_idx) = self.top_call();

        self.br(label_idx as u32);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-call
    pub fn call(&mut self, idx: u32) {
        let func_inst = Rc::clone(&self.funcs[idx as usize]);

        {
            let func_inst = func_inst.borrow();

            self.invoke(&func_inst, None);
        }
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-call-indirect
    pub fn call_indirect(&mut self, type_idx: u32, table_idx: u32) {
        let i = self.pop_u32();
        let table = Rc::clone(&self.tables[table_idx as usize]);

        {
            let table = table.borrow();

            if i >= table.size() {
                panic!("未定义的表元素：{}", i);
            }

            let module = Rc::clone(&self.module);
            let ft = &module.type_sec[type_idx as usize];
            let func_inst = table.get_func_inst(i);

            {
                let func_inst = func_inst.borrow();

                if ft != func_inst.get_type() {
                    panic!("间接调用参数 {:?} 不匹配，应为 {:?}", ft, func_inst.get_type());
                }

                self.invoke(&func_inst, None);
            }
        }
    }
}
