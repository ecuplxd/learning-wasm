use super::section::{Expr, LabelIdx};
use super::types::ValType;
use crate::execution::types::v128;

#[derive(Debug)]
pub struct MemoryArg {
    pub align: u32,
    pub offset: u32,
}

#[derive(Debug)]
pub struct Block {
    pub type_: BlockType,
    pub expr: Expr,
}

impl Block {
    pub fn new(type_: BlockType, expr: Expr) -> Self {
        Self { type_, expr }
    }
}

#[derive(Debug, Default)]
pub enum BlockType {
    #[default]
    I32,
    I64,
    F32,
    F64,
    V128,
    ExternRef,
    Empty,
    TypeIdx(i32),
}

impl From<i32> for BlockType {
    fn from(v: i32) -> Self {
        match v {
            -1 => Self::I32,
            -2 => Self::I64,
            -3 => Self::F32,
            -4 => Self::F64,
            -5 => Self::V128,
            -17 => Self::ExternRef,
            -64 => Self::Empty,
            value if value >= 0 => Self::TypeIdx(value),
            _ => panic!("无效的 BlockType：{}", v),
        }
    }
}

#[derive(Debug, Default)]
pub struct IfBlock {
    pub type_: BlockType,
    pub if_expr: Expr,
    pub else_expr: Expr,
}

#[derive(Debug)]
pub struct BrTableArg {
    pub labels: Vec<LabelIdx>,
    pub default: LabelIdx,
}

pub type LaneIdx = u8;
pub type Lane2 = [u8; 2];
pub type Lane4 = [u8; 4];
pub type Lane8 = [u8; 8];
pub type Lane16 = [u8; 16];

