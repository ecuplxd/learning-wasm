use super::errors::DecodeErr;
use super::instruction::{Block, BlockType, BrTableArg, IfBlock, Instruction, MemoryArg};
use super::reader::{DecodeResult, Reader};
use super::section::{
    CodeSeg, CustomSeg, DataMode, DataSeg, ElementMode, ElementSeg, ExportDesc, ExportSeg, Expr,
    FuncIdx, GlobalIdx, GlobalSeg, ImportDesc, ImportSeg, LabelIdx, Locals, MaybeU32, MemIdx, TableIdx,
    TypeIdx,
};
use super::types::{FuncType, GlobalType, Limits, MemType, Mut, RefType, TableType, ValType};

pub trait Decode<T = Self> {
    type Output = T;

    fn decode(reader: &mut Reader) -> DecodeResult<Self::Output>;

    fn decodes(reader: &mut Reader) -> DecodeResult<Vec<Self::Output>> {
        let total = reader.get_leb_u32()?;
        let items = (0..total).map(|_| Self::decode(reader)).collect();

        items
    }
}

impl Decode for CustomSeg {
    fn decode(reader: &mut Reader) -> DecodeResult<CustomSeg> {
        let custom = CustomSeg {
            name: reader.get_name()?,
            data: reader.remain()?,
        };

        Ok(custom)
    }
}

impl Decode for FuncType {
    fn decode(reader: &mut Reader) -> DecodeResult<FuncType> {
        match reader.get_u8()? {
            0x60 => (),
            val => Err(DecodeErr::InvalidType(val))?,
        }

        let func = FuncType {
            params: ValType::decodes(reader)?,
            results: ValType::decodes(reader)?,
        };

        Ok(func)
    }
}

impl Decode for ValType {
    fn decode(reader: &mut Reader) -> DecodeResult<ValType> {
        Ok(ValType::from(reader.get_u8()?))
    }
}

/// import -> module -> name -> desc
impl Decode for ImportSeg {
    fn decode(reader: &mut Reader) -> DecodeResult<ImportSeg> {
        let module = reader.get_name()?;
        let name = reader.get_name()?;
        let desc = match reader.get_u8()? {
            0x00 => ImportDesc::Func(TypeIdx::decode(reader)?),
            0x01 => ImportDesc::Table(TableType::decode(reader)?),
            0x02 => ImportDesc::Mem(MemType::decode(reader)?),
            0x03 => ImportDesc::Global(GlobalType::decode(reader)?),
            kind => Err(DecodeErr::InvalidImportKind(kind))?,
        };

        let import = ImportSeg { module, name, desc };

        Ok(import)
    }
}

/// idx
impl Decode for TypeIdx {
    fn decode(reader: &mut Reader) -> DecodeResult<TypeIdx> {
        reader.get_leb_u32()
    }
}

impl Decode for TableType {
    fn decode(reader: &mut Reader) -> DecodeResult<TableType> {
        let elem_type = match reader.get_u8()? {
            0x70 => RefType::FuncRef,
            0x6f => RefType::ExternRef,
            elem_type => Err(DecodeErr::InvalidTableElemType(elem_type))?,
        };
        let table = TableType {
            elem_type,
            limits: Limits::decode(reader)?,
        };

        Ok(table)
    }
}

impl Decode for Limits {
    fn decode(reader: &mut Reader) -> DecodeResult<Limits> {
        // 0，只指定 min，1 指定 min + max
        let with_max = match reader.get_u8()? {
            0 => false,
            1 => true,
            val => Err(DecodeErr::InvalidLimitMode(val))?,
        };

        let limits = Limits {
            min: reader.get_leb_u32()?,
            max: match with_max {
                true => Some(reader.get_leb_u32()?),
                false => None,
            },
        };

        Ok(limits)
    }
}

impl Decode for GlobalSeg {
    fn decode(reader: &mut Reader) -> DecodeResult<GlobalSeg> {
        let global = GlobalSeg {
            type_: GlobalType::decode(reader)?,
            init_expr: Vec::<Expr>::decode(reader)?,
        };

        Ok(global)
    }
}

impl Decode for GlobalType {
    fn decode(reader: &mut Reader) -> DecodeResult<GlobalType> {
        let global_type = GlobalType {
            val_type: ValType::decode(reader)?,
            mut_: Mut::decode(reader)?,
        };

        Ok(global_type)
    }
}

impl Decode for Mut {
    fn decode(reader: &mut Reader) -> DecodeResult<Mut> {
        Mut::from_u8(reader.get_u8()?)
    }
}

impl Decode for ExportSeg {
    fn decode(reader: &mut Reader) -> DecodeResult<ExportSeg> {
        let name = reader.get_name()?;
        let desc = match reader.get_u8()? {
            0x00 => ExportDesc::Func(FuncIdx::decode(reader)?),
            0x01 => ExportDesc::Table(TableIdx::decode(reader)?),
            0x02 => ExportDesc::Mem(MemIdx::decode(reader)?),
            0x03 => ExportDesc::Global(GlobalIdx::decode(reader)?),
            kind => Err(DecodeErr::InvalidExportKind(kind))?,
        };
        let export = ExportSeg { name, desc };

        Ok(export)
    }
}

