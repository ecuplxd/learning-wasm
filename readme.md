一个学习性质的 wasm 虚拟机，符合当前 2.0 规范的语义，对于所有合法的 wasm 模块及输入，能通过测试。

运行测试：

```shell
git submodule init
git submodule update
cd tests
chmod +x wast2json
# 使用官方 wabt 下的 wast2json 工具，if 和 comments 会解析失败，疑似 wabt 的 bug
python walk.py
# linux 系统可能需要安装 openssl
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

[wabt 下载](https://github.com/WebAssembly/wabt/releases)

测试结果：

```shell
12050@learning MINGW64 ~/Documents/code/rust/wasm (main)
$ cargo tarpaulin --out Html
Jan 11 13:58:59.697  INFO cargo_tarpaulin::config: Creating config
Jan 11 13:58:59.829  INFO cargo_tarpaulin: Running Tarpaulin
Jan 11 13:58:59.829  INFO cargo_tarpaulin: Building project
Jan 11 13:58:59.829  INFO cargo_tarpaulin::cargo: Cleaning project
   Compiling proc-macro2 v1.0.75
   Compiling unicode-ident v1.0.12
   Compiling cfg-if v1.0.0
   Compiling thiserror v1.0.56
   Compiling ppv-lite86 v0.2.17
   Compiling serde v1.0.194
   Compiling serde_json v1.0.108
   Compiling paste v1.0.14
   Compiling itoa v1.0.10
   Compiling ryu v1.0.16
   Compiling getrandom v0.2.11
   Compiling rand_core v0.6.4
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.5
   Compiling quote v1.0.35
   Compiling syn v2.0.47
   Compiling thiserror-impl v1.0.56
   Compiling serde_derive v1.0.194
   Compiling wasm v0.1.0 (C:\Users\12050\Documents\code\rust\wasm)
    Finished test [unoptimized + debuginfo] target(s) in 25.51s
Jan 11 14:00:04.117  INFO cargo_tarpaulin::process_handling: running C:\Users\12050\Documents\code\rust\wasm\target\debug\deps\wasm-dd7ce5dfb30ef176.exe
Jan 11 14:00:04.117  INFO cargo_tarpaulin::process_handling: Setting LLVM_PROFILE_FILE

running 4 tests
test binary::leb128::test::test_decode_int ... ok
test binary::leb128::test::test_decode_uint ... ok
test binary::leb128::test::test_encode_signed ... ok
test binary::leb128::test::test_encode_unsigned ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

Jan 11 14:00:04.133  INFO cargo_tarpaulin::statemachine::instrumented: For binary: target\debug\deps\wasm-dd7ce5dfb30ef176.exe
Jan 11 14:00:04.133  INFO cargo_tarpaulin::statemachine::instrumented: Generated: target\tarpaulin\profraws\wasm-dd7ce5dfb30ef176.exe_10623697279723036792_0-7512.profraw
Jan 11 14:00:04.134  INFO cargo_tarpaulin::statemachine::instrumented: Merging coverage reports
Jan 11 14:00:04.136  INFO cargo_tarpaulin::statemachine::instrumented: Mapping coverage data to source
Jan 11 14:00:04.434  INFO cargo_tarpaulin::process_handling: running C:\Users\12050\Documents\code\rust\wasm\target\debug\deps\wasm-406cad1d36eac3f6.exe
Jan 11 14:00:04.434  INFO cargo_tarpaulin::process_handling: Setting LLVM_PROFILE_FILE

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

Jan 11 14:00:04.448  INFO cargo_tarpaulin::statemachine::instrumented: For binary: target\debug\deps\wasm-406cad1d36eac3f6.exe
Jan 11 14:00:04.448  INFO cargo_tarpaulin::statemachine::instrumented: Generated: target\tarpaulin\profraws\wasm-406cad1d36eac3f6.exe_18405242399302775433_0-4412.profraw
Jan 11 14:00:04.448  INFO cargo_tarpaulin::statemachine::instrumented: Merging coverage reports
Jan 11 14:00:04.449  WARN cargo_tarpaulin::statemachine::instrumented: profraw file has no records after merging. If this is unexpected it may be caused by a panic or signal used in a test that prevented the LLVM instrumentation runtime from serialising results
Jan 11 14:00:04.449  INFO cargo_tarpaulin::process_handling: running C:\Users\12050\Documents\code\rust\wasm\target\debug\deps\spec-5c6ae22aa31575e3.exe
Jan 11 14:00:04.449  INFO cargo_tarpaulin::process_handling: Setting LLVM_PROFILE_FILE

