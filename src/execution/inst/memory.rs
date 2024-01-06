use std::simd::ToBytes;

use crate::binary::instruction::{Lane16, Lane8};
use crate::binary::types::MemType;
use crate::execution::types::v128;

pub trait Memory {
    fn alloc(size: u32) -> Vec<u8> {
        vec![0; size as usize]
    }

    fn mem_size(&self) -> u32;
    fn mem_grow(&mut self, size: u32) -> i32;

    fn mem_read(&self, addr: u64) -> u8 {
        self.mem_reads(addr, 1)[0]
    }

    fn mem_write(&mut self, addr: u64, data: u8) {
        self.mem_writes(addr, &[data]);
    }

    fn mem_reads(&self, addr: u64, n: u64) -> Vec<u8>;

    fn mem_writes(&mut self, addr: u64, bytes: &[u8]);

    fn mem_read_8(&self, addr: u64) -> Lane8 {
        let mut bytes = [0u8; 8];
        let data = self.mem_reads(addr, 8);

        bytes.copy_from_slice(&data);

        bytes
    }

    fn mem_read_16(&self, addr: u64) -> Lane16 {
        let mut bytes = [0u8; 16];
        let data = self.mem_reads(addr, 16);

        bytes.copy_from_slice(&data);

        bytes
    }

    fn mem_read_i16(&self, addr: u64) -> i16 {
        let mut bytes = [0u8; 2];
        let data = self.mem_reads(addr, 2);

        bytes.copy_from_slice(&data);

        i16::from_le_bytes(bytes)
    }

    fn mem_read_i32(&self, addr: u64) -> i32 {
        let mut bytes = [0u8; 4];
        let data = self.mem_reads(addr, 4);

        bytes.copy_from_slice(&data);

        i32::from_le_bytes(bytes)
    }

    fn mem_read_i64(&self, addr: u64) -> i64 {
        let mut bytes = [0u8; 8];
        let data = self.mem_reads(addr, 8);

        bytes.copy_from_slice(&data);

        i64::from_le_bytes(bytes)
    }

    fn mem_read_f32(&self, addr: u64) -> f32 {
        let mut bytes = [0u8; 4];
        let data = self.mem_reads(addr, 4);

        bytes.copy_from_slice(&data);

        f32::from_le_bytes(bytes)
    }

    fn mem_read_f64(&self, addr: u64) -> f64 {
        let mut bytes = [0u8; 8];
        let data = self.mem_reads(addr, 8);

        bytes.copy_from_slice(&data);

        f64::from_le_bytes(bytes)
    }

    fn mem_read_v128(&self, addr: u64) -> v128 {
        let bytes = self.mem_read_16(addr);

        v128::new(bytes)
    }

    fn mem_write_v128(&mut self, addr: u64, v: v128) {
        let bytes = v.as_i32x4().to_le_bytes().to_array();

        self.mem_writes(addr, &bytes);
    }
}

pub const PAGE_SIZE: u32 = 65536;
pub const MAX_PAGE_SIZE: u32 = 65536;

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

    pub fn max(&self) -> Option<u32> {
        self.type_.max
    }

    pub fn copy(&mut self, addr: usize, n: usize, dest: usize) {
        self.data.copy_within(addr..addr + n, dest);
    }
}

impl Memory for MemInst {
    fn mem_read(&self, addr: u64) -> u8 {
        self.data[addr as usize]
    }

    fn mem_reads(&self, addr: u64, n: u64) -> Vec<u8> {
        let addr = addr as usize;
        let n = n as usize;

        self.data[addr..addr + n].to_vec()
    }

    fn mem_writes(&mut self, addr: u64, bytes: &[u8]) {
        let addr = addr as usize;
        let slice = &mut self.data[addr..addr + bytes.len()];

        slice.copy_from_slice(bytes);
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

        old_size as i32
    }
}