impl Decode for MaybeU32 {
    fn decode(reader: &mut Reader) -> DecodeResult<MaybeU32> {
        Ok(Some(reader.get_leb_u32()?))
    }
}

impl Decode for ElementSeg {
    fn decode(reader: &mut Reader) -> DecodeResult<ElementSeg> {
        let flag = reader.get_leb_u32()?;

        if flag > 7 {
            Err(DecodeErr::InvalidElemMode(flag))?
        }

        let mode = match flag {
            0 | 4 => ElementMode::Active {
                table_idx: 0,
                offset_expr: Vec::<Expr>::decode(reader)?,
            },
            2 | 6 => ElementMode::Active {
                table_idx: TableIdx::decode(reader)?,
                offset_expr: Vec::<Expr>::decode(reader)?,
            },
            1 | 5 => ElementMode::Passive,
            3 | 7 => ElementMode::Declarative,
            _ => unreachable!(),
        };
        let type_ = match flag {
            0..=4 => ValType::FuncRef,
            _ => ValType::from(reader.get_u8()?),
        };

        if !type_.is_ref_type() {
            Err(DecodeErr::TableElemNotARef)?
        }

        let elem_kind = match flag {
            1..=3 => reader.get_leb_i32()?,
            _ => 0x00,
        };
        let func_idxs = match flag {
            0..=3 => FuncIdx::decodes(reader)?,
            _ => vec![],
        };
        let init_expr = match flag {
            4..=7 => Vec::<Expr>::decodes(reader)?.into_iter().collect(),
            _ => vec![],
        };

        let element = ElementSeg {
            flag,
            mode,
            type_,
            elem_kind,
            func_idxs,
            init_expr,
        };

        Ok(element)
    }
}

impl Decode for CodeSeg {
    fn decode(reader: &mut Reader) -> DecodeResult<CodeSeg> {
        let size = reader.get_leb_u32()?;
        let body_bytes = reader.bytes(size as usize)?;
        let mut body_reader = Reader::new(&body_bytes, reader.data_count);

        let code = CodeSeg {
            size,
            locals: Locals::decodes(&mut body_reader)?,
            body: Vec::<Expr>::decode(&mut body_reader)?,
        };

        if code.local_size() >= 0x1000_0000 {
            Err(DecodeErr::LocalsTooLarge)?
        }

        Ok(code)
    }
}

impl Decode for Locals {
    fn decode(reader: &mut Reader) -> DecodeResult<Locals> {
        let local = Locals {
            n: reader.get_leb_u32()?,
            value_type: ValType::decode(reader)?,
        };

        Ok(local)
    }
}

impl Decode for Vec<Expr> {
    type Output = Expr;

    fn decode(reader: &mut Reader) -> DecodeResult<Expr> {
        let (exprs, last_instr) = Expr::decode(reader)?;

        match last_instr {
            Instruction::End => Ok(exprs),
            _ => Err(DecodeErr::ExprUnexpectedEnd)?,
        }
    }
}

impl Decode for Expr {
    type Output = (Expr, Instruction);

    fn decode(reader: &mut Reader) -> DecodeResult<(Expr, Instruction)> {
        let mut exprs = vec![];
        let mut last_instr = Instruction::Nop;
        let data_count = reader.data_count;

        while reader.not_end()? {
            let instr = Instruction::decode(reader)?;

            match instr {
                Instruction::End | Instruction::Else => {
                    last_instr = instr;

                    break;
                }
                Instruction::MemoryGrow(idx) if idx != 0 => {
                    Err(DecodeErr::NotZero("MemoryGrow".to_string(), idx))?
                }
                Instruction::MemorySize(idx) if idx != 0 => {
                    Err(DecodeErr::NotZero("MemorySize".to_string(), idx))?
                }
                Instruction::MemoryInit(_, _) if data_count.is_none() => {
                    Err(DecodeErr::LossDataCount("MemoryInit".to_string()))?
                }
                Instruction::DataDrop(_) if data_count.is_none() => {
                    Err(DecodeErr::LossDataCount("DataDrop".to_string()))?
                }
                _ => exprs.push(instr),
            };
        }

        Ok((exprs, last_instr))
    }
}

impl Decode for DataSeg {
    fn decode(reader: &mut Reader) -> DecodeResult<DataSeg> {
        let flag = reader.get_leb_u32()?;

        if flag > 2 {
            Err(DecodeErr::InvalidDataMode(flag))?
        }

        let mode = match flag {
            0 | 2 => DataMode::Active,
            _ => DataMode::Passive,
        };
        let mem_idx = match flag {
            2 => MemIdx::decode(reader)?,
            _ => 0,
        };
        let offset_expr = match mode {
            DataMode::Active => Vec::<Expr>::decode(reader)?,
            _ => vec![],
        };
        let init = reader.seqs()?;

        let data = DataSeg {
            flag,
            mode,
            init,
            mem_idx,
            offset_expr,
        };

        Ok(data)
    }
}

