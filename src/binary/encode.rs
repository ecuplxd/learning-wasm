use super::instruction::{Block, BlockType, BrTableArg, IfBlock, Instruction, MemoryArg};
use super::leb128::{encode_name, encode_signed, encode_u32, encode_unsigned, encode_usize};
use super::section::{
    CodeSeg, CustomSeg, DataMode, DataSeg, ElementMode, ElementSeg, ExportDesc, ExportSeg, Expr,
    GlobalSeg, ImportDesc, ImportSeg, Locals, MaybeU32, Section, TypeIdx,
};
use super::types::{FuncType, GlobalType, Limits, RefType, TableType, ValType};

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
}

pub trait Encodes {
    fn encodes(&self, count_byte: bool) -> Vec<u8>;
}

impl<T> Encodes for Vec<T>
where
    T: Encode,
{
    fn encodes(&self, count_byte: bool) -> Vec<u8> {
        let mut result = vec![];
        let total = encode_usize(self.len());
        let datas = self.iter().flat_map(|item| item.encode()).collect::<Vec<_>>();

        if count_byte {
            result.extend(encode_usize(total.len() + datas.len()));
        }

        result.extend(total);
        result.extend(datas);

        result
    }
}

impl CustomSeg {
    pub fn encode(&self) -> Vec<u8> {
        let mut result = vec![Section::Custom as u8];

        let name = encode_name(&self.name);
        let data = self.data.clone();
        let total = encode_usize(name.len() + data.len());

        result.extend(total);
        result.extend(name);
        result.extend(data);

        result
    }
}

impl Encode for FuncType {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![0x60];

        result.extend(self.params.encodes(false));
        result.extend(self.results.encodes(false));

        result
    }
}

impl Encode for ValType {
    fn encode(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl Encode for ImportSeg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let desc = match &self.desc {
            ImportDesc::Func(idx) => [vec![0x00], idx.encode()].concat(),
            ImportDesc::Table(type_) => [vec![0x01], type_.encode()].concat(),
            ImportDesc::Mem(type_) => [vec![0x02], type_.encode()].concat(),
            ImportDesc::Global(type_) => [vec![0x03], type_.encode()].concat(),
        };

        result.extend(encode_name(&self.module));
        result.extend(encode_name(&self.name));
        result.extend(desc);

        result
    }
}

impl Encode for TypeIdx {
    fn encode(&self) -> Vec<u8> {
        encode_u32(*self)
    }
}

impl Encode for TableType {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let elem_type = match self.elem_type {
            RefType::FuncRef => 0x70,
            RefType::ExternRef => 0x6f,
        };

        result.push(elem_type);
        result.extend(self.limits.encode());

        result
    }
}

impl Encode for Limits {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let with_max = match self.max {
            Some(_) => 1,
            None => 0,
        };

        result.extend(encode_u32(with_max));
        result.extend(encode_u32(self.min));

        if let Some(max) = self.max {
            result.extend(encode_u32(max));
        }

        result
    }
}

impl Encode for GlobalSeg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend(self.type_.encode());
        result.extend(self.init_expr.encode());

        result
    }
}

impl Encode for GlobalType {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend(self.val_type.encode());
        result.push(self.mut_ as u8);

        result
    }
}

impl Encode for ExportSeg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let desc = match &self.desc {
            ExportDesc::Func(idx) => [vec![0x00], idx.encode()].concat(),
            ExportDesc::Table(idx) => [vec![0x01], idx.encode()].concat(),
            ExportDesc::Mem(idx) => [vec![0x02], idx.encode()].concat(),
            ExportDesc::Global(idx) => [vec![0x03], idx.encode()].concat(),
        };

        result.extend(encode_name(&self.name));
        result.extend(desc);

        result
    }
}

pub fn encode_maybeu32_sec(sec_id: Section, v: MaybeU32) -> Vec<u8> {
    let mut result = vec![];

    if let Some(data) = v {
        let data = data.encode();

        result.push(sec_id as u8);
        result.extend(encode_usize(data.len()));
        result.extend(data);
    }

    result
}

