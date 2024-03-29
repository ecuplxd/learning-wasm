use std::simd::ToBytes;

use crate::binary::instruction::{Lane16, Lane8};
use crate::binary::section::MaybeU32;
use crate::binary::types::MemType;
use crate::execution::errors::{InstError, VMState, Trap};
use crate::execution::value::v128;

pub const PAGE_SIZE: u32 = 65536;
pub const MAX_PAGE_SIZE: u32 = 65536;

pub trait Memory {
    fn alloc(size: u32) -> Vec<u8> {
        vec![0; size as usize]
    }

    fn mem_size(&self) -> u32;
    fn mem_grow(&mut self, size: u32) -> i32;

    fn mem_read(&self, addr: u64) -> VMState<u8> {
        let data = self.mem_reads(addr, 1)?;

        Ok(data[0])
    }

    fn mem_write(&mut self, addr: u64, data: u8) -> VMState {
        self.mem_writes(addr, &[data])
    }

    fn mem_reads(&self, addr: u64, n: u64) -> VMState<Vec<u8>>;

    fn mem_writes(&mut self, addr: u64, bytes: &[u8]) -> VMState;

    fn mem_read_8(&self, addr: u64) -> VMState<Lane8> {
        let mut bytes = [0u8; 8];
        let data = self.mem_reads(addr, 8)?;

        bytes.copy_from_slice(&data);

        Ok(bytes)
    }

    fn mem_read_16(&self, addr: u64) -> VMState<Lane16> {
        let mut bytes = [0u8; 16];
        let data = self.mem_reads(addr, 16)?;

        bytes.copy_from_slice(&data);

        Ok(bytes)
    }

    fn mem_read_i16(&self, addr: u64) -> VMState<i16> {
        let mut bytes = [0u8; 2];
        let data = self.mem_reads(addr, 2)?;

        bytes.copy_from_slice(&data);

        Ok(i16::from_le_bytes(bytes))
    }

    fn mem_read_i32(&self, addr: u64) -> VMState<i32> {
        let mut bytes = [0u8; 4];
        let data = self.mem_reads(addr, 4)?;

        bytes.copy_from_slice(&data);

        Ok(i32::from_le_bytes(bytes))
    }

    fn mem_read_i64(&self, addr: u64) -> VMState<i64> {
        let mut bytes = [0u8; 8];
        let data = self.mem_reads(addr, 8)?;

        bytes.copy_from_slice(&data);

        Ok(i64::from_le_bytes(bytes))
    }

    fn mem_read_f32(&self, addr: u64) -> VMState<f32> {
        let mut bytes = [0u8; 4];
        let data = self.mem_reads(addr, 4)?;

        bytes.copy_from_slice(&data);

        Ok(f32::from_le_bytes(bytes))
    }

    fn mem_read_f64(&self, addr: u64) -> VMState<f64> {
        let mut bytes = [0u8; 8];
        let data = self.mem_reads(addr, 8)?;

        bytes.copy_from_slice(&data);

        Ok(f64::from_le_bytes(bytes))
    }

    fn mem_read_v128(&self, addr: u64) -> VMState<v128> {
        let bytes = self.mem_read_16(addr)?;

        Ok(v128::new(bytes))
    }

    fn mem_write_v128(&mut self, addr: u64, v: v128) -> VMState {
        let bytes = v.as_i32x4().to_le_bytes().to_array();

        self.mem_writes(addr, &bytes)
    }
}

#[derive(Debug, Default)]
pub struct MemInst {
    type_: MemType,
    data: Vec<u8>,
}

impl MemInst {
    pub fn new(type_: MemType) -> Self {
        let init_size = type_.min * PAGE_SIZE;

        Self {
            type_,
            data: Self::alloc(init_size),
        }
    }

    pub fn get_type(&self) -> &MemType {
        &self.type_
    }

    pub fn max(&self) -> MaybeU32 {
        self.type_.max
    }

    pub fn copy(&mut self, addr: usize, n: usize, dest: usize) {
        self.data.copy_within(addr..addr + n, dest);
    }
}

impl Memory for MemInst {
    fn mem_read(&self, addr: u64) -> VMState<u8> {
        let addr = addr as usize;

        Ok(self.data[addr])
    }

    fn mem_reads(&self, addr: u64, n: u64) -> VMState<Vec<u8>> {
        let addr = addr as usize;
        let n = n as usize;

        if (addr + n) > self.data.len() {
            Err(Trap::OutofRange)?;
        }

        Ok(self.data[addr..addr + n].to_vec())
    }

    fn mem_writes(&mut self, addr: u64, bytes: &[u8]) -> VMState {
        let addr = addr as usize;
        let total = addr + bytes.len();

        if total > self.data.len() || total < addr {
            Err(InstError::OutofBoundMem)?;
        }

        let slice = &mut self.data[addr..total];

        slice.copy_from_slice(bytes);

        Ok(())
    }

    fn mem_size(&self) -> u32 {
        (self.data.len() as u32) / PAGE_SIZE
    }

    fn mem_grow(&mut self, size: u32) -> i32 {
        let old_size = self.mem_size();

        if size == 0 {
            return old_size as i32;
        }

        let new_size = old_size + size;
        let max = self.type_.max;

        match max {
            Some(max) if max < new_size => {
                println!("{} 超过该内存块可以分配的总上限：{}", new_size, max);

                return -1;
            }
            None if new_size > MAX_PAGE_SIZE => {
                println!("{} 超过可以分配的内存总上限：{}", new_size, MAX_PAGE_SIZE);

                return -1;
            }
            _ => self.data.resize((new_size * PAGE_SIZE) as usize, 0),
        }

        // 如果被其他模块导入，链接的时候将导致类型不匹配，所以需要进行变更
        self.type_.min += 1;

        old_size as i32
    }
}