impl Decode for Instruction {
    fn decode(reader: &mut Reader) -> DecodeResult<Instruction> {
        let instruction = match reader.get_u8()? {
            0x00 => Instruction::Unreachable,
            0x01 => Instruction::Nop,
            0x02 => Instruction::Block(Block::decode(reader)?),
            0x03 => Instruction::Loop(Block::decode(reader)?),
            0x04 => Instruction::If(IfBlock::decode(reader)?),
            0x05 => Instruction::Else,
            0x0b => Instruction::End,
            0x0c => Instruction::Br(reader.get_leb_u32()?),
            0x0d => Instruction::BrIf(reader.get_leb_u32()?),
            0x0e => Instruction::BrTable(BrTableArg::decode(reader)?),
            0x0f => Instruction::Return,
            0x10 => Instruction::Call(reader.get_leb_u32()?),
            0x11 => Instruction::CallIndirect(reader.get_leb_u32()?, reader.get_leb_u32()?),
            0x1a => Instruction::Drop,
            0x1b => Instruction::Select,
            0x1c => Instruction::Select2(reader.get_u8()?, ValType::decode(reader)?),
            0x20 => Instruction::LocalGet(reader.get_leb_u32()?),
            0x21 => Instruction::LocalSet(reader.get_leb_u32()?),
            0x22 => Instruction::LocalTee(reader.get_leb_u32()?),
            0x23 => Instruction::GlobalGet(reader.get_leb_u32()?),
            0x24 => Instruction::GlobalSet(reader.get_leb_u32()?),
            0x25 => Instruction::TableGet(reader.get_leb_u32()?),
            0x26 => Instruction::TableSet(reader.get_leb_u32()?),
            0x28 => Instruction::I32Load(MemoryArg::decode(reader)?),
            0x29 => Instruction::I64Load(MemoryArg::decode(reader)?),
            0x2a => Instruction::F32Load(MemoryArg::decode(reader)?),
            0x2b => Instruction::F64Load(MemoryArg::decode(reader)?),
            0x2c => Instruction::I32Load8S(MemoryArg::decode(reader)?),
            0x2d => Instruction::I32Load8U(MemoryArg::decode(reader)?),
            0x2e => Instruction::I32Load16S(MemoryArg::decode(reader)?),
            0x2f => Instruction::I32Load16U(MemoryArg::decode(reader)?),
            0x30 => Instruction::I64Load8S(MemoryArg::decode(reader)?),
            0x31 => Instruction::I64Load8U(MemoryArg::decode(reader)?),
            0x32 => Instruction::I64Load16S(MemoryArg::decode(reader)?),
            0x33 => Instruction::I64Load16U(MemoryArg::decode(reader)?),
            0x34 => Instruction::I64Load32S(MemoryArg::decode(reader)?),
            0x35 => Instruction::I64Load32U(MemoryArg::decode(reader)?),
            0x36 => Instruction::I32Store(MemoryArg::decode(reader)?),
            0x37 => Instruction::I64Store(MemoryArg::decode(reader)?),
            0x38 => Instruction::F32Store(MemoryArg::decode(reader)?),
            0x39 => Instruction::F64Store(MemoryArg::decode(reader)?),
            0x3a => Instruction::I32Store8(MemoryArg::decode(reader)?),
            0x3b => Instruction::I32Store16(MemoryArg::decode(reader)?),
            0x3c => Instruction::I64Store8(MemoryArg::decode(reader)?),
            0x3d => Instruction::I64Store16(MemoryArg::decode(reader)?),
            0x3e => Instruction::I64Store32(MemoryArg::decode(reader)?),
            0x3f => Instruction::MemorySize(reader.get_u8()?),
            0x40 => Instruction::MemoryGrow(reader.get_u8()?),
            0x41 => Instruction::I32Const(reader.get_leb_i32()?),
            0x42 => Instruction::I64Const(reader.get_leb_i64()?),
            0x43 => Instruction::F32Const(reader.get_f32()?),
            0x44 => Instruction::F64Const(reader.get_f64()?),
            0x45 => Instruction::I32Eqz,
            0x46 => Instruction::I32Eq,
            0x47 => Instruction::I32Ne,
            0x48 => Instruction::I32LtS,
            0x49 => Instruction::I32LtU,
            0x4a => Instruction::I32GtS,
            0x4b => Instruction::I32GtU,
            0x4c => Instruction::I32LeS,
            0x4d => Instruction::I32LeU,
            0x4e => Instruction::I32GeS,
            0x4f => Instruction::I32GeU,
            0x50 => Instruction::I64Eqz,
            0x51 => Instruction::I64Eq,
            0x52 => Instruction::I64Ne,
            0x53 => Instruction::I64LtS,
            0x54 => Instruction::I64LtU,
            0x55 => Instruction::I64GtS,
            0x56 => Instruction::I64GtU,
            0x57 => Instruction::I64LeS,
            0x58 => Instruction::I64LeU,
            0x59 => Instruction::I64GeS,
            0x5a => Instruction::I64GeU,
            0x5b => Instruction::F32Eq,
            0x5c => Instruction::F32Ne,
            0x5d => Instruction::F32Lt,
            0x5e => Instruction::F32Gt,
            0x5f => Instruction::F32Le,
            0x60 => Instruction::F32Ge,
            0x61 => Instruction::F64Eq,
            0x62 => Instruction::F64Ne,
            0x63 => Instruction::F64Lt,
            0x64 => Instruction::F64Gt,
            0x65 => Instruction::F64Le,
            0x66 => Instruction::F64Ge,
            0x67 => Instruction::I32Clz,
            0x68 => Instruction::I32Ctz,
            0x69 => Instruction::I32Popcnt,
            0x6a => Instruction::I32Add,
            0x6b => Instruction::I32Sub,
            0x6c => Instruction::I32Mul,
            0x6d => Instruction::I32DivS,
            0x6e => Instruction::I32DivU,
            0x6f => Instruction::I32RemS,
            0x70 => Instruction::I32RemU,
            0x71 => Instruction::I32And,
            0x72 => Instruction::I32Or,
            0x73 => Instruction::I32Xor,
            0x74 => Instruction::I32Shl,
            0x75 => Instruction::I32ShrS,
            0x76 => Instruction::I32ShrU,
            0x77 => Instruction::I32Rotl,
            0x78 => Instruction::I32Rotr,
            0x79 => Instruction::I64Clz,
            0x7a => Instruction::I64Ctz,
            0x7b => Instruction::I64Popcnt,
            0x7c => Instruction::I64Add,
            0x7d => Instruction::I64Sub,
            0x7e => Instruction::I64Mul,
            0x7f => Instruction::I64DivS,
            0x80 => Instruction::I64DivU,
            0x81 => Instruction::I64RemS,
            0x82 => Instruction::I64RemU,
            0x83 => Instruction::I64And,
            0x84 => Instruction::I64Or,
            0x85 => Instruction::I64Xor,
            0x86 => Instruction::I64Shl,
            0x87 => Instruction::I64ShrS,
            0x88 => Instruction::I64ShrU,
            0x89 => Instruction::I64Rotl,
            0x8a => Instruction::I64Rotr,
            0x8b => Instruction::F32Abs,
            0x8c => Instruction::F32Neg,
            0x8d => Instruction::F32Ceil,
            0x8e => Instruction::F32Floor,
            0x8f => Instruction::F32Trunc,
            0x90 => Instruction::F32Nearest,
            0x91 => Instruction::F32Sqrt,
            0x92 => Instruction::F32Add,
            0x93 => Instruction::F32Sub,
            0x94 => Instruction::F32Mul,
            0x95 => Instruction::F32Div,
            0x96 => Instruction::F32Min,
            0x97 => Instruction::F32Max,
            0x98 => Instruction::F32Copysign,
            0x99 => Instruction::F64Abs,
            0x9a => Instruction::F64Neg,
            0x9b => Instruction::F64Ceil,
            0x9c => Instruction::F64Floor,
            0x9d => Instruction::F64Trunc,
            0x9e => Instruction::F64Nearest,
            0x9f => Instruction::F64Sqrt,
            0xa0 => Instruction::F64Add,
            0xa1 => Instruction::F64Sub,
            0xa2 => Instruction::F64Mul,
            0xa3 => Instruction::F64Div,
            0xa4 => Instruction::F64Min,
            0xa5 => Instruction::F64Max,
            0xa6 => Instruction::F64Copysign,
            0xa7 => Instruction::I32WrapI64,
            0xa8 => Instruction::I32TruncF32S,
            0xa9 => Instruction::I32TruncF32U,
            0xaa => Instruction::I32TruncF64S,
            0xab => Instruction::I32TruncF64U,
            0xac => Instruction::I64ExtendI32S,
            0xad => Instruction::I64ExtendI32U,
            0xae => Instruction::I64TruncF32S,
            0xaf => Instruction::I64TruncF32U,
            0xb0 => Instruction::I64TruncF64S,
            0xb1 => Instruction::I64TruncF64U,
            0xb2 => Instruction::F32ConvertI32S,
            0xb3 => Instruction::F32ConvertI32U,
            0xb4 => Instruction::F32ConvertI64S,
            0xb5 => Instruction::F32ConvertI64U,
            0xb6 => Instruction::F32DemoteF64,
            0xb7 => Instruction::F64ConvertI32S,
            0xb8 => Instruction::F64ConvertI32U,
            0xb9 => Instruction::F64ConvertI64S,
            0xba => Instruction::F64ConvertI64U,
            0xbb => Instruction::F64PromoteF32,
            0xbc => Instruction::I32ReinterpretF32,
            0xbd => Instruction::I64ReinterpretF64,
            0xbe => Instruction::F32ReinterpretI32,
            0xbf => Instruction::F64ReinterpretI64,
            0xc0 => Instruction::I32Extend8S,
            0xc1 => Instruction::I32Extend16S,
            0xc2 => Instruction::I64Extend8S,
            0xc3 => Instruction::I64Extend16S,
            0xc4 => Instruction::I64Extend32S,
            0xd0 => Instruction::RefNull(reader.get_leb_u64()?),
            0xd1 => Instruction::RefIsNull,
            0xd2 => Instruction::RefFunc(reader.get_leb_u32()?),
            0xfc => match reader.get_leb_u32()? {
                0x00 => Instruction::I32TruncSatF32S,
                0x01 => Instruction::I32TruncSatF32U,
                0x02 => Instruction::I32TruncSatF64S,
                0x03 => Instruction::I32TruncSatF64U,
                0x04 => Instruction::I64TruncSatF32S,
                0x05 => Instruction::I64TruncSatF32U,
                0x06 => Instruction::I64TruncSatF64S,
                0x07 => Instruction::I64TruncSatF64U,
                0x08 => Instruction::MemoryInit(reader.get_leb_u32()?, reader.get_leb_u32()?),
                0x09 => Instruction::DataDrop(reader.get_leb_u32()?),
                0x0a => Instruction::MemoryCopy(reader.get_leb_u32()?, reader.get_leb_u32()?),
                0x0b => Instruction::MemoryFill(reader.get_leb_u32()?),
                0x0c => Instruction::TableInit(reader.get_leb_u32()?, reader.get_leb_u32()?),
                0x0d => Instruction::ElemDrop(reader.get_leb_u32()?),
                0x0e => Instruction::TableCopy(reader.get_leb_u32()?, reader.get_leb_u32()?),
                0x0f => Instruction::TableGrow(reader.get_leb_u32()?),
                0x10 => Instruction::TableSize(reader.get_leb_u32()?),
                0x11 => Instruction::TableFill(reader.get_leb_u32()?),
                opcode => Err(DecodeErr::UnknownOpcode(0xfc, opcode as u8))?,
            },
            0xfd => match reader.get_u8()? {
                0x00 => Instruction::V128Load(MemoryArg::decode(reader)?),
                0x01 => Instruction::V128Load8x8S(MemoryArg::decode(reader)?),
                0x02 => Instruction::V128Load8x8U(MemoryArg::decode(reader)?),
                0x03 => Instruction::V128Load16x4S(MemoryArg::decode(reader)?),
                0x04 => Instruction::V128Load16x4U(MemoryArg::decode(reader)?),
                0x05 => Instruction::V128Load32x2S(MemoryArg::decode(reader)?),
                0x06 => Instruction::V128Load32x2U(MemoryArg::decode(reader)?),
                0x07 => Instruction::V128Load8Splat(MemoryArg::decode(reader)?),
                0x08 => Instruction::V128Load16Splat(MemoryArg::decode(reader)?),
                0x09 => Instruction::V128Load32Splat(MemoryArg::decode(reader)?),
                0x0a => Instruction::V128Load64Splat(MemoryArg::decode(reader)?),
                0x0b => Instruction::V128Store(MemoryArg::decode(reader)?),
                0x0c => Instruction::V128Const(reader.get_v128()?),
                0x0d => Instruction::I8x16Shuffle(reader.byte_16()?),
                0x0e => Instruction::I8x16Swizzle,
                0x0f => Instruction::I8x16Splat,
                0x10 => Instruction::I16x8Splat,
                0x11 => Instruction::I32x4Splat,
                0x12 => Instruction::I64x2Splat,
                0x13 => Instruction::F32x4Splat,
                0x14 => Instruction::F64x2Splat,
                0x15 => Instruction::I8x16ExtractLaneS(reader.get_u8()?),
                0x16 => Instruction::I8x16ExtractLaneU(reader.get_u8()?),
                0x17 => Instruction::I8x16ReplaceLane(reader.get_u8()?),
                0x18 => Instruction::I16x8ExtractLaneS(reader.get_u8()?),
                0x19 => Instruction::I16x8ExtractLaneU(reader.get_u8()?),
                0x1a => Instruction::I16x8ReplaceLane(reader.get_u8()?),
                0x1b => Instruction::I32x4ExtractLane(reader.get_u8()?),
                0x1c => Instruction::I32x4ReplaceLane(reader.get_u8()?),
                0x1d => Instruction::I64x2ExtractLane(reader.get_u8()?),
                0x1e => Instruction::I64x2ReplaceLane(reader.get_u8()?),
                0x1f => Instruction::F32x4ExtractLane(reader.get_u8()?),
                0x20 => Instruction::F32x4ReplaceLane(reader.get_u8()?),
                0x21 => Instruction::F64x2ExtractLane(reader.get_u8()?),
                0x22 => Instruction::F64x2ReplaceLane(reader.get_u8()?),
                0x23 => Instruction::I8x16Eq,
                0x24 => Instruction::I8x16Ne,
                0x25 => Instruction::I8x16LtS,
                0x26 => Instruction::I8x16LtU,
                0x27 => Instruction::I8x16GtS,
                0x28 => Instruction::I8x16GtU,
                0x29 => Instruction::I8x16LeS,
                0x2a => Instruction::I8x16LeU,
                0x2b => Instruction::I8x16GeS,
                0x2c => Instruction::I8x16GeU,
                0x2d => Instruction::I16x8Eq,
                0x2e => Instruction::I16x8Ne,
                0x2f => Instruction::I16x8LtS,
                0x30 => Instruction::I16x8LtU,
                0x31 => Instruction::I16x8GtS,
                0x32 => Instruction::I16x8GtU,
                0x33 => Instruction::I16x8LeS,
                0x34 => Instruction::I16x8LeU,
                0x35 => Instruction::I16x8GeS,
                0x36 => Instruction::I16x8GeU,
                0x37 => Instruction::I32x4Eq,
                0x38 => Instruction::I32x4Ne,
                0x39 => Instruction::I32x4LtS,
                0x3a => Instruction::I32x4LtU,
                0x3b => Instruction::I32x4GtS,
                0x3c => Instruction::I32x4GtU,
                0x3d => Instruction::I32x4LeS,
                0x3e => Instruction::I32x4LeU,
                0x3f => Instruction::I32x4GeS,
                0x40 => Instruction::I32x4GeU,
                0x41 => Instruction::F32x4Eq,
                0x42 => Instruction::F32x4Ne,
                0x43 => Instruction::F32x4Lt,
                0x44 => Instruction::F32x4Gt,
                0x45 => Instruction::F32x4Le,
                0x46 => Instruction::F32x4Ge,
                0x47 => Instruction::F64x2Eq,
                0x48 => Instruction::F64x2Ne,
                0x49 => Instruction::F64x2Lt,
                0x4a => Instruction::F64x2Gt,
                0x4b => Instruction::F64x2Le,
                0x4c => Instruction::F64x2Ge,
                0x4d => Instruction::V128Not,
                0x4e => Instruction::V128And,
                0x4f => Instruction::V128Andnot,
                0x50 => Instruction::V128Or,
                0x51 => Instruction::V128Xor,
                0x52 => Instruction::V128Bitselect,
                0x53 => Instruction::V128AnyTrue,
                0x54 => Instruction::V128Load8Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x55 => Instruction::V128Load16Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x56 => Instruction::V128Load32Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x57 => Instruction::V128Load64Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x58 => Instruction::V128Store8Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x59 => Instruction::V128Store16Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x5a => Instruction::V128Store32Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x5b => Instruction::V128Store64Lane(MemoryArg::decode(reader)?, reader.get_u8()?),
                0x5c => Instruction::V128Load32Zero(MemoryArg::decode(reader)?),
                0x5d => Instruction::V128Load64Zero(MemoryArg::decode(reader)?),
                0x5e => Instruction::F32x4DemoteF64x2Zero,
                0x5f => Instruction::F64x2PromoteLowF32x4,
                0x60 => Instruction::I8x16Abs,
                0x61 => Instruction::I8x16Neg,
                0x62 => Instruction::I8x16Popcnt,
                0x63 => Instruction::I8x16AllTrue,
                0x64 => Instruction::I8x16Bitmask,
                0x65 => Instruction::I8x16NarrowI16x8S,
                0x66 => Instruction::I8x16NarrowI16x8U,
                0x67 => Instruction::F32x4Ceil,
                0x68 => Instruction::F32x4Floor,
                0x69 => Instruction::F32x4Trunc,
                0x6a => Instruction::F32x4Nearest,
                0x6b => Instruction::I8x16Shl,
                0x6c => Instruction::I8x16ShrS,
                0x6d => Instruction::I8x16ShrU,
                0x6e => Instruction::I8x16Add,
                0x6f => Instruction::I8x16AddSatS,
                0x70 => Instruction::I8x16AddSatU,
                0x71 => Instruction::I8x16Sub,
                0x72 => Instruction::I8x16SubSatS,
                0x73 => Instruction::I8x16SubSatU,
                0x74 => Instruction::F64x2Ceil,
                0x75 => Instruction::F64x2Floor,
                0x76 => Instruction::I8x16MinS,
                0x77 => Instruction::I8x16MinU,
                0x78 => Instruction::I8x16MaxS,
                0x79 => Instruction::I8x16MaxU,
                0x7a => Instruction::F64x2Trunc,
                0x7b => Instruction::I8x16AvgrU,
                0x7c => Instruction::I16x8ExtaddPairwiseI8x16S,
                0x7d => Instruction::I16x8ExtaddPairwiseI8x16U,
                0x7e => Instruction::I32x4ExtaddPairwiseI16x8S,
                0x7f => Instruction::I32x4ExtaddPairwiseI16x8U,
                0x80 => Instruction::I16x8Abs(reader.get_u8()?),
                0x81 => Instruction::I16x8Neg(reader.get_u8()?),
                0x82 => Instruction::I16x8Q15mulrSatS(reader.get_u8()?),
                0x83 => Instruction::I16x8AllTrue(reader.get_u8()?),
                0x84 => Instruction::I16x8Bitmask(reader.get_u8()?),
                0x85 => Instruction::I16x8NarrowI32x4S(reader.get_u8()?),
                0x86 => Instruction::I16x8NarrowI32x4U(reader.get_u8()?),
                0x87 => Instruction::I16x8ExtendLowI8x16S(reader.get_u8()?),
                0x88 => Instruction::I16x8ExtendHighI8x16S(reader.get_u8()?),
                0x89 => Instruction::I16x8ExtendLowI8x16U(reader.get_u8()?),
                0x8a => Instruction::I16x8ExtendHighI8x16U(reader.get_u8()?),
                0x8b => Instruction::I16x8Shl(reader.get_u8()?),
                0x8c => Instruction::I16x8ShrS(reader.get_u8()?),
                0x8d => Instruction::I16x8ShrU(reader.get_u8()?),
                0x8e => Instruction::I16x8Add(reader.get_u8()?),
                0x8f => Instruction::I16x8AddSatS(reader.get_u8()?),
                0x90 => Instruction::I16x8AddSatU(reader.get_u8()?),
                0x91 => Instruction::I16x8Sub(reader.get_u8()?),
                0x92 => Instruction::I16x8SubSatS(reader.get_u8()?),
                0x93 => Instruction::I16x8SubSatU(reader.get_u8()?),
                0x94 => Instruction::F64x2Nearest(reader.get_u8()?),
                0x95 => Instruction::I16x8Mul(reader.get_u8()?),
                0x96 => Instruction::I16x8MinS(reader.get_u8()?),
                0x97 => Instruction::I16x8MinU(reader.get_u8()?),
                0x98 => Instruction::I16x8MaxS(reader.get_u8()?),
                0x99 => Instruction::I16x8MaxU(reader.get_u8()?),
                0x9b => Instruction::I16x8AvgrU(reader.get_u8()?),
                0x9c => Instruction::I16x8ExtmulLowI8x16S(reader.get_u8()?),
                0x9d => Instruction::I16x8ExtmulHighI8x16S(reader.get_u8()?),
                0x9e => Instruction::I16x8ExtmulLowI8x16U(reader.get_u8()?),
                0x9f => Instruction::I16x8ExtmulHighI8x16U(reader.get_u8()?),
                0xa0 => Instruction::I32x4Abs(reader.get_u8()?),
                0xa1 => Instruction::I32x4Neg(reader.get_u8()?),
                0xa3 => Instruction::I32x4AllTrue(reader.get_u8()?),
                0xa4 => Instruction::I32x4Bitmask(reader.get_u8()?),
                0xa7 => Instruction::I32x4ExtendLowI16x8S(reader.get_u8()?),
                0xa8 => Instruction::I32x4ExtendHighI16x8S(reader.get_u8()?),
                0xa9 => Instruction::I32x4ExtendLowI16x8U(reader.get_u8()?),
                0xaa => Instruction::I32x4ExtendHighI16x8U(reader.get_u8()?),
                0xab => Instruction::I32x4Shl(reader.get_u8()?),
                0xac => Instruction::I32x4ShrS(reader.get_u8()?),
                0xad => Instruction::I32x4ShrU(reader.get_u8()?),
                0xae => Instruction::I32x4Add(reader.get_u8()?),
                0xb1 => Instruction::I32x4Sub(reader.get_u8()?),
                0xb5 => Instruction::I32x4Mul(reader.get_u8()?),
                0xb6 => Instruction::I32x4MinS(reader.get_u8()?),
                0xb7 => Instruction::I32x4MinU(reader.get_u8()?),
                0xb8 => Instruction::I32x4MaxS(reader.get_u8()?),
                0xb9 => Instruction::I32x4MaxU(reader.get_u8()?),
                0xba => Instruction::I32x4DotI16x8S(reader.get_u8()?),
                0xbc => Instruction::I32x4ExtmulLowI16x8S(reader.get_u8()?),
                0xbd => Instruction::I32x4ExtmulHighI16x8S(reader.get_u8()?),
                0xbe => Instruction::I32x4ExtmulLowI16x8U(reader.get_u8()?),
                0xbf => Instruction::I32x4ExtmulHighI16x8U(reader.get_u8()?),
                0xc0 => Instruction::I64x2Abs(reader.get_u8()?),
                0xc1 => Instruction::I64x2Neg(reader.get_u8()?),
                0xc3 => Instruction::I64x2AllTrue(reader.get_u8()?),
                0xc4 => Instruction::I64x2Bitmask(reader.get_u8()?),
                0xc7 => Instruction::I64x2ExtendLowI32x4S(reader.get_u8()?),
                0xc8 => Instruction::I64x2ExtendHighI32x4S(reader.get_u8()?),
                0xc9 => Instruction::I64x2ExtendLowI32x4U(reader.get_u8()?),
                0xca => Instruction::I64x2ExtendHighI32x4U(reader.get_u8()?),
                0xcb => Instruction::I64x2Shl(reader.get_u8()?),
                0xcc => Instruction::I64x2ShrS(reader.get_u8()?),
                0xcd => Instruction::I64x2ShrU(reader.get_u8()?),
                0xce => Instruction::I64x2Add(reader.get_u8()?),
                0xd1 => Instruction::I64x2Sub(reader.get_u8()?),
                0xd5 => Instruction::I64x2Mul(reader.get_u8()?),
                0xd6 => Instruction::I64x2Eq(reader.get_u8()?),
                0xd7 => Instruction::I64x2Ne(reader.get_u8()?),
                0xd8 => Instruction::I64x2LtS(reader.get_u8()?),
                0xd9 => Instruction::I64x2GtS(reader.get_u8()?),
                0xda => Instruction::I64x2LeS(reader.get_u8()?),
                0xdb => Instruction::I64x2GeS(reader.get_u8()?),
                0xdc => Instruction::I64x2ExtmulLowI32x4S(reader.get_u8()?),
                0xdd => Instruction::I64x2ExtmulHighI32x4S(reader.get_u8()?),
                0xde => Instruction::I64x2ExtmulLowI32x4U(reader.get_u8()?),
                0xdf => Instruction::I64x2ExtmulHighI32x4U(reader.get_u8()?),
                0xe0 => Instruction::F32x4Abs(reader.get_u8()?),
                0xe1 => Instruction::F32x4Neg(reader.get_u8()?),
                0xe3 => Instruction::F32x4Sqrt(reader.get_u8()?),
                0xe4 => Instruction::F32x4Add(reader.get_u8()?),
                0xe5 => Instruction::F32x4Sub(reader.get_u8()?),
                0xe6 => Instruction::F32x4Mul(reader.get_u8()?),
                0xe7 => Instruction::F32x4Div(reader.get_u8()?),
                0xe8 => Instruction::F32x4Min(reader.get_u8()?),
                0xe9 => Instruction::F32x4Max(reader.get_u8()?),
                0xea => Instruction::F32x4Pmin(reader.get_u8()?),
                0xeb => Instruction::F32x4Pmax(reader.get_u8()?),
                0xec => Instruction::F64x2Abs(reader.get_u8()?),
                0xed => Instruction::F64x2Neg(reader.get_u8()?),
                0xef => Instruction::F64x2Sqrt(reader.get_u8()?),
                0xf0 => Instruction::F64x2Add(reader.get_u8()?),
                0xf1 => Instruction::F64x2Sub(reader.get_u8()?),
                0xf2 => Instruction::F64x2Mul(reader.get_u8()?),
                0xf3 => Instruction::F64x2Div(reader.get_u8()?),
                0xf4 => Instruction::F64x2Min(reader.get_u8()?),
                0xf5 => Instruction::F64x2Max(reader.get_u8()?),
                0xf6 => Instruction::F64x2Pmin(reader.get_u8()?),
                0xf7 => Instruction::F64x2Pmax(reader.get_u8()?),
                0xf8 => Instruction::I32x4TruncSatF32x4S(reader.get_u8()?),
                0xf9 => Instruction::I32x4TruncSatF32x4U(reader.get_u8()?),
                0xfa => Instruction::F32x4ConvertI32x4S(reader.get_u8()?),
                0xfb => Instruction::F32x4ConvertI32x4U(reader.get_u8()?),
                0xfc => Instruction::I32x4TruncSatF64x2SZero(reader.get_u8()?),
                0xfd => Instruction::I32x4TruncSatF64x2UZero(reader.get_u8()?),
                0xfe => Instruction::F64x2ConvertLowI32x4S(reader.get_u8()?),
                0xff => Instruction::F64x2ConvertLowI32x4U(reader.get_u8()?),
                opcode => Err(DecodeErr::UnknownOpcode(0xfd, opcode))?,
            },
            prefix => Err(DecodeErr::UnknownOpcodePrefix(prefix))?,
        };

        Ok(instruction)
    }
}