impl Encode for ElementSeg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let flag = encode_u32(self.flag);
        let mode = match &self.mode {
            ElementMode::Active {
                table_idx,
                offset_expr,
            } => match self.flag {
                0 | 4 => offset_expr.encode(),
                2 | 6 => [table_idx.encode(), offset_expr.encode()].concat(),
                _ => vec![],
            },
            _ => vec![],
        };
        let type_ = match self.flag {
            0..=4 => vec![],
            _ => self.type_.encode(),
        };
        let elem_kind = match self.flag {
            1..=3 => encode_signed(self.elem_kind as i64),
            _ => vec![],
        };
        let init = match self.flag {
            0..=3 => self.func_idxs.encodes(false),
            _ => {
                let total = encode_usize(self.init_expr.len());
                let expr = self.init_expr.iter().flat_map(|expr| expr.encode()).collect();

                [total, expr].concat()
            }
        };

        result.extend(flag);
        result.extend(mode);
        result.extend(type_);
        result.extend(elem_kind);
        result.extend(init);

        result
    }
}

impl Encode for CodeSeg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let locals = self.locals.encodes(false);
        let body = self.body.encode();
        let size = locals.len() + body.len();

        result.extend(encode_usize(size));
        result.extend(locals);
        result.extend(body);

        result
    }
}

impl Encode for Locals {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend(encode_u32(self.n));
        result.extend(self.value_type.encode());

        result
    }
}

impl Encode for Expr {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        for instr in self {
            result.extend(instr.encode());
        }

        result.push(Instruction::End.discriminant() as u8);

        result
    }
}

impl Encode for DataSeg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let flag = encode_u32(self.flag);
        let mem_idx = match self.flag {
            2 => self.mem_idx.encode(),
            _ => vec![],
        };
        let offset = match self.mode {
            DataMode::Active => self.offset_expr.encode(),
            _ => vec![],
        };
        let init = [encode_usize(self.init.len()), self.init.clone()].concat();

        result.extend(flag);
        result.extend(mem_idx);
        result.extend(offset);
        result.extend(init);

        result
    }
}