running 145 tests
test test::test_address ... ok
test test::test_align ... ok
test test::test_binary ... ok
test test::test_binary_leb128 ... ok
test test::test_block ... ok
test test::test_br ... ok
test test::test_br_if ... ok
test test::test_br_table ... ok
test test::test_bulk ... ok
test test::test_call ... ok
test test::test_call_indirect ... ok
test test::test_const ... ok
test test::test_conversions ... ok
test test::test_custom ... ok
test test::test_data ... ok
test test::test_elem ... ok
test test::test_endianness ... ok
test test::test_exports ... ok
test test::test_f32 ... ok
test test::test_f32_bitwise ... ok
test test::test_f32_cmp ... ok
test test::test_f64 ... ok
test test::test_f64_bitwise ... ok
test test::test_f64_cmp ... ok
test test::test_fac ... ok
test test::test_float_exprs ... ok
test test::test_float_literals ... ok
test test::test_float_memory ... ok
test test::test_float_misc ... ok
test test::test_forward ... ok
test test::test_func ... ok
test test::test_func_ptrs ... ok
test test::test_global ... ok
test test::test_i32 ... ok
test test::test_i64 ... ok
test test::test_imports ... ok
test test::test_inline_module ... ok
test test::test_int_exprs ... ok
test test::test_int_literals ... ok
test test::test_labels ... ok
test test::test_left_to_right ... ok
test test::test_linking ... ok
test test::test_load ... ok
test test::test_local_get ... ok
test test::test_local_set ... ok
test test::test_local_tee ... ok
test test::test_loop ... ok
test test::test_memory ... ok
test test::test_memory_copy ... ok
test test::test_memory_fill ... ok
test test::test_memory_grow ... ok
test test::test_memory_init ... ok
test test::test_memory_redundancy ... ok
test test::test_memory_size ... ok
test test::test_memory_trap ... ok
test test::test_names ... ok
test test::test_nop ... ok
test test::test_obsolete_keywords ... ok
test test::test_ref_func ... ok
test test::test_ref_is_null ... ok
test test::test_ref_null ... ok
test test::test_return ... ok
test test::test_select ... ok
test test::test_simd_address ... ok
test test::test_simd_align ... ok
test test::test_simd_bit_shift ... ok
test test::test_simd_bitwise ... ok
test test::test_simd_boolean ... ok
test test::test_simd_const ... ok
test test::test_simd_conversions ... ok
test test::test_simd_f32x4 ... ok
test test::test_simd_f32x4_arith ... ok
test test::test_simd_f32x4_cmp ... ok
test test::test_simd_f32x4_pmin_pmax ... ok
test test::test_simd_f32x4_rounding ... ok
test test::test_simd_f64x2 ... ok
test test::test_simd_f64x2_arith ... ok
test test::test_simd_f64x2_cmp ... ok
test test::test_simd_f64x2_pmin_pmax ... ok
test test::test_simd_f64x2_rounding ... ok
test test::test_simd_i16x8_arith ... ok
test test::test_simd_i16x8_arith2 ... ok
test test::test_simd_i16x8_cmp ... ok
test test::test_simd_i16x8_extadd_pairwise_i8x16 ... ok
test test::test_simd_i16x8_extmul_i8x16 ... ok
test test::test_simd_i16x8_q15mulr_sat_s ... ok
test test::test_simd_i16x8_sat_arith ... ok
test test::test_simd_i32x4_arith ... ok
test test::test_simd_i32x4_arith2 ... ok
test test::test_simd_i32x4_cmp ... ok
test test::test_simd_i32x4_dot_i16x8 ... ok
test test::test_simd_i32x4_extadd_pairwise_i16x8 ... ok
test test::test_simd_i32x4_extmul_i16x8 ... ok
test test::test_simd_i32x4_trunc_sat_f32x4 ... ok
test test::test_simd_i32x4_trunc_sat_f64x2 ... ok
test test::test_simd_i64x2_arith ... ok
test test::test_simd_i64x2_arith2 ... ok
test test::test_simd_i64x2_cmp ... ok
test test::test_simd_i64x2_extmul_i32x4 ... ok
test test::test_simd_i8x16_arith ... ok
test test::test_simd_i8x16_arith2 ... ok
test test::test_simd_i8x16_cmp ... ok
test test::test_simd_i8x16_sat_arith ... ok
test test::test_simd_int_to_int_extend ... ok
test test::test_simd_lane ... ok
test test::test_simd_linking ... ok
test test::test_simd_load ... ok
test test::test_simd_load16_lane ... ok
test test::test_simd_load32_lane ... ok
test test::test_simd_load64_lane ... ok
test test::test_simd_load8_lane ... ok
test test::test_simd_load_extend ... ok
test test::test_simd_load_splat ... ok
test test::test_simd_load_zero ... ok
test test::test_simd_splat ... ok
test test::test_simd_store ... ok
test test::test_simd_store16_lane ... ok
test test::test_simd_store32_lane ... ok
test test::test_simd_store64_lane ... ok
test test::test_simd_store8_lane ... ok
test test::test_skip_stack_guard_page ... ok
test test::test_stack ... ok
test test::test_start ... ok
test test::test_store ... ok
test test::test_switch ... ok
test test::test_table ... ok
test test::test_table_copy ... ok
test test::test_table_fill ... ok
test test::test_table_get ... ok
test test::test_table_grow ... ok
test test::test_table_init ... ok
test test::test_table_set ... ok
test test::test_table_size ... ok
test test::test_table_sub ... ok
test test::test_token ... ok
test test::test_traps ... ok
test test::test_type ... ok
test test::test_unreachable ... ok
test test::test_unreached_invalid ... ok
test test::test_unreached_valid ... ok
test test::test_unwind ... ok
test test::test_utf8_custom_section_id ... ok
test test::test_utf8_import_field ... ok
test test::test_utf8_import_module ... ok
test test::test_utf8_invalid_encoding ... ok