impl Decode for MemoryArg {
    fn decode(reader: &mut Reader) -> DecodeResult<MemoryArg> {
        let mem_arg = MemoryArg {
            align: reader.get_leb_u32()?,
            offset: reader.get_leb_u32()?,
        };

        Ok(mem_arg)
    }
}

impl Decode for Block {
    fn decode(reader: &mut Reader) -> DecodeResult<Block> {
        let block_type = BlockType::decode(reader)?;
        let (expr, last_instr) = Expr::decode(reader)?;

        match last_instr {
            Instruction::End => Ok(Block::new(block_type, expr)),
            _ => Err(DecodeErr::InvalidBlock)?,
        }
    }
}

impl Decode for IfBlock {
    fn decode(reader: &mut Reader) -> DecodeResult<IfBlock> {
        let block_type = BlockType::decode(reader)?;
        let (if_expr, last_instr) = Expr::decode(reader)?;

        let mut if_block = IfBlock {
            type_: block_type,
            if_expr,
            else_expr: vec![],
        };

        if matches!(last_instr, Instruction::Else) {
            let (else_expr, last_instr) = Expr::decode(reader)?;

            match last_instr {
                Instruction::End => if_block.else_expr = else_expr,
                _ => Err(DecodeErr::InvalidElseBlock)?,
            }
        }

        Ok(if_block)
    }
}

impl Decode for BlockType {
    fn decode(reader: &mut Reader) -> DecodeResult<BlockType> {
        Ok(BlockType::from(reader.get_leb_i32()?))
    }
}

impl Decode for BrTableArg {
    fn decode(reader: &mut Reader) -> DecodeResult<BrTableArg> {
        let br_table_arg = BrTableArg {
            labels: LabelIdx::decodes(reader)?,
            default: LabelIdx::decode(reader)?,
        };

        Ok(br_table_arg)
    }
}
