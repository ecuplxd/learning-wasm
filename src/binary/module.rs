use std::fs;

use super::decode::Decode;
use super::encode::{encode_maybeu32_sec, Encode, Encodes};
use super::errors::DecodeErr;
use super::reader::{DecodeResult, Reader};
use super::section::{
    CodeSeg, CustomSeg, DataCountSeg, DataSeg, ElementSeg, ExportSeg, GlobalSeg, ImportSeg, Section,
    StartSeg, TypeIdx,
};
use super::types::*;
use super::validate::{Validate, ValidateResult};

const MAGIC: u32 = 0x6d736100;
const VERSION: u32 = 0x00000001;

/// Modules
#[derive(Debug, Default)]
pub struct Module {
    magic: String,
    version: u32,
    custom_sec: Vec<CustomSeg>,
    pub type_sec: Vec<FuncType>,
    pub import_sec: Vec<ImportSeg>,
    pub func_sec: Vec<TypeIdx>,
    pub table_sec: Vec<TableType>,
    pub mem_sec: Vec<MemType>,
    pub global_sec: Vec<GlobalSeg>,
    pub export_sec: Vec<ExportSeg>,
    pub start_sec: StartSeg,
    pub elem_sec: Vec<ElementSeg>,
    pub code_sec: Vec<CodeSeg>,
    pub data_sec: Vec<DataSeg>,
    /// 校验 data_sec
    pub data_counat_sec: DataCountSeg,
}

impl Module {
    pub fn new() -> Self {
        Self {
            magic: "\0asm".to_string(),
            version: VERSION,
            ..Default::default()
        }
    }

    pub fn from_file(path: &str) -> DecodeResult<Self> {
        let wasm = fs::read(path).expect("文件读取失败");

        Self::from_data(wasm)
    }

    pub fn from_data(data: Vec<u8>) -> DecodeResult<Self> {
        let mut reader = Reader::new(&data, None);

        Self::decode(&mut reader)
    }

    fn encode_sec<T>(sec_id: Section, sec_data: &Vec<T>) -> Vec<u8>
    where
        T: Encode,
    {
        let mut result: Vec<u8> = vec![];

        if !sec_data.is_empty() {
            result.push(sec_id as u8);
            result.extend(sec_data.encodes(true));
        }

        result
    }
}

impl Decode for Module {
    fn decode(reader: &mut Reader) -> DecodeResult<Module> {
        match reader.get_u32()? {
            MAGIC => (),
            magic => Err(DecodeErr::MagicUnMatch(magic))?,
        };
        match reader.get_u32()? {
            VERSION => (),
            version => Err(DecodeErr::VersionUnMatch(version))?,
        };

        let mut module = Module::new();
        let mut sec_counts: Vec<usize> = vec![0; 13];

        while reader.not_end()? {
            let i = reader.get_u8()? as usize;
            let id = Section::from_u8(i as u8)?;

            // 使用 onXXRead 回调方式实现
            if id != Section::Custom && sec_counts[i] >= 1 {
                Err(DecodeErr::MultipleSection(id.clone(), sec_counts[i]))?
            }

            let sec_data = reader.seqs()?;
            let mut sec_reader = Reader::new(&sec_data, module.data_counat_sec);

            sec_counts[i] = sec_counts[i] + 1;

            match id {
                Section::Custom => module.custom_sec.push(CustomSeg::decode(&mut sec_reader)?),
                Section::Type => module.type_sec = FuncType::decodes(&mut sec_reader)?,
                Section::Import => module.import_sec = ImportSeg::decodes(&mut sec_reader)?,
                Section::Function => module.func_sec = TypeIdx::decodes(&mut sec_reader)?,
                Section::Table => module.table_sec = TableType::decodes(&mut sec_reader)?,
                Section::Memory => module.mem_sec = MemType::decodes(&mut sec_reader)?,
                Section::Global => module.global_sec = GlobalSeg::decodes(&mut sec_reader)?,
                Section::Export => module.export_sec = ExportSeg::decodes(&mut sec_reader)?,
                Section::Start => module.start_sec = StartSeg::decode(&mut sec_reader)?,
                Section::Element => module.elem_sec = ElementSeg::decodes(&mut sec_reader)?,
                Section::Code => module.code_sec = CodeSeg::decodes(&mut sec_reader)?,
                Section::Data => module.data_sec = DataSeg::decodes(&mut sec_reader)?,
                Section::DataCount => module.data_counat_sec = DataCountSeg::decode(&mut sec_reader)?,
            };

            if sec_reader.remain().is_ok_and(|data| !data.is_empty()) {
                Err(DecodeErr::SectionSizeMismatch)?;
            }
        }

        module.validate_decode()?;

        Ok(module)
    }
}

impl Module {
    fn validate_decode(&self) -> DecodeResult<()> {
        if self.code_sec.len() != self.func_sec.len() {
            Err(DecodeErr::FuncAndCodeNotEq(
                self.code_sec.len(),
                self.func_sec.len(),
            ))?;
        }

        if let Some(count) = self.data_counat_sec {
            if self.data_sec.len() != (count as usize) {
                Err(DecodeErr::DataAndDataCountNotEq(
                    self.data_sec.len(),
                    count as usize,
                ))?
            }
        }

        Ok(())
    }
}

impl Encode for Module {
    fn encode(&self) -> Vec<u8> {
        let mut results = vec![];

        results.extend(MAGIC.to_le_bytes());
        results.extend(VERSION.to_le_bytes());

        results.extend(Module::encode_sec(Section::Type, &self.type_sec));
        results.extend(Module::encode_sec(Section::Import, &self.import_sec));
        results.extend(Module::encode_sec(Section::Function, &self.func_sec));
        results.extend(Module::encode_sec(Section::Table, &self.table_sec));
        results.extend(Module::encode_sec(Section::Memory, &self.mem_sec));
        results.extend(Module::encode_sec(Section::Global, &self.global_sec));
        results.extend(Module::encode_sec(Section::Export, &self.export_sec));
        results.extend(encode_maybeu32_sec(Section::Start, self.start_sec));
        results.extend(Module::encode_sec(Section::Element, &self.elem_sec));
        results.extend(encode_maybeu32_sec(Section::DataCount, self.data_counat_sec));
        results.extend(Module::encode_sec(Section::Code, &self.code_sec));
        results.extend(Module::encode_sec(Section::Data, &self.data_sec));
        results.extend(self.custom_sec.iter().flat_map(|custom| custom.encode()));

        results
    }
}

impl Validate for Module {
    fn validate(&self) -> ValidateResult {
        let module = self;

        Module::validates(&self.import_sec, module)?;
        Module::validates(&self.func_sec, module)?;
        Module::validates(&self.table_sec, module)?;
        Module::validates(&self.mem_sec, module)?;
        Module::validates(&self.global_sec, module)?;
        self.export_sec.validate_use_module(module)?;
        self.start_sec.validate_use_module(module)?;
        Module::validates(&self.elem_sec, module)?;
        Module::validates(&self.code_sec, module)?;
        Module::validates(&self.data_sec, module)?;

        Ok(())
    }
}
