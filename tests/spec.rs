#![allow(dead_code)]
#![feature(portable_simd)]
#[cfg(test)]
mod models {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct WabtJson {
        source_filename: String,
        pub commands: Vec<Command>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Command {
        #[serde(flatten)]
        pub type_: CommandType,
        line: usize,
    }

    pub enum AssertKind {
        Return,
        Exhaustion,
        Trap,
        Invalid,
        Malformed,
        Uninstantiable,
        Unlinkable,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    #[serde(rename_all = "snake_case")]
    pub enum CommandType {
        Module(Module),
        Action { action: Action },
        AssertReturn(AssertReturn),
        AssertExhaustion(AssertExhaustion),
        AssertTrap(AssertTrap),
        AssertInvalid(AssertModule),
        AssertMalformed(AssertModule),
        AssertUninstantiable(AssertModule),
        AssertUnlinkable(AssertModule),
        Register(Register),
    }

    #[derive(Debug, Deserialize)]
    pub struct Module {
        pub name: Option<String>,
        pub filename: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    #[serde(rename_all = "lowercase")]
    pub enum Action {
        Get(GetAction),
        Invoke(InvokeAction),
    }

    #[derive(Debug, Deserialize)]
    pub struct InvokeAction {
        pub module: Option<String>,
        pub field: String,
        pub args: Vec<Const>,
    }

    #[derive(Debug, Deserialize)]
    pub struct GetAction {
        pub module: Option<String>,
        pub field: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "lowercase")]
    #[serde(tag = "type", content = "value")]
    pub enum Const {
        I32(String),
        I64(String),
        F32(String),
        F64(String),
        Externref(String),
        Funcref(String),
        Exnref(String),
        #[serde(untagged)]
        V128(SIMD),
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct SIMD {
        pub lane_type: LaneType,
        pub value: Vec<String>,
    }

    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "lowercase")]
    pub enum LaneType {
        I8,
        I16,
        I32,
        I64,
        F32,
        F64,
    }

    #[derive(Debug, Deserialize)]
    pub struct AssertReturn {
        pub action: Action,
        pub expected: Vec<Const>,
    }