impl Instruction {
    pub fn discriminant(&self) -> u16 {
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        let opcode = self.discriminant();
        let opcodes = match opcode < 0xfc {
            true => vec![opcode as u8],
            // 多字节编码
            false => opcode.to_be_bytes().to_vec(),
        };
        let data: Vec<u8> = match self {
            Instruction::Block(block) => block.encode(),
            Instruction::Loop(block) => block.encode(),
            Instruction::If(block) => block.encode(),
            Instruction::Br(data) => encode_u32(*data),
            Instruction::BrIf(data) => encode_u32(*data),
            Instruction::BrTable(arg) => arg.encode(),
            Instruction::Call(data) => encode_u32(*data),
            Instruction::CallIndirect(idx1, idx2) => [idx1.encode(), idx2.encode()].concat(),
            Instruction::Select2(data, type_) => [vec![*data], type_.encode()].concat(),
            Instruction::LocalGet(data) => encode_u32(*data),
            Instruction::LocalSet(data) => encode_u32(*data),
            Instruction::LocalTee(data) => encode_u32(*data),
            Instruction::GlobalGet(data) => encode_u32(*data),
            Instruction::GlobalSet(data) => encode_u32(*data),
            Instruction::TableGet(data) => encode_u32(*data),
            Instruction::TableSet(data) => encode_u32(*data),
            Instruction::I32Load(memarg) => memarg.encode(),
            Instruction::I64Load(memarg) => memarg.encode(),
            Instruction::F32Load(memarg) => memarg.encode(),
            Instruction::F64Load(memarg) => memarg.encode(),
            Instruction::I32Load8S(memarg) => memarg.encode(),
            Instruction::I32Load8U(memarg) => memarg.encode(),
            Instruction::I32Load16S(memarg) => memarg.encode(),
            Instruction::I32Load16U(memarg) => memarg.encode(),
            Instruction::I64Load8S(memarg) => memarg.encode(),
            Instruction::I64Load8U(memarg) => memarg.encode(),
            Instruction::I64Load16S(memarg) => memarg.encode(),
            Instruction::I64Load16U(memarg) => memarg.encode(),
            Instruction::I64Load32S(memarg) => memarg.encode(),
            Instruction::I64Load32U(memarg) => memarg.encode(),
            Instruction::I32Store(memarg) => memarg.encode(),
            Instruction::I64Store(memarg) => memarg.encode(),
            Instruction::F32Store(memarg) => memarg.encode(),
            Instruction::F64Store(memarg) => memarg.encode(),
            Instruction::I32Store8(memarg) => memarg.encode(),
            Instruction::I32Store16(memarg) => memarg.encode(),
            Instruction::I64Store8(memarg) => memarg.encode(),
            Instruction::I64Store16(memarg) => memarg.encode(),
            Instruction::I64Store32(memarg) => memarg.encode(),
            Instruction::MemorySize(data) => vec![*data],
            Instruction::MemoryGrow(data) => vec![*data],
            Instruction::I32Const(data) => encode_signed(*data as i64),
            Instruction::I64Const(data) => encode_signed(*data),
            Instruction::F32Const(data) => data.to_le_bytes().to_vec(),
            Instruction::F64Const(data) => data.to_le_bytes().to_vec(),
            Instruction::RefNull(data) => encode_unsigned(*data),
            Instruction::RefFunc(data) => encode_u32(*data),
            Instruction::MemoryInit(segment, idx) => [encode_u32(*segment), encode_u32(*idx)].concat(),
            Instruction::DataDrop(data) => encode_u32(*data),
            Instruction::MemoryCopy(data1, data2) => [data1.encode(), data2.encode()].concat(),
            Instruction::MemoryFill(data) => encode_u32(*data),
            Instruction::TableInit(data1, data2) => [data1.encode(), data2.encode()].concat(),
            Instruction::ElemDrop(data) => encode_u32(*data),
            Instruction::TableCopy(data1, data2) => [data1.encode(), data2.encode()].concat(),
            Instruction::TableGrow(data) => encode_u32(*data),
            Instruction::TableSize(data) => encode_u32(*data),
            Instruction::TableFill(data) => encode_u32(*data),
            Instruction::V128Load(memarg) => memarg.encode(),
            Instruction::V128Load8x8S(memarg) => memarg.encode(),
            Instruction::V128Load8x8U(memarg) => memarg.encode(),
            Instruction::V128Load16x4S(memarg) => memarg.encode(),
            Instruction::V128Load16x4U(memarg) => memarg.encode(),
            Instruction::V128Load32x2S(memarg) => memarg.encode(),
            Instruction::V128Load32x2U(memarg) => memarg.encode(),
            Instruction::V128Load8Splat(memarg) => memarg.encode(),
            Instruction::V128Load16Splat(memarg) => memarg.encode(),
            Instruction::V128Load32Splat(memarg) => memarg.encode(),
            Instruction::V128Load64Splat(memarg) => memarg.encode(),
            Instruction::V128Store(memarg) => memarg.encode(),
            Instruction::V128Const(data) => data.as_u8x16().to_array().to_vec(),
            Instruction::I8x16Shuffle(lane_16) => lane_16.to_vec(),
            Instruction::I8x16ExtractLaneS(lane) => vec![*lane],
            Instruction::I8x16ExtractLaneU(lane) => vec![*lane],
            Instruction::I8x16ReplaceLane(lane) => vec![*lane],
            Instruction::I16x8ExtractLaneS(lane) => vec![*lane],
            Instruction::I16x8ExtractLaneU(lane) => vec![*lane],
            Instruction::I16x8ReplaceLane(lane) => vec![*lane],
            Instruction::I32x4ExtractLane(lane) => vec![*lane],
            Instruction::I32x4ReplaceLane(lane) => vec![*lane],
            Instruction::I64x2ExtractLane(lane) => vec![*lane],
            Instruction::I64x2ReplaceLane(lane) => vec![*lane],
            Instruction::F32x4ExtractLane(lane) => vec![*lane],
            Instruction::F32x4ReplaceLane(lane) => vec![*lane],
            Instruction::F64x2ExtractLane(lane) => vec![*lane],
            Instruction::F64x2ReplaceLane(lane) => vec![*lane],
            Instruction::V128Load8Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Load16Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Load32Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Load64Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Store8Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Store16Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Store32Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Store64Lane(memarg, lane) => [memarg.encode(), vec![*lane]].concat(),
            Instruction::V128Load32Zero(memarg) => memarg.encode(),
            Instruction::V128Load64Zero(memarg) => memarg.encode(),
            Instruction::I16x8Abs(data) => vec![*data],
            Instruction::I16x8Neg(data) => vec![*data],
            Instruction::I16x8Q15mulrSatS(data) => vec![*data],
            Instruction::I16x8AllTrue(data) => vec![*data],
            Instruction::I16x8Bitmask(data) => vec![*data],
            Instruction::I16x8NarrowI32x4S(data) => vec![*data],
            Instruction::I16x8NarrowI32x4U(data) => vec![*data],
            Instruction::I16x8ExtendLowI8x16S(data) => vec![*data],
            Instruction::I16x8ExtendHighI8x16S(data) => vec![*data],
            Instruction::I16x8ExtendLowI8x16U(data) => vec![*data],
            Instruction::I16x8ExtendHighI8x16U(data) => vec![*data],
            Instruction::I16x8Shl(data) => vec![*data],
            Instruction::I16x8ShrS(data) => vec![*data],
            Instruction::I16x8ShrU(data) => vec![*data],
            Instruction::I16x8Add(data) => vec![*data],
            Instruction::I16x8AddSatS(data) => vec![*data],
            Instruction::I16x8AddSatU(data) => vec![*data],
            Instruction::I16x8Sub(data) => vec![*data],
            Instruction::I16x8SubSatS(data) => vec![*data],
            Instruction::I16x8SubSatU(data) => vec![*data],
            Instruction::F64x2Nearest(data) => vec![*data],
            Instruction::I16x8Mul(data) => vec![*data],
            Instruction::I16x8MinS(data) => vec![*data],
            Instruction::I16x8MinU(data) => vec![*data],
            Instruction::I16x8MaxS(data) => vec![*data],
            Instruction::I16x8MaxU(data) => vec![*data],
            Instruction::I16x8AvgrU(data) => vec![*data],
            Instruction::I16x8ExtmulLowI8x16S(data) => vec![*data],
            Instruction::I16x8ExtmulHighI8x16S(data) => vec![*data],
            Instruction::I16x8ExtmulLowI8x16U(data) => vec![*data],
            Instruction::I16x8ExtmulHighI8x16U(data) => vec![*data],
            Instruction::I32x4Abs(data) => vec![*data],
            Instruction::I32x4Neg(data) => vec![*data],
            Instruction::I32x4AllTrue(data) => vec![*data],
            Instruction::I32x4Bitmask(data) => vec![*data],
            Instruction::I32x4ExtendLowI16x8S(data) => vec![*data],
            Instruction::I32x4ExtendHighI16x8S(data) => vec![*data],
            Instruction::I32x4ExtendLowI16x8U(data) => vec![*data],
            Instruction::I32x4ExtendHighI16x8U(data) => vec![*data],
            Instruction::I32x4Shl(data) => vec![*data],
            Instruction::I32x4ShrS(data) => vec![*data],
            Instruction::I32x4ShrU(data) => vec![*data],
            Instruction::I32x4Add(data) => vec![*data],
            Instruction::I32x4Sub(data) => vec![*data],
            Instruction::I32x4Mul(data) => vec![*data],
            Instruction::I32x4MinS(data) => vec![*data],
            Instruction::I32x4MinU(data) => vec![*data],
            Instruction::I32x4MaxS(data) => vec![*data],
            Instruction::I32x4MaxU(data) => vec![*data],
            Instruction::I32x4DotI16x8S(data) => vec![*data],
            Instruction::I32x4ExtmulLowI16x8S(data) => vec![*data],
            Instruction::I32x4ExtmulHighI16x8S(data) => vec![*data],
            Instruction::I32x4ExtmulLowI16x8U(data) => vec![*data],
            Instruction::I32x4ExtmulHighI16x8U(data) => vec![*data],
            Instruction::I64x2Abs(data) => vec![*data],
            Instruction::I64x2Neg(data) => vec![*data],
            Instruction::I64x2AllTrue(data) => vec![*data],
            Instruction::I64x2Bitmask(data) => vec![*data],
            Instruction::I64x2ExtendLowI32x4S(data) => vec![*data],
            Instruction::I64x2ExtendHighI32x4S(data) => vec![*data],
            Instruction::I64x2ExtendLowI32x4U(data) => vec![*data],
            Instruction::I64x2ExtendHighI32x4U(data) => vec![*data],
            Instruction::I64x2Shl(data) => vec![*data],
            Instruction::I64x2ShrS(data) => vec![*data],
            Instruction::I64x2ShrU(data) => vec![*data],
            Instruction::I64x2Add(data) => vec![*data],
            Instruction::I64x2Sub(data) => vec![*data],
            Instruction::I64x2Mul(data) => vec![*data],
            Instruction::I64x2Eq(data) => vec![*data],
            Instruction::I64x2Ne(data) => vec![*data],
            Instruction::I64x2LtS(data) => vec![*data],
            Instruction::I64x2GtS(data) => vec![*data],
            Instruction::I64x2LeS(data) => vec![*data],
            Instruction::I64x2GeS(data) => vec![*data],
            Instruction::I64x2ExtmulLowI32x4S(data) => vec![*data],
            Instruction::I64x2ExtmulHighI32x4S(data) => vec![*data],
            Instruction::I64x2ExtmulLowI32x4U(data) => vec![*data],
            Instruction::I64x2ExtmulHighI32x4U(data) => vec![*data],
            Instruction::F32x4Abs(data) => vec![*data],
            Instruction::F32x4Neg(data) => vec![*data],
            Instruction::F32x4Sqrt(data) => vec![*data],
            Instruction::F32x4Add(data) => vec![*data],
            Instruction::F32x4Sub(data) => vec![*data],
            Instruction::F32x4Mul(data) => vec![*data],
            Instruction::F32x4Div(data) => vec![*data],
            Instruction::F32x4Min(data) => vec![*data],
            Instruction::F32x4Max(data) => vec![*data],
            Instruction::F32x4Pmin(data) => vec![*data],
            Instruction::F32x4Pmax(data) => vec![*data],
            Instruction::F64x2Abs(data) => vec![*data],
            Instruction::F64x2Neg(data) => vec![*data],
            Instruction::F64x2Sqrt(data) => vec![*data],
            Instruction::F64x2Add(data) => vec![*data],
            Instruction::F64x2Sub(data) => vec![*data],
            Instruction::F64x2Mul(data) => vec![*data],
            Instruction::F64x2Div(data) => vec![*data],
            Instruction::F64x2Min(data) => vec![*data],
            Instruction::F64x2Max(data) => vec![*data],
            Instruction::F64x2Pmin(data) => vec![*data],
            Instruction::F64x2Pmax(data) => vec![*data],
            Instruction::I32x4TruncSatF32x4S(data) => vec![*data],
            Instruction::I32x4TruncSatF32x4U(data) => vec![*data],
            Instruction::F32x4ConvertI32x4S(data) => vec![*data],
            Instruction::F32x4ConvertI32x4U(data) => vec![*data],
            Instruction::I32x4TruncSatF64x2SZero(data) => vec![*data],
            Instruction::I32x4TruncSatF64x2UZero(data) => vec![*data],
            Instruction::F64x2ConvertLowI32x4S(data) => vec![*data],
            Instruction::F64x2ConvertLowI32x4U(data) => vec![*data],
            _ => vec![],
        };

        result.extend(opcodes);
        result.extend(data);

        result
    }
}

impl Encode for MemoryArg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend(encode_u32(self.align));
        result.extend(encode_u32(self.offset));

        result
    }
}

impl Encode for Block {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend(self.type_.encode());
        result.extend(self.expr.encode());

        result
    }
}

impl Encode for IfBlock {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend(self.type_.encode());
        result.extend(self.if_expr.encode());

        if !self.else_expr.is_empty() {
            result.pop();
            result.push(Instruction::Else.discriminant() as u8);
            result.extend(self.else_expr.encode());
        }

        result
    }
}

impl Encode for BlockType {
    fn encode(&self) -> Vec<u8> {
        match self {
            BlockType::I32 => encode_signed(-1),
            BlockType::I64 => encode_signed(-2),
            BlockType::F32 => encode_signed(-3),
            BlockType::F64 => encode_signed(-4),
            BlockType::V128 => encode_signed(-5),
            BlockType::ExternRef => encode_signed(-17),
            BlockType::Empty => encode_signed(-64),
            BlockType::TypeIdx(idx) => encode_signed(*idx as i64),
        }
    }
}

impl Encode for BrTableArg {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];

        result.extend(self.labels.encodes(false));
        result.extend(self.default.encode());

        result
    }
}