#[repr(u16)]
#[derive(Debug)]
pub enum Instruction {
    Unreachable = 0x00,                           // unreachable 0x00
    Nop = 0x01,                                   // nop 0x01
    Block(Block) = 0x02,                          // block 0x02
    Loop(Block) = 0x03,                           // loop 0x03
    If(IfBlock) = 0x04,                           // if 0x04
    Else = 0x05,                                  // else 0x05
    End = 0x0b,                                   // end 0x0B
    Br(LabelIdx) = 0x0c,                          // br 0x0C
    BrIf(LabelIdx) = 0x0d,                        // br_if 0x0D
    BrTable(BrTableArg) = 0x0e,                   // br_table 0x0E
    Return = 0x0f,                                // return 0x0F
    Call(u32) = 0x10,                             // call 0x10
    CallIndirect(u32, u32) = 0x11,                // call_indirect 0x11
    Drop = 0x1a,                                  // drop 0x1A
    Select = 0x1b,                                // select 0x1B
    Select2(u8, ValType) = 0x1c,                  // select 0x1C
    LocalGet(u32) = 0x20,                         // local_get 0x20
    LocalSet(u32) = 0x21,                         // local_set 0x21
    LocalTee(u32) = 0x22,                         // local_tee 0x22
    GlobalGet(u32) = 0x23,                        // global_get 0x23
    GlobalSet(u32) = 0x24,                        // global_set 0x24
    TableGet(u32) = 0x25,                         // table_get 0x25
    TableSet(u32) = 0x26,                         // table_set 0x26
    I32Load(MemoryArg) = 0x28,                    // i32_load 0x28
    I64Load(MemoryArg) = 0x29,                    // i64_load 0x29
    F32Load(MemoryArg) = 0x2a,                    // f32_load 0x2A
    F64Load(MemoryArg) = 0x2b,                    // f64_load 0x2B
    I32Load8S(MemoryArg) = 0x2c,                  // i32_load8_s 0x2C
    I32Load8U(MemoryArg) = 0x2d,                  // i32_load8_u 0x2D
    I32Load16S(MemoryArg) = 0x2e,                 // i32_load16_s 0x2E
    I32Load16U(MemoryArg) = 0x2f,                 // i32_load16_u 0x2F
    I64Load8S(MemoryArg) = 0x30,                  // i64_load8_s 0x30
    I64Load8U(MemoryArg) = 0x31,                  // i64_load8_u 0x31
    I64Load16S(MemoryArg) = 0x32,                 // i64_load16_s 0x32
    I64Load16U(MemoryArg) = 0x33,                 // i64_load16_u 0x33
    I64Load32S(MemoryArg) = 0x34,                 // i64_load32_s 0x34
    I64Load32U(MemoryArg) = 0x35,                 // i64_load32_u 0x35
    I32Store(MemoryArg) = 0x36,                   // i32_store 0x36
    I64Store(MemoryArg) = 0x37,                   // i64_store 0x37
    F32Store(MemoryArg) = 0x38,                   // f32_store 0x38
    F64Store(MemoryArg) = 0x39,                   // f64_store 0x39
    I32Store8(MemoryArg) = 0x3a,                  // i32_store8 0x3A
    I32Store16(MemoryArg) = 0x3b,                 // i32_store16 0x3B
    I64Store8(MemoryArg) = 0x3c,                  // i64_store8 0x3C
    I64Store16(MemoryArg) = 0x3d,                 // i64_store16 0x3D
    I64Store32(MemoryArg) = 0x3e,                 // i64_store32 0x3E
    MemorySize(u32) = 0x3f,                       // memory_size 0x3F
    MemoryGrow(u32) = 0x40,                       // memory_grow 0x40
    I32Const(i32) = 0x41,                         // i32_const 0x41
    I64Const(i64) = 0x42,                         // i64_const 0x42
    F32Const(f32) = 0x43,                         // f32_const 0x43
    F64Const(f64) = 0x44,                         // f64_const 0x44
    I32Eqz = 0x45,                                // i32_eqz 0x45
    I32Eq = 0x46,                                 // i32_eq 0x46
    I32Ne = 0x47,                                 // i32_ne 0x47
    I32LtS = 0x48,                                // i32_lt_s 0x48
    I32LtU = 0x49,                                // i32_lt_u 0x49
    I32GtS = 0x4a,                                // i32_gt_s 0x4A
    I32GtU = 0x4b,                                // i32_gt_u 0x4B
    I32LeS = 0x4c,                                // i32_le_s 0x4C
    I32LeU = 0x4d,                                // i32_le_u 0x4D
    I32GeS = 0x4e,                                // i32_ge_s 0x4E
    I32GeU = 0x4f,                                // i32_ge_u 0x4F
    I64Eqz = 0x50,                                // i64_eqz 0x50
    I64Eq = 0x51,                                 // i64_eq 0x51
    I64Ne = 0x52,                                 // i64_ne 0x52
    I64LtS = 0x53,                                // i64_lt_s 0x53
    I64LtU = 0x54,                                // i64_lt_u 0x54
    I64GtS = 0x55,                                // i64_gt_s 0x55
    I64GtU = 0x56,                                // i64_gt_u 0x56
    I64LeS = 0x57,                                // i64_le_s 0x57
    I64LeU = 0x58,                                // i64_le_u 0x58
    I64GeS = 0x59,                                // i64_ge_s 0x59
    I64GeU = 0x5a,                                // i64_ge_u 0x5A
    F32Eq = 0x5b,                                 // f32_eq 0x5B
    F32Ne = 0x5c,                                 // f32_ne 0x5C
    F32Lt = 0x5d,                                 // f32_lt 0x5D
    F32Gt = 0x5e,                                 // f32_gt 0x5E
    F32Le = 0x5f,                                 // f32_le 0x5F
    F32Ge = 0x60,                                 // f32_ge 0x60
    F64Eq = 0x61,                                 // f64_eq 0x61
    F64Ne = 0x62,                                 // f64_ne 0x62
    F64Lt = 0x63,                                 // f64_lt 0x63
    F64Gt = 0x64,                                 // f64_gt 0x64
    F64Le = 0x65,                                 // f64_le 0x65
    F64Ge = 0x66,                                 // f64_ge 0x66
    I32Clz = 0x67,                                // i32_clz 0x67
    I32Ctz = 0x68,                                // i32_ctz 0x68
    I32Popcnt = 0x69,                             // i32_popcnt 0x69
    I32Add = 0x6a,                                // i32_add 0x6A
    I32Sub = 0x6b,                                // i32_sub 0x6B
    I32Mul = 0x6c,                                // i32_mul 0x6C
    I32DivS = 0x6d,                               // i32_div_s 0x6D
    I32DivU = 0x6e,                               // i32_div_u 0x6E
    I32RemS = 0x6f,                               // i32_rem_s 0x6F
    I32RemU = 0x70,                               // i32_rem_u 0x70
    I32And = 0x71,                                // i32_and 0x71
    I32Or = 0x72,                                 // i32_or 0x72
    I32Xor = 0x73,                                // i32_xor 0x73
    I32Shl = 0x74,                                // i32_shl 0x74
    I32ShrS = 0x75,                               // i32_shr_s 0x75
    I32ShrU = 0x76,                               // i32_shr_u 0x76
    I32Rotl = 0x77,                               // i32_rotl 0x77
    I32Rotr = 0x78,                               // i32_rotr 0x78
    I64Clz = 0x79,                                // i64_clz 0x79
    I64Ctz = 0x7a,                                // i64_ctz 0x7A
    I64Popcnt = 0x7b,                             // i64_popcnt 0x7B
    I64Add = 0x7c,                                // i64_add 0x7C
    I64Sub = 0x7d,                                // i64_sub 0x7D
    I64Mul = 0x7e,                                // i64_mul 0x7E
    I64DivS = 0x7f,                               // i64_div_s 0x7F
    I64DivU = 0x80,                               // i64_div_u 0x80
    I64RemS = 0x81,                               // i64_rem_s 0x81
    I64RemU = 0x82,                               // i64_rem_u 0x82
    I64And = 0x83,                                // i64_and 0x83
    I64Or = 0x84,                                 // i64_or 0x84
    I64Xor = 0x85,                                // i64_xor 0x85
    I64Shl = 0x86,                                // i64_shl 0x86
    I64ShrS = 0x87,                               // i64_shr_s 0x87
    I64ShrU = 0x88,                               // i64_shr_u 0x88
    I64Rotl = 0x89,                               // i64_rotl 0x89
    I64Rotr = 0x8a,                               // i64_rotr 0x8A
    F32Abs = 0x8b,                                // f32_abs 0x8B
    F32Neg = 0x8c,                                // f32_neg 0x8C
    F32Ceil = 0x8d,                               // f32_ceil 0x8D
    F32Floor = 0x8e,                              // f32_floor 0x8E
    F32Trunc = 0x8f,                              // f32_trunc 0x8F
    F32Nearest = 0x90,                            // f32_nearest 0x90
    F32Sqrt = 0x91,                               // f32_sqrt 0x91
    F32Add = 0x92,                                // f32_add 0x92
    F32Sub = 0x93,                                // f32_sub 0x93
    F32Mul = 0x94,                                // f32_mul 0x94
    F32Div = 0x95,                                // f32_div 0x95
    F32Min = 0x96,                                // f32_min 0x96
    F32Max = 0x97,                                // f32_max 0x97
    F32Copysign = 0x98,                           // f32_copysign 0x98
    F64Abs = 0x99,                                // f64_abs 0x99
    F64Neg = 0x9a,                                // f64_neg 0x9A
    F64Ceil = 0x9b,                               // f64_ceil 0x9B
    F64Floor = 0x9c,                              // f64_floor 0x9C
    F64Trunc = 0x9d,                              // f64_trunc 0x9D
    F64Nearest = 0x9e,                            // f64_nearest 0x9E
    F64Sqrt = 0x9f,                               // f64_sqrt 0x9F
    F64Add = 0xa0,                                // f64_add 0xA0
    F64Sub = 0xa1,                                // f64_sub 0xA1
    F64Mul = 0xa2,                                // f64_mul 0xA2
    F64Div = 0xa3,                                // f64_div 0xA3
    F64Min = 0xa4,                                // f64_min 0xA4
    F64Max = 0xa5,                                // f64_max 0xA5
    F64Copysign = 0xa6,                           // f64_copysign 0xA6
    I32WrapI64 = 0xa7,                            // i32_wrap_i64 0xA7
    I32TruncF32S = 0xa8,                          // i32_trunc_f32_s 0xA8
    I32TruncF32U = 0xa9,                          // i32_trunc_f32_u 0xA9
    I32TruncF64S = 0xaa,                          // i32_trunc_f64_s 0xAA
    I32TruncF64U = 0xab,                          // i32_trunc_f64_u 0xAB
    I64ExtendI32S = 0xac,                         // i64_extend_i32_s 0xAC
    I64ExtendI32U = 0xad,                         // i64_extend_i32_u 0xAD
    I64TruncF32S = 0xae,                          // i64_trunc_f32_s 0xAE
    I64TruncF32U = 0xaf,                          // i64_trunc_f32_u 0xAF
    I64TruncF64S = 0xb0,                          // i64_trunc_f64_s 0xB0
    I64TruncF64U = 0xb1,                          // i64_trunc_f64_u 0xB1
    F32ConvertI32S = 0xb2,                        // f32_convert_i32_s 0xB2
    F32ConvertI32U = 0xb3,                        // f32_convert_i32_u 0xB3
    F32ConvertI64S = 0xb4,                        // f32_convert_i64_s 0xB4
    F32ConvertI64U = 0xb5,                        // f32_convert_i64_u 0xB5
    F32DemoteF64 = 0xb6,                          // f32_demote_f64 0xB6
    F64ConvertI32S = 0xb7,                        // f64_convert_i32_s 0xB7
    F64ConvertI32U = 0xb8,                        // f64_convert_i32_u 0xB8
    F64ConvertI64S = 0xb9,                        // f64_convert_i64_s 0xB9
    F64ConvertI64U = 0xba,                        // f64_convert_i64_u 0xBA
    F64PromoteF32 = 0xbb,                         // f64_promote_f32 0xBB
    I32ReinterpretF32 = 0xbc,                     // i32_reinterpret_f32 0xBC
    I64ReinterpretF64 = 0xbd,                     // i64_reinterpret_f64 0xBD
    F32ReinterpretI32 = 0xbe,                     // f32_reinterpret_i32 0xBE
    F64ReinterpretI64 = 0xbf,                     // f64_reinterpret_i64 0xBF
    I32Extend8S = 0xc0,                           // i32_extend8_s 0xC0
    I32Extend16S = 0xc1,                          // i32_extend16_s 0xC1
    I64Extend8S = 0xc2,                           // i64_extend8_s 0xC2
    I64Extend16S = 0xc3,                          // i64_extend16_s 0xC3
    I64Extend32S = 0xc4,                          // i64_extend32_s 0xC4
    RefNull(u64) = 0xd0,                          // ref_null 0xD0
    RefIsNull = 0xd1,                             // ref_is_null 0xD1
    RefFunc(u32) = 0xd2,                          // ref_func 0xD2
    I32TruncSatF32S = 0xfc00,                     // i32_trunc_sat_f32_s 0xFC 0x00
    I32TruncSatF32U = 0xfc01,                     // i32_trunc_sat_f32_u 0xFC 0x01
    I32TruncSatF64S = 0xfc02,                     // i32_trunc_sat_f64_s 0xFC 0x02
    I32TruncSatF64U = 0xfc03,                     // i32_trunc_sat_f64_u 0xFC 0x03
    I64TruncSatF32S = 0xfc04,                     // i64_trunc_sat_f32_s 0xFC 0x04
    I64TruncSatF32U = 0xfc05,                     // i64_trunc_sat_f32_u 0xFC 0x05
    I64TruncSatF64S = 0xfc06,                     // i64_trunc_sat_f64_s 0xFC 0x06
    I64TruncSatF64U = 0xfc07,                     // i64_trunc_sat_f64_u 0xFC 0x07
    MemoryInit(u32, u32) = 0xfc08,                // memory_init 0xFC 0x08
    DataDrop(u32) = 0xfc09,                       // data_drop 0xFC 0x09
    MemoryCopy(u32, u32) = 0xfc0a,                // memory_copy 0xFC 0x0A
    MemoryFill(u32) = 0xfc0b,                     // memory_fill 0xFC 0x0B
    TableInit(u32, u32) = 0xfc0c,                 // table_init 0xFC 0x0C
    ElemDrop(u32) = 0xfc0d,                       // elem_drop 0xFC 0x0D
    TableCopy(u32, u32) = 0xfc0e,                 // table_copy 0xFC 0x0E
    TableGrow(u32) = 0xfc0f,                      // table_grow 0xFC 0x0F
    TableSize(u32) = 0xfc10,                      // table_size 0xFC 0x10
    TableFill(u32) = 0xfc11,                      // table_fill 0xFC 0x11
    V128Load(MemoryArg) = 0xfd00,                 // v128_load 0xFD 0x00
    V128Load8x8S(MemoryArg) = 0xfd01,             // v128_load8x8_s 0xFD 0x01
    V128Load8x8U(MemoryArg) = 0xfd02,             // v128_load8x8_u 0xFD 0x02
    V128Load16x4S(MemoryArg) = 0xfd03,            // v128_load16x4_s 0xFD 0x03
    V128Load16x4U(MemoryArg) = 0xfd04,            // v128_load16x4_u 0xFD 0x04
    V128Load32x2S(MemoryArg) = 0xfd05,            // v128_load32x2_s 0xFD 0x05
    V128Load32x2U(MemoryArg) = 0xfd06,            // v128_load32x2_u 0xFD 0x06
    V128Load8Splat(MemoryArg) = 0xfd07,           // v128_load8_splat 0xFD 0x07
    V128Load16Splat(MemoryArg) = 0xfd08,          // v128_load16_splat 0xFD 0x08
    V128Load32Splat(MemoryArg) = 0xfd09,          // v128_load32_splat 0xFD 0x09
    V128Load64Splat(MemoryArg) = 0xfd0a,          // v128_load64_splat 0xFD 0x0A
    V128Store(MemoryArg) = 0xfd0b,                // v128_store 0xFD 0x0B
    V128Const(v128) = 0xfd0c,                     // v128_const 0xFD 0x0C
    I8x16Shuffle(Lane16) = 0xfd0d,                // i8x16_shuffle 0xFD 0x0D
    I8x16Swizzle = 0xfd0e,                        // i8x16_swizzle 0xFD 0x0E
    I8x16Splat = 0xfd0f,                          // i8x16_splat 0xFD 0x0F
    I16x8Splat = 0xfd10,                          // i16x8_splat 0xFD 0x10
    I32x4Splat = 0xfd11,                          // i32x4_splat 0xFD 0x11
    I64x2Splat = 0xfd12,                          // i64x2_splat 0xFD 0x12
    F32x4Splat = 0xfd13,                          // f32x4_splat 0xFD 0x13
    F64x2Splat = 0xfd14,                          // f64x2_splat 0xFD 0x14
    I8x16ExtractLaneS(LaneIdx) = 0xfd15,          // i8x16_extract_lane_s 0xFD 0x15
    I8x16ExtractLaneU(LaneIdx) = 0xfd16,          // i8x16_extract_lane_u 0xFD 0x16
    I8x16ReplaceLane(LaneIdx) = 0xfd17,           // i8x16_replace_lane 0xFD 0x17
    I16x8ExtractLaneS(LaneIdx) = 0xfd18,          // i16x8_extract_lane_s 0xFD 0x18
    I16x8ExtractLaneU(LaneIdx) = 0xfd19,          // i16x8_extract_lane_u 0xFD 0x19
    I16x8ReplaceLane(LaneIdx) = 0xfd1a,           // i16x8_replace_lane 0xFD 0x1A
    I32x4ExtractLane(LaneIdx) = 0xfd1b,           // i32x4_extract_lane 0xFD 0x1B
    I32x4ReplaceLane(LaneIdx) = 0xfd1c,           // i32x4_replace_lane 0xFD 0x1C
    I64x2ExtractLane(LaneIdx) = 0xfd1d,           // i64x2_extract_lane 0xFD 0x1D
    I64x2ReplaceLane(LaneIdx) = 0xfd1e,           // i64x2_replace_lane 0xFD 0x1E
    F32x4ExtractLane(LaneIdx) = 0xfd1f,           // f32x4_extract_lane 0xFD 0x1F
    F32x4ReplaceLane(LaneIdx) = 0xfd20,           // f32x4_replace_lane 0xFD 0x20
    F64x2ExtractLane(LaneIdx) = 0xfd21,           // f64x2_extract_lane 0xFD 0x21
    F64x2ReplaceLane(LaneIdx) = 0xfd22,           // f64x2_replace_lane 0xFD 0x22
    I8x16Eq = 0xfd23,                             // i8x16_eq 0xFD 0x23
    I8x16Ne = 0xfd24,                             // i8x16_ne 0xFD 0x24
    I8x16LtS = 0xfd25,                            // i8x16_lt_s 0xFD 0x25
    I8x16LtU = 0xfd26,                            // i8x16_lt_u 0xFD 0x26
    I8x16GtS = 0xfd27,                            // i8x16_gt_s 0xFD 0x27
    I8x16GtU = 0xfd28,                            // i8x16_gt_u 0xFD 0x28
    I8x16LeS = 0xfd29,                            // i8x16_le_s 0xFD 0x29
    I8x16LeU = 0xfd2a,                            // i8x16_le_u 0xFD 0x2A
    I8x16GeS = 0xfd2b,                            // i8x16_ge_s 0xFD 0x2B
    I8x16GeU = 0xfd2c,                            // i8x16_ge_u 0xFD 0x2C
    I16x8Eq = 0xfd2d,                             // i16x8_eq 0xFD 0x2D
    I16x8Ne = 0xfd2e,                             // i16x8_ne 0xFD 0x2E
    I16x8LtS = 0xfd2f,                            // i16x8_lt_s 0xFD 0x2F
    I16x8LtU = 0xfd30,                            // i16x8_lt_u 0xFD 0x30
    I16x8GtS = 0xfd31,                            // i16x8_gt_s 0xFD 0x31
    I16x8GtU = 0xfd32,                            // i16x8_gt_u 0xFD 0x32
    I16x8LeS = 0xfd33,                            // i16x8_le_s 0xFD 0x33
    I16x8LeU = 0xfd34,                            // i16x8_le_u 0xFD 0x34
    I16x8GeS = 0xfd35,                            // i16x8_ge_s 0xFD 0x35
    I16x8GeU = 0xfd36,                            // i16x8_ge_u 0xFD 0x36
    I32x4Eq = 0xfd37,                             // i32x4_eq 0xFD 0x37
    I32x4Ne = 0xfd38,                             // i32x4_ne 0xFD 0x38
    I32x4LtS = 0xfd39,                            // i32x4_lt_s 0xFD 0x39
    I32x4LtU = 0xfd3a,                            // i32x4_lt_u 0xFD 0x3A
    I32x4GtS = 0xfd3b,                            // i32x4_gt_s 0xFD 0x3B
    I32x4GtU = 0xfd3c,                            // i32x4_gt_u 0xFD 0x3C
    I32x4LeS = 0xfd3d,                            // i32x4_le_s 0xFD 0x3D
    I32x4LeU = 0xfd3e,                            // i32x4_le_u 0xFD 0x3E
    I32x4GeS = 0xfd3f,                            // i32x4_ge_s 0xFD 0x3F
    I32x4GeU = 0xfd40,                            // i32x4_ge_u 0xFD 0x40
    F32x4Eq = 0xfd41,                             // f32x4_eq 0xFD 0x41
    F32x4Ne = 0xfd42,                             // f32x4_ne 0xFD 0x42
    F32x4Lt = 0xfd43,                             // f32x4_lt 0xFD 0x43
    F32x4Gt = 0xfd44,                             // f32x4_gt 0xFD 0x44
    F32x4Le = 0xfd45,                             // f32x4_le 0xFD 0x45
    F32x4Ge = 0xfd46,                             // f32x4_ge 0xFD 0x46
    F64x2Eq = 0xfd47,                             // f64x2_eq 0xFD 0x47
    F64x2Ne = 0xfd48,                             // f64x2_ne 0xFD 0x48
    F64x2Lt = 0xfd49,                             // f64x2_lt 0xFD 0x49
    F64x2Gt = 0xfd4a,                             // f64x2_gt 0xFD 0x4A
    F64x2Le = 0xfd4b,                             // f64x2_le 0xFD 0x4B
    F64x2Ge = 0xfd4c,                             // f64x2_ge 0xFD 0x4C
    V128Not = 0xfd4d,                             // v128_not 0xFD 0x4D
    V128And = 0xfd4e,                             // v128_and 0xFD 0x4E
    V128Andnot = 0xfd4f,                          // v128_andnot 0xFD 0x4F
    V128Or = 0xfd50,                              // v128_or 0xFD 0x50
    V128Xor = 0xfd51,                             // v128_xor 0xFD 0x51
    V128Bitselect = 0xfd52,                       // v128_bitselect 0xFD 0x52
    V128AnyTrue = 0xfd53,                         // v128_any_true 0xFD 0x53
    V128Load8Lane(MemoryArg, LaneIdx) = 0xfd54,   // v128_load8_lane 0xFD 0x54
    V128Load16Lane(MemoryArg, LaneIdx) = 0xfd55,  // v128_load16_lane 0xFD 0x55
    V128Load32Lane(MemoryArg, LaneIdx) = 0xfd56,  // v128_load32_lane 0xFD 0x56
    V128Load64Lane(MemoryArg, LaneIdx) = 0xfd57,  // v128_load64_lane 0xFD 0x57
    V128Store8Lane(MemoryArg, LaneIdx) = 0xfd58,  // v128_store8_lane 0xFD 0x58
    V128Store16Lane(MemoryArg, LaneIdx) = 0xfd59, // v128_store16_lane 0xFD 0x59
    V128Store32Lane(MemoryArg, LaneIdx) = 0xfd5a, // v128_store32_lane 0xFD 0x5A
    V128Store64Lane(MemoryArg, LaneIdx) = 0xfd5b, // v128_store64_lane 0xFD 0x5B
    V128Load32Zero(MemoryArg) = 0xfd5c,           // v128_load32_zero 0xFD 0x5C
    V128Load64Zero(MemoryArg) = 0xfd5d,           // v128_load64_zero 0xFD 0x5D
    F32x4DemoteF64x2Zero = 0xfd5e,                // f32x4_demote_f64x2_zero 0xFD 0x5E
    F64x2PromoteLowF32x4 = 0xfd5f,                // f64x2_promote_low_f32x4 0xFD 0x5F
    I8x16Abs = 0xfd60,                            // i8x16_abs 0xFD 0x60
    I8x16Neg = 0xfd61,                            // i8x16_neg 0xFD 0x61
    I8x16Popcnt = 0xfd62,                         // i8x16_popcnt 0xFD 0x62
    I8x16AllTrue = 0xfd63,                        // i8x16_all_true 0xFD 0x63
    I8x16Bitmask = 0xfd64,                        // i8x16_bitmask 0xFD 0x64
    I8x16NarrowI16x8S = 0xfd65,                   // i8x16_narrow_i16x8_s 0xFD 0x65
    I8x16NarrowI16x8U = 0xfd66,                   // i8x16_narrow_i16x8_u 0xFD 0x66
    F32x4Ceil = 0xfd67,                           // f32x4_ceil 0xFD 0x67
    F32x4Floor = 0xfd68,                          // f32x4_floor 0xFD 0x68
    F32x4Trunc = 0xfd69,                          // f32x4_trunc 0xFD 0x69
    F32x4Nearest = 0xfd6a,                        // f32x4_nearest 0xFD 0x6A
    I8x16Shl = 0xfd6b,                            // i8x16_shl 0xFD 0x6B
    I8x16ShrS = 0xfd6c,                           // i8x16_shr_s 0xFD 0x6C
    I8x16ShrU = 0xfd6d,                           // i8x16_shr_u 0xFD 0x6D
    I8x16Add = 0xfd6e,                            // i8x16_add 0xFD 0x6E
    I8x16AddSatS = 0xfd6f,                        // i8x16_add_sat_s 0xFD 0x6F
    I8x16AddSatU = 0xfd70,                        // i8x16_add_sat_u 0xFD 0x70
    I8x16Sub = 0xfd71,                            // i8x16_sub 0xFD 0x71
    I8x16SubSatS = 0xfd72,                        // i8x16_sub_sat_s 0xFD 0x72
    I8x16SubSatU = 0xfd73,                        // i8x16_sub_sat_u 0xFD 0x73
    F64x2Ceil = 0xfd74,                           // f64x2_ceil 0xFD 0x74
    F64x2Floor = 0xfd75,                          // f64x2_floor 0xFD 0x75
    I8x16MinS = 0xfd76,                           // i8x16_min_s 0xFD 0x76
    I8x16MinU = 0xfd77,                           // i8x16_min_u 0xFD 0x77
    I8x16MaxS = 0xfd78,                           // i8x16_max_s 0xFD 0x78
    I8x16MaxU = 0xfd79,                           // i8x16_max_u 0xFD 0x79
    F64x2Trunc = 0xfd7a,                          // f64x2_trunc 0xFD 0x7A
    I8x16AvgrU = 0xfd7b,                          // i8x16_avgr_u 0xFD 0x7B
    I16x8ExtaddPairwiseI8x16S = 0xfd7c,           // i16x8_extadd_pairwise_i8x16_s 0xFD 0x7C
    I16x8ExtaddPairwiseI8x16U = 0xfd7d,           // i16x8_extadd_pairwise_i8x16_u 0xFD 0x7D
    I32x4ExtaddPairwiseI16x8S = 0xfd7e,           // i32x4_extadd_pairwise_i16x8_s 0xFD 0x7E
    I32x4ExtaddPairwiseI16x8U = 0xfd7f,           // i32x4_extadd_pairwise_i16x8_u 0xFD 0x7F
    I16x8Abs(u8) = 0xfd80,                        // i16x8_abs 0xFD 0x80 0x01
    I16x8Neg(u8) = 0xfd81,                        // i16x8_neg 0xFD 0x81 0x01
    I16x8Q15mulrSatS(u8) = 0xfd82,                // i16x8_q15mulr_sat_s 0xFD 0x82 0x01
    I16x8AllTrue(u8) = 0xfd83,                    // i16x8_all_true 0xFD 0x83 0x01
    I16x8Bitmask(u8) = 0xfd84,                    // i16x8_bitmask 0xFD 0x84 0x01
    I16x8NarrowI32x4S(u8) = 0xfd85,               // i16x8_narrow_i32x4_s 0xFD 0x85 0x01
    I16x8NarrowI32x4U(u8) = 0xfd86,               // i16x8_narrow_i32x4_u 0xFD 0x86 0x01
    I16x8ExtendLowI8x16S(u8) = 0xfd87,            // i16x8_extend_low_i8x16_s 0xFD 0x87 0x01
    I16x8ExtendHighI8x16S(u8) = 0xfd88,           // i16x8_extend_high_i8x16_s 0xFD 0x88 0x01
    I16x8ExtendLowI8x16U(u8) = 0xfd89,            // i16x8_extend_low_i8x16_u 0xFD 0x89 0x01
    I16x8ExtendHighI8x16U(u8) = 0xfd8a,           // i16x8_extend_high_i8x16_u 0xFD 0x8A 0x01
    I16x8Shl(u8) = 0xfd8b,                        // i16x8_shl 0xFD 0x8B 0x01
    I16x8ShrS(u8) = 0xfd8c,                       // i16x8_shr_s 0xFD 0x8C 0x01
    I16x8ShrU(u8) = 0xfd8d,                       // i16x8_shr_u 0xFD 0x8D 0x01
    I16x8Add(u8) = 0xfd8e,                        // i16x8_add 0xFD 0x8E 0x01
    I16x8AddSatS(u8) = 0xfd8f,                    // i16x8_add_sat_s 0xFD 0x8F 0x01
    I16x8AddSatU(u8) = 0xfd90,                    // i16x8_add_sat_u 0xFD 0x90 0x01
    I16x8Sub(u8) = 0xfd91,                        // i16x8_sub 0xFD 0x91 0x01
    I16x8SubSatS(u8) = 0xfd92,                    // i16x8_sub_sat_s 0xFD 0x92 0x01
    I16x8SubSatU(u8) = 0xfd93,                    // i16x8_sub_sat_u 0xFD 0x93 0x01
    F64x2Nearest(u8) = 0xfd94,                    // f64x2_nearest 0xFD 0x94 0x01
    I16x8Mul(u8) = 0xfd95,                        // i16x8_mul 0xFD 0x95 0x01
    I16x8MinS(u8) = 0xfd96,                       // i16x8_min_s 0xFD 0x96 0x01
    I16x8MinU(u8) = 0xfd97,                       // i16x8_min_u 0xFD 0x97 0x01
    I16x8MaxS(u8) = 0xfd98,                       // i16x8_max_s 0xFD 0x98 0x01
    I16x8MaxU(u8) = 0xfd99,                       // i16x8_max_u 0xFD 0x99 0x01
    I16x8AvgrU(u8) = 0xfd9b,                      // i16x8_avgr_u 0xFD 0x9B 0x01
    I16x8ExtmulLowI8x16S(u8) = 0xfd9c,            // i16x8_extmul_low_i8x16_s 0xFD 0x9C 0x01
    I16x8ExtmulHighI8x16S(u8) = 0xfd9d,           // i16x8_extmul_high_i8x16_s 0xFD 0x9D 0x01
    I16x8ExtmulLowI8x16U(u8) = 0xfd9e,            // i16x8_extmul_low_i8x16_u 0xFD 0x9E 0x01
    I16x8ExtmulHighI8x16U(u8) = 0xfd9f,           // i16x8_extmul_high_i8x16_u 0xFD 0x9F 0x01
    I32x4Abs(u8) = 0xfda0,                        // i32x4_abs 0xFD 0xA0 0x01
    I32x4Neg(u8) = 0xfda1,                        // i32x4_neg 0xFD 0xA1 0x01
    I32x4AllTrue(u8) = 0xfda3,                    // i32x4_all_true 0xFD 0xA3 0x01
    I32x4Bitmask(u8) = 0xfda4,                    // i32x4_bitmask 0xFD 0xA4 0x01
    I32x4ExtendLowI16x8S(u8) = 0xfda7,            // i32x4_extend_low_i16x8_s 0xFD 0xA7 0x01
    I32x4ExtendHighI16x8S(u8) = 0xfda8,           // i32x4_extend_high_i16x8_s 0xFD 0xA8 0x01
    I32x4ExtendLowI16x8U(u8) = 0xfda9,            // i32x4_extend_low_i16x8_u 0xFD 0xA9 0x01
    I32x4ExtendHighI16x8U(u8) = 0xfdaa,           // i32x4_extend_high_i16x8_u 0xFD 0xAA 0x01
    I32x4Shl(u8) = 0xfdab,                        // i32x4_shl 0xFD 0xAB 0x01
    I32x4ShrS(u8) = 0xfdac,                       // i32x4_shr_s 0xFD 0xAC 0x01
    I32x4ShrU(u8) = 0xfdad,                       // i32x4_shr_u 0xFD 0xAD 0x01
    I32x4Add(u8) = 0xfdae,                        // i32x4_add 0xFD 0xAE 0x01
    I32x4Sub(u8) = 0xfdb1,                        // i32x4_sub 0xFD 0xB1 0x01
    I32x4Mul(u8) = 0xfdb5,                        // i32x4_mul 0xFD 0xB5 0x01
    I32x4MinS(u8) = 0xfdb6,                       // i32x4_min_s 0xFD 0xB6 0x01
    I32x4MinU(u8) = 0xfdb7,                       // i32x4_min_u 0xFD 0xB7 0x01
    I32x4MaxS(u8) = 0xfdb8,                       // i32x4_max_s 0xFD 0xB8 0x01
    I32x4MaxU(u8) = 0xfdb9,                       // i32x4_max_u 0xFD 0xB9 0x01
    I32x4DotI16x8S(u8) = 0xfdba,                  // i32x4_dot_i16x8_s 0xFD 0xBA 0x01
    I32x4ExtmulLowI16x8S(u8) = 0xfdbc,            // i32x4_extmul_low_i16x8_s 0xFD 0xBC 0x01
    I32x4ExtmulHighI16x8S(u8) = 0xfdbd,           // i32x4_extmul_high_i16x8_s 0xFD 0xBD 0x01
    I32x4ExtmulLowI16x8U(u8) = 0xfdbe,            // i32x4_extmul_low_i16x8_u 0xFD 0xBE 0x01
    I32x4ExtmulHighI16x8U(u8) = 0xfdbf,           // i32x4_extmul_high_i16x8_u 0xFD 0xBF 0x01
    I64x2Abs(u8) = 0xfdc0,                        // i64x2_abs 0xFD 0xC0 0x01
    I64x2Neg(u8) = 0xfdc1,                        // i64x2_neg 0xFD 0xC1 0x01
    I64x2AllTrue(u8) = 0xfdc3,                    // i64x2_all_true 0xFD 0xC3 0x01
    I64x2Bitmask(u8) = 0xfdc4,                    // i64x2_bitmask 0xFD 0xC4 0x01
    I64x2ExtendLowI32x4S(u8) = 0xfdc7,            // i64x2_extend_low_i32x4_s 0xFD 0xC7 0x01
    I64x2ExtendHighI32x4S(u8) = 0xfdc8,           // i64x2_extend_high_i32x4_s 0xFD 0xC8 0x01
    I64x2ExtendLowI32x4U(u8) = 0xfdc9,            // i64x2_extend_low_i32x4_u 0xFD 0xC9 0x01
    I64x2ExtendHighI32x4U(u8) = 0xfdca,           // i64x2_extend_high_i32x4_u 0xFD 0xCA 0x01
    I64x2Shl(u8) = 0xfdcb,                        // i64x2_shl 0xFD 0xCB 0x01
    I64x2ShrS(u8) = 0xfdcc,                       // i64x2_shr_s 0xFD 0xCC 0x01
    I64x2ShrU(u8) = 0xfdcd,                       // i64x2_shr_u 0xFD 0xCD 0x01
    I64x2Add(u8) = 0xfdce,                        // i64x2_add 0xFD 0xCE 0x01
    I64x2Sub(u8) = 0xfdd1,                        // i64x2_sub 0xFD 0xD1 0x01
    I64x2Mul(u8) = 0xfdd5,                        // i64x2_mul 0xFD 0xD5 0x01
    I64x2Eq(u8) = 0xfdd6,                         // i64x2_eq 0xFD 0xD6 0x01
    I64x2Ne(u8) = 0xfdd7,                         // i64x2_ne 0xFD 0xD7 0x01
    I64x2LtS(u8) = 0xfdd8,                        // i64x2_lt_s 0xFD 0xD8 0x01
    I64x2GtS(u8) = 0xfdd9,                        // i64x2_gt_s 0xFD 0xD9 0x01
    I64x2LeS(u8) = 0xfdda,                        // i64x2_le_s 0xFD 0xDA 0x01
    I64x2GeS(u8) = 0xfddb,                        // i64x2_ge_s 0xFD 0xDB 0x01
    I64x2ExtmulLowI32x4S(u8) = 0xfddc,            // i64x2_extmul_low_i32x4_s 0xFD 0xDC 0x01
    I64x2ExtmulHighI32x4S(u8) = 0xfddd,           // i64x2_extmul_high_i32x4_s 0xFD 0xDD 0x01
    I64x2ExtmulLowI32x4U(u8) = 0xfdde,            // i64x2_extmul_low_i32x4_u 0xFD 0xDE 0x01
    I64x2ExtmulHighI32x4U(u8) = 0xfddf,           // i64x2_extmul_high_i32x4_u 0xFD 0xDF 0x01
    F32x4Abs(u8) = 0xfde0,                        // f32x4_abs 0xFD 0xE0 0x01
    F32x4Neg(u8) = 0xfde1,                        // f32x4_neg 0xFD 0xE1 0x01
    F32x4Sqrt(u8) = 0xfde3,                       // f32x4_sqrt 0xFD 0xE3 0x01
    F32x4Add(u8) = 0xfde4,                        // f32x4_add 0xFD 0xE4 0x01
    F32x4Sub(u8) = 0xfde5,                        // f32x4_sub 0xFD 0xE5 0x01
    F32x4Mul(u8) = 0xfde6,                        // f32x4_mul 0xFD 0xE6 0x01
    F32x4Div(u8) = 0xfde7,                        // f32x4_div 0xFD 0xE7 0x01
    F32x4Min(u8) = 0xfde8,                        // f32x4_min 0xFD 0xE8 0x01
    F32x4Max(u8) = 0xfde9,                        // f32x4_max 0xFD 0xE9 0x01
    F32x4Pmin(u8) = 0xfdea,                       // f32x4_pmin 0xFD 0xEA 0x01
    F32x4Pmax(u8) = 0xfdeb,                       // f32x4_pmax 0xFD 0xEB 0x01
    F64x2Abs(u8) = 0xfdec,                        // f64x2_abs 0xFD 0xEC 0x01
    F64x2Neg(u8) = 0xfded,                        // f64x2_neg 0xFD 0xED 0x01
    F64x2Sqrt(u8) = 0xfdef,                       // f64x2_sqrt 0xFD 0xEF 0x01
    F64x2Add(u8) = 0xfdf0,                        // f64x2_add 0xFD 0xF0 0x01
    F64x2Sub(u8) = 0xfdf1,                        // f64x2_sub 0xFD 0xF1 0x01
    F64x2Mul(u8) = 0xfdf2,                        // f64x2_mul 0xFD 0xF2 0x01
    F64x2Div(u8) = 0xfdf3,                        // f64x2_div 0xFD 0xF3 0x01
    F64x2Min(u8) = 0xfdf4,                        // f64x2_min 0xFD 0xF4 0x01
    F64x2Max(u8) = 0xfdf5,                        // f64x2_max 0xFD 0xF5 0x01
    F64x2Pmin(u8) = 0xfdf6,                       // f64x2_pmin 0xFD 0xF6 0x01
    F64x2Pmax(u8) = 0xfdf7,                       // f64x2_pmax 0xFD 0xF7 0x01
    I32x4TruncSatF32x4S(u8) = 0xfdf8,             // i32x4_trunc_sat_f32x4_s 0xFD 0xF8 0x01
    I32x4TruncSatF32x4U(u8) = 0xfdf9,             // i32x4_trunc_sat_f32x4_u 0xFD 0xF9 0x01
    F32x4ConvertI32x4S(u8) = 0xfdfa,              // f32x4_convert_i32x4_s 0xFD 0xFA 0x01
    F32x4ConvertI32x4U(u8) = 0xfdfb,              // f32x4_convert_i32x4_u 0xFD 0xFB 0x01
    I32x4TruncSatF64x2SZero(u8) = 0xfdfc,         // i32x4_trunc_sat_f64x2_s_zero 0xFD 0xFC 0x01
    I32x4TruncSatF64x2UZero(u8) = 0xfdfd,         // i32x4_trunc_sat_f64x2_u_zero 0xFD 0xFD 0x01
    F64x2ConvertLowI32x4S(u8) = 0xfdfe,           // f64x2_convert_low_i32x4_s 0xFD 0xFE 0x01
    F64x2ConvertLowI32x4U(u8) = 0xfdff,           // f64x2_convert_low_i32x4_u 0xFD 0xFF 0x01
}