test result: ok. 145 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.04s

Jan 11 14:00:09.502  INFO cargo_tarpaulin::statemachine::instrumented: For binary: target\debug\deps\spec-5c6ae22aa31575e3.exe
Jan 11 14:00:09.503  INFO cargo_tarpaulin::statemachine::instrumented: Generated: target\tarpaulin\profraws\spec-5c6ae22aa31575e3.exe_16417738510065967105_0-21536.profraw
Jan 11 14:00:09.503  INFO cargo_tarpaulin::statemachine::instrumented: Merging coverage reports
Jan 11 14:00:09.512  INFO cargo_tarpaulin::statemachine::instrumented: Mapping coverage data to source
Jan 11 14:00:10.155  INFO cargo_tarpaulin::report: Coverage Results:
|| Tested/Total Lines:
|| src\binary\decode.rs: 591/615
|| src\binary\encode.rs: 363/397
|| src\binary\instruction.rs: 11/12
|| src\binary\leb128.rs: 57/58
|| src\binary\module.rs: 74/91
|| src\binary\reader.rs: 52/63
|| src\binary\section.rs: 25/31
|| src\binary\types.rs: 51/59
|| src\binary\validate.rs: 0/140
|| src\execution\importer.rs: 2/12
|| src\execution\inst\element.rs: 3/3
|| src\execution\inst\function.rs: 15/27
|| src\execution\inst\global.rs: 12/14
|| src\execution\inst\memory.rs: 82/88
|| src\execution\inst\table.rs: 40/47
|| src\execution\instr\control.rs: 105/117
|| src\execution\instr\exec.rs: 438/440
|| src\execution\instr\memory.rs: 127/127
|| src\execution\instr\numeric.rs: 484/484
|| src\execution\instr\parametric.rs: 11/11
|| src\execution\instr\reference.rs: 9/9
|| src\execution\instr\table.rs: 47/47
|| src\execution\instr\trunc_sat.rs: 69/69
|| src\execution\instr\variable.rs: 22/22
|| src\execution\instr\vector.rs: 1501/1505
|| src\execution\mod.rs: 5/5
|| src\execution\stack\frame.rs: 14/15
|| src\execution\stack\operand.rs: 38/38
|| src\execution\types.rs: 106/136
|| src\execution\vm.rs: 187/211
|| src\main.rs: 0/1
||
92.79% coverage, 4541/4894 lines covered
```

# done

- 二进制模块编解码
- 虚拟机

# todo

- validate -> AssertInvalid
- AssertExhaustion
- wasi
- dump
- wat 的解析