    #[derive(Debug, Deserialize)]
    pub struct AssertExhaustion {
        pub action: Action,
        text: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct AssertTrap {
        pub action: Action,
        text: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct AssertModule {
        pub filename: String,
        pub text: String,
        pub module_type: ModuleType,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum ModuleType {
        Binary,
        Text,
    }

    #[derive(Debug, Deserialize)]
    pub struct Register {
        pub name: Option<String>,
        #[serde(rename = "as")]
        pub as_: String,
    }
}

mod into {
    use std::simd::{f32x4, f64x2, u16x8, u32x4, u64x2, u8x16};

    use wasm::execution::importer::Importer;
    use wasm::execution::types::{v128, ToV128, ValInst};

    use crate::models::{Const, LaneType, SIMD};

    impl Const {
        pub fn as_value(&self, vm: Option<&dyn Importer>) -> ValInst {
            match self {
                Const::I32(s) => ValInst::from(s.parse::<u32>().expect("不是 i32 字符串")),
                Const::I64(s) => ValInst::from(s.parse::<u64>().expect("不是 i64 字符串")),
                Const::F32(s) => ValInst::from(str_to_f32(s) as f32),
                Const::F64(s) => ValInst::from(str_to_f64(&s)),
                Const::Externref(s) => match s.parse::<u32>() {
                    Ok(idx) => ValInst::ExternRef(Some(idx)),
                    Err(_) => ValInst::ExternRef(None),
                },
                Const::Funcref(s) => match s.parse::<u32>() {
                    Ok(idx) => todo!("将 Funcref({}) 转为 ValInst", idx), // 目前看来测试集里都是 null
                    Err(_) => ValInst::FuncRef(None),
                },
                Const::Exnref(_) => ValInst::NullRef,
                Const::V128(simd) => ValInst::from(Into::<v128>::into(simd.clone())),
            }
        }

        pub fn as_values(consts: &Vec<Const>, vm: Option<&dyn Importer>) -> Vec<ValInst> {
            consts.iter().map(|item| item.as_value(vm)).collect()
        }
    }

    impl Into<v128> for SIMD {
        fn into(self) -> v128 {
            let iter = self.value.iter();

            match self.lane_type {
                LaneType::I8 => {
                    let arr = iter
                        .map(|s| s.parse::<u8>().expect("不是 i8 字符串"))
                        .collect::<Vec<_>>();

                    u8x16::from_slice(&arr).v128()
                }
                LaneType::I16 => {
                    let arr = iter
                        .map(|s| s.parse::<u16>().expect("不是 i16 字符串"))
                        .collect::<Vec<_>>();

                    u16x8::from_slice(&arr).v128()
                }
                LaneType::I32 => {
                    let arr = iter
                        .map(|s| s.parse::<u32>().expect("不是 i32 字符串"))
                        .collect::<Vec<_>>();

                    u32x4::from_slice(&arr).v128()
                }
                LaneType::I64 => {
                    let arr = iter
                        .map(|s| s.parse::<u64>().expect("不是 i64 字符串"))
                        .collect::<Vec<_>>();

                    u64x2::from_slice(&arr).v128()
                }
                LaneType::F32 => {
                    let arr = iter.map(|s| str_to_f32(s)).collect::<Vec<_>>();

                    f32x4::from_slice(&arr).v128()
                }
                LaneType::F64 => {
                    let arr = iter.map(|s| str_to_f64(s)).collect::<Vec<_>>();

                    f64x2::from_slice(&arr).v128()
                }
            }
        }
    }

    fn str_to_f32(num_str: &str) -> f32 {
        match num_str {
            "nan:canonical" => f32::from_bits(0x7fc0_0000),
            "nan:arithmetic" => f32::NAN,
            _ => f32::from_bits(num_str.parse::<u32>().expect("不是 f32 字符串")),
        }
    }

    fn str_to_f64(num_str: &str) -> f64 {
        match num_str {
            "nan:canonical" => f64::from_bits(0x7ff8_0000_0000_0000),
            "nan:arithmetic" => f64::NAN,
            _ => f64::from_bits(num_str.parse::<u64>().expect("不是 f64 字符串")),
        }
    }
}

mod run {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm::binary;
    use wasm::binary::encode::Encode;
    use wasm::binary::reader::DecodeResult;
    use wasm::execution::errors::VMState;
    use wasm::execution::importer::{Importer, MImporter};
    use wasm::execution::types::{LoadFrom, ValInst};
    use wasm::execution::vm::VM;

    use crate::models::{
        Action, AssertModule, AssertReturn, Const, GetAction, InvokeAction, LaneType, Module, ModuleType,
    };

    pub const LATEST_NAME: &str = "latest";

    impl Module {
        pub fn create(
            root: &str,
            filename: &str,
            name: Option<String>,
            maps: MImporter,
        ) -> (VMState<VM>, String) {
            println!("create module {} {:?}", filename, name);

            let file_path = root.to_string() + filename;
            let module = binary::module::Module::from_file(&file_path).unwrap();

            Module::test_module_encode(&module);

            let name = name.clone().unwrap_or(LATEST_NAME.to_string());
            let vm = VM::load_and_run(&name, LoadFrom::Module(module), Some(maps));

            (vm, name)
        }

        pub fn test_module_encode(module: &binary::module::Module) {
            let encode1 = module.encode();
            let d_module = binary::module::Module::from_data(encode1.clone()).unwrap();
            let encode2 = d_module.encode();
            let eq = encode1 == encode2;

            assert_eq!(eq, true);
        }

        pub fn get(name: Option<String>, maps: MImporter) -> Rc<RefCell<dyn Importer>> {
            let name = name.unwrap_or(LATEST_NAME.to_string());
            let vm_ = maps.get(&name);

            vm_.expect("无法获取到 vm 实例").clone()
        }
    }

    impl AssertModule {
        pub fn is_binary_module(&self) -> bool {
            self.module_type == ModuleType::Binary
        }

        pub fn create(&self, root: &str) -> DecodeResult<binary::module::Module> {
            let file_path = root.to_string() + &self.filename;

            println!("create assert module：{} assert：{} ", self.filename, self.text);

            binary::module::Module::from_file(&file_path)
        }
    }

    pub trait Execute {
        fn run(&self, maps: MImporter) -> (Vec<ValInst>, String);
    }

    impl Execute for Action {
        fn run(&self, maps: MImporter) -> (Vec<ValInst>, String) {
            match self {
                Action::Get(action) => action.run(maps),
                Action::Invoke(action) => action.run(maps),
            }
        }
    }

    impl Execute for GetAction {
        fn run(&self, maps: MImporter) -> (Vec<ValInst>, String) {
            let vm_ = Module::get(self.module.clone(), maps);
            let vm = vm_.borrow();
            let name = vm.get_name().to_string();

            match vm.resolve_global(&self.field) {
                Some(global) => (vec![global.borrow().value()], name),
                None => (vec![], name),
            }
        }
    }

    impl Execute for InvokeAction {
        fn run(&self, maps: MImporter) -> (Vec<ValInst>, String) {
            let vm_ = Module::get(self.module.clone(), maps);
            let mut vm = vm_.borrow_mut();
            let name = vm.get_name().to_string();
            let args = Const::as_values(&self.args, Some(&*vm));

            println!("{} 调用 {} 参数：{:?} ", vm.get_name(), self.field, args);

            if self.field == "grow" {
                println!("");
            }

            let rets = vm.call_by_name(&self.field, args);

            (rets, name)
        }
    }

    pub fn match_f32(ret: &ValInst, rhs: &ValInst, kind: &str) {
        match kind {
            "nan:canonical" => assert_eq!(is_f32_canonical_nan(ret.as_f32()), true),
            "nan:arithmetic" => assert_eq!(is_f32_arithmetic_nan(ret.as_f32()), true),
            _ => assert_eq!(ret, rhs),
        };
    }

    pub fn match_f64(ret: &ValInst, rhs: &ValInst, kind: &str) {
        match kind {
            "nan:canonical" => assert_eq!(is_f64_canonical_nan(ret.as_f64()), true),
            "nan:arithmetic" => assert_eq!(is_f64_arithmetic_nan(ret.as_f64()), true),
            _ => assert_eq!(ret, rhs),
        };
    }

    pub fn match_v128_f32(ret: &ValInst, expected: &Const) {
        let lhs = ret.as_v128().as_f32x4();
        let rhs = expected.as_value(None).as_v128().as_f32x4();
        let all_true = lhs
            .as_array()
            .iter()
            .zip(rhs.as_array())
            .all(|(lhs, rhs)| ValInst::F32(*lhs) == ValInst::F32(*rhs));

        assert_eq!(all_true, true);
    }

    pub fn match_v128_f64(ret: &ValInst, expected: &Const) {
        let lhs = ret.as_v128().as_f64x2();
        let rhs = expected.as_value(None).as_v128().as_f64x2();
        let all_true = lhs
            .as_array()
            .iter()
            .zip(rhs.as_array())
            .all(|(lhs, rhs)| ValInst::F64(*lhs) == ValInst::F64(*rhs));

        assert_eq!(all_true, true);
    }

    impl AssertReturn {
        pub fn run(&self, maps: MImporter) {
            let (rets, vm_name) = self.action.run(maps.clone());
            let temp = Module::get(Some(vm_name), maps);
            let vm = temp.borrow();

            let expecteds = self
                .expected
                .iter()
                .map(|item| item.as_value(Some(&*vm)))
                .collect::<Vec<_>>();

            println!("\t返回：{:?}\n\t期望：{:?}", rets, expecteds);

            for (i, ret) in rets.iter().enumerate() {
                let expected = &self.expected[i];

                match expected {
                    Const::F32(kind) => match_f32(ret, &expecteds[i], kind),
                    Const::F64(kind) => match_f64(ret, &expecteds[i], kind),
                    Const::V128(simd) => match simd.lane_type {
                        LaneType::F32 => match_v128_f32(ret, expected),
                        LaneType::F64 => match_v128_f64(ret, expected),
                        _ => assert_eq!(ret, &expecteds[i]),
                    },
                    _ => assert_eq!(ret, &expecteds[i]),
                }
            }
        }
    }

    fn is_f32_canonical_nan(val: f32) -> bool {
        const K_QUIET_NAN: u32 = 0x7fc00000;
        const K_QUIET_NEG_NAN: u32 = 0xffc00000;
        let bits = val.to_bits();

        (bits == K_QUIET_NAN) || (bits == K_QUIET_NEG_NAN)
    }

    fn is_f32_arithmetic_nan(val: f32) -> bool {
        val.is_nan()
    }

    fn is_f64_canonical_nan(val: f64) -> bool {
        const K_QUIET_NAN: u64 = 0x7ff8000000000000;
        const K_QUIET_NEG_NAN: u64 = 0xfff8000000000000;
        let bits = val.to_bits();

        (bits == K_QUIET_NAN) || (bits == K_QUIET_NEG_NAN)
    }

    // Todo
    fn is_f64_arithmetic_nan(val: f64) -> bool {
        val.is_nan()
    }
}

mod spec_test {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm::binary::types::{FuncType, GlobalType, Limits, Mut, RefType, TableType, ValType};
    use wasm::execution::importer::Importer;
    use wasm::execution::inst::function::FuncInst;
    use wasm::execution::inst::global::GlobalInst;
    use wasm::execution::inst::memory::MemInst;
    use wasm::execution::inst::table::TableInst;
    use wasm::execution::inst::{RFuncInst, RGlobalInst};
    use wasm::execution::types::{RefInst, ValInst, ValInsts};

    #[derive(Clone, Copy)]
    pub struct SpecTestModule;

    macro_rules! func {
        ($name:ident => $ret:expr) => {
            impl SpecTestModule {
                pub fn $name(args: ValInsts) -> ValInsts {
                    println!("call SpecTestModule::{} -> {:?}", stringify!($name), args);

                    $ret
                }
            }
        };
    }

    func!(print => vec![]);
    func!(print_i32 => vec![]);
    func!(print_i64 => vec![]);
    func!(print_f32 => vec![]);
    func!(print_f64 => vec![]);
    func!(print_i32_f32 => vec![]);
    func!(print_f64_f64 => vec![]);
    func!(ret_11 => vec![ValInst::I32(11)]);
    func!(ret_22 => vec![ValInst::I32(22)]);

    fn new_func_inst(func_inst: FuncInst) -> RFuncInst {
        Rc::new(RefCell::new(func_inst))
    }

    impl Importer for SpecTestModule {
        fn get_name(&self) -> &str {
            "spectest"
        }

        fn resolve_func(&self, name: &str) -> Option<RFuncInst> {
            let ft: FuncType = match name {
                "print" => FuncType::new_params(vec![]),
                "print_i32" => FuncType::new_param(ValType::I32, 1),
                "print_i64" => FuncType::new_param(ValType::I64, 1),
                "print_f32" => FuncType::new_param(ValType::F32, 1),
                "print_f64" => FuncType::new_param(ValType::F64, 1),
                "print_i32_f32" => FuncType::new_params(vec![ValType::I32, ValType::F32]),
                "print_f64_f64" => FuncType::new_params(vec![ValType::F64, ValType::F64]),
                _ => unimplemented!("SpecTestModule resolve_func：{}", name),
            };
            let self_ = Rc::new(RefCell::new(*self));
            let inst = Rc::new(RefCell::new(FuncInst::from_importer(ft, self_, name)));

            Some(Rc::clone(&inst))
        }

        fn resolve_table(&self, name: &str) -> Option<wasm::execution::inst::RTableInst> {
            let self_ = Rc::new(RefCell::new(*self));

            let ft = FuncType::new_result(ValType::I32);
            let ret_11 = new_func_inst(FuncInst::from_importer(ft, self_.clone(), "ret_11"));

            let ft = FuncType::new_result(ValType::I32);
            let ret_22 = new_func_inst(FuncInst::from_importer(ft, self_, "ret_22"));

            let mut table_inst = TableInst::new(TableType {
                elem_type: RefType::FuncRef,
                limits: Limits {
                    min: 10,
                    max: Some(20),
                },
            });
            let elems = vec![ret_11.clone(), ret_11.clone(), ret_22]
                .into_iter()
                .enumerate()
                .map(|(i, item)| ValInst::new_func_ref(RefInst(i as u32, item)))
                .collect::<Vec<_>>();

            table_inst.set_elems(0, &elems);

            let table = match name {
                "table" => table_inst,
                _ => todo!("resolve_global：{}", name),
            };

            Some(Rc::new(RefCell::new(table)))
        }

        fn resolve_mem(&self, name: &str) -> Option<wasm::execution::inst::RMemInst> {
            let memory = match name {
                "memory" => MemInst::new(Limits { min: 1, max: Some(2) }),
                _ => todo!("resolve_mem {:?}", name),
            };

            Some(Rc::new(RefCell::new(memory)))
        }

        fn resolve_global(&self, name: &str) -> Option<RGlobalInst> {
            let global_inst = match name {
                "global_i32" => GlobalInst::new(
                    GlobalType {
                        val_type: ValType::I32,
                        mut_: Mut::Const,
                    },
                    ValInst::I32(666),
                ),
                "global_i64" => GlobalInst::new(
                    GlobalType {
                        val_type: ValType::I64,
                        mut_: Mut::Const,
                    },
                    ValInst::I64(666),
                ),
                "global_f32" => GlobalInst::new(
                    GlobalType {
                        val_type: ValType::F32,
                        mut_: Mut::Const,
                    },
                    ValInst::F32(0.0),
                ),
                "global_f64" => GlobalInst::new(
                    GlobalType {
                        val_type: ValType::F64,
                        mut_: Mut::Const,
                    },
                    ValInst::F64(0.0),
                ),
                _ => todo!("resolve_global：{}", name),
            };

            Some(Rc::new(RefCell::new(global_inst)))
        }

        fn call_by_name(&mut self, name: &str, args: ValInsts) -> ValInsts {
            match name {
                "print" => SpecTestModule::print(args),
                "print_i32" => SpecTestModule::print_i32(args),
                "print_i64" => SpecTestModule::print_i64(args),
                "print_f32" => SpecTestModule::print_f32(args),
                "print_f64" => SpecTestModule::print_f64(args),
                "print_i32_f32" => SpecTestModule::print_i32_f32(args),
                "print_f64_f64" => SpecTestModule::print_f64_f64(args),
                "ret_11" => SpecTestModule::ret_11(args),
                "ret_22" => SpecTestModule::ret_22(args),
                _ => unimplemented!("SpecTestModule call_by_name：{}", name),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fs;
    use std::rc::Rc;

    use paste::paste;
    use wasm::execution::importer::MImporter;

    use super::models::WabtJson;
    use crate::models::{CommandType, Module};
    use crate::run::{Execute, LATEST_NAME};
    use crate::spec_test::SpecTestModule;

    macro_rules! load {
        ($name:ident) => {
            paste! {
                #[test]
                fn [<test_ $name>]()  {
                    let (root, wabt_json) = load_wabt_json(stringify!($name));

                    run_test(root, wabt_json);
                }
            }
        };
    }

    pub fn load_wabt_json(name: &str) -> (String, WabtJson) {
        let file = format!("./tests/output/{name}/{name}.json");
        let json_str = fs::read_to_string(file).expect("json 读取失败");

        let deserialized: Result<WabtJson, _> = serde_json::from_str(&json_str);
        let root = format!("./tests/output/{name}/");

        (root, deserialized.expect("json 解析失败"))
    }

    fn run_test(root: String, wabt_json: WabtJson) {
        let mut maps: MImporter = HashMap::new();
        let spec_test_module = Rc::new(RefCell::new(SpecTestModule));

        maps.insert("sys".to_string(), spec_test_module.clone());
        maps.insert("spectest".to_string(), spec_test_module);

        for command in wabt_json.commands {
            let maps_copy = maps.clone();

            match command.type_ {
                CommandType::Module(module) => {
                    if module.filename == "linking.20.wasm" {
                        println!("");
                    }

                    let (vm_, name) = Module::create(&root, &module.filename, module.name, maps_copy);
                    let vm = vm_.expect("合法模块是不可能实例化失败的");
                    let vm_rc = Rc::new(RefCell::new(vm));

                    if name != LATEST_NAME {
                        maps.insert(LATEST_NAME.to_string(), vm_rc.clone());
                    }

                    maps.insert(name.clone(), vm_rc);
                }
                CommandType::Action { action } => {
                    action.run(maps_copy);
                }
                CommandType::AssertReturn(action) => action.run(maps_copy),
                // CommandType::AssertExhaustion(exhaustion) => todo!(),
                // CommandType::AssertTrap(trap) => todo!(),
                // CommandType::AssertInvalid(module) => todo!(),
                CommandType::AssertMalformed(assert_module) => {
                    if assert_module.is_binary_module() {
                        let module = assert_module.create(&root);

                        assert!(module.is_err(), "不可能解析成功");

                        if let Err(err) = module {
                            println!("err：{:#}", err);
                        }
                    }
                }
                CommandType::AssertUninstantiable(module) => {
                    // 需要先处理 unreachable
                    if module.filename == "linking.39.wasm" || module.filename == "start.8.wasm" {
                        return;
                    }

                    if module.is_binary_module() {
                        let name = Some("uninstantiable".to_string());
                        let (vm, _) = Module::create(&root, &module.filename, name, maps_copy);

                        assert!(vm.is_err(), "不可能实例化成功");

                        if let Err(err) = vm {
                            println!("err：{:#}\n", err);
                        }
                    }
                }
                // CommandType::AssertUnlinkable(module) => {
                //     let name = Some("uninstantiable".to_string());
                //     let (vm, name) = Module::create(&root, &module.filename, name, maps_copy);
                // },
                CommandType::Register(register) => {
                    let vm = Module::get(register.name.clone(), maps_copy);

                    maps.insert(register.as_, vm);
                }
                _ => (),
            }
        }
    }

    load!(address);
    load!(align);
    load!(binary_leb128);
    load!(binary);
    load!(block);
    load!(br);
    load!(br_if);
    load!(br_table);
    load!(bulk);
    load!(call);
    load!(call_indirect);
    // load!(comments);
    load!(const);
    load!(conversions);
    load!(custom);
    load!(data);
    load!(elem);
    load!(endianness);
    load!(exports);
    load!(f32);
    load!(f32_bitwise);
    load!(f32_cmp);
    load!(f64);
    // load!(if);
    load!(f64_bitwise);
    load!(f64_cmp);
    load!(fac);
    load!(float_exprs);
    load!(float_literals);
    load!(float_memory);
    load!(float_misc);
    load!(forward);
    load!(func);
    load!(func_ptrs);
    load!(global);
    load!(i32);
    load!(i64);
    load!(imports);
    load!(inline_module);
    load!(int_exprs);
    load!(int_literals);
    load!(labels);
    load!(left_to_right);
    load!(linking);
    load!(load);
    load!(local_get);
    load!(local_set);
    load!(local_tee);
    load!(loop);
    load!(memory);
    load!(memory_copy);
    load!(memory_fill);
    load!(memory_grow);
    load!(memory_init);
    load!(memory_redundancy);
    load!(memory_size);
    load!(memory_trap);
    load!(names);
    load!(nop);
    load!(obsolete_keywords);
    load!(ref_func);
    load!(ref_is_null);
    load!(ref_null);
    load!(return);
    load!(select);
    load!(simd_address);
    load!(simd_align);
    load!(simd_bitwise);
    load!(simd_bit_shift);
    load!(simd_boolean);
    load!(simd_const);
    load!(simd_conversions);
    load!(simd_f32x4);
    load!(simd_f32x4_arith);
    load!(simd_f32x4_cmp);
    load!(simd_f32x4_pmin_pmax);
    load!(simd_f32x4_rounding);
    load!(simd_f64x2);
    load!(simd_f64x2_arith);
    load!(simd_f64x2_cmp);
    load!(simd_f64x2_pmin_pmax);
    load!(simd_f64x2_rounding);
    load!(simd_i16x8_arith);
    load!(simd_i16x8_arith2);
    load!(simd_i16x8_cmp);
    load!(simd_i16x8_extadd_pairwise_i8x16);
    load!(simd_i16x8_extmul_i8x16);
    load!(simd_i16x8_q15mulr_sat_s);
    load!(simd_i16x8_sat_arith);
    load!(simd_i32x4_arith);
    load!(simd_i32x4_arith2);
    load!(simd_i32x4_cmp);
    load!(simd_i32x4_dot_i16x8);
    load!(simd_i32x4_extadd_pairwise_i16x8);
    load!(simd_i32x4_extmul_i16x8);
    load!(simd_i32x4_trunc_sat_f32x4);
    load!(simd_i32x4_trunc_sat_f64x2);
    load!(simd_i64x2_arith);
    load!(simd_i64x2_arith2);
    load!(simd_i64x2_cmp);
    load!(simd_i64x2_extmul_i32x4);
    load!(simd_i8x16_arith);
    load!(simd_i8x16_arith2);
    load!(simd_i8x16_cmp);
    load!(simd_i8x16_sat_arith);
    load!(simd_int_to_int_extend);
    load!(simd_lane);
    load!(simd_linking);
    load!(simd_load);
    load!(simd_load16_lane);
    load!(simd_load32_lane);
    load!(simd_load64_lane);
    load!(simd_load8_lane);
    load!(simd_load_extend);
    load!(simd_load_splat);
    load!(simd_load_zero);
    load!(simd_splat);
    load!(simd_store);
    load!(simd_store16_lane);
    load!(simd_store32_lane);
    load!(simd_store64_lane);
    load!(simd_store8_lane);
    load!(skip_stack_guard_page);
    load!(stack);
    load!(start);
    load!(store);
    load!(switch);
    load!(table_sub);
    load!(table);
    load!(table_copy);
    load!(table_fill);
    load!(table_get);
    load!(table_grow);
    load!(table_init);
    load!(table_set);
    load!(table_size);
    load!(token);
    load!(traps);
    load!(type);
    load!(unreachable);
    load!(unreached_invalid);
    load!(unreached_valid);
    load!(unwind);
    load!(utf8_custom_section_id);
    load!(utf8_import_field);
    load!(utf8_import_module);
    load!(utf8_invalid_encoding);
}
