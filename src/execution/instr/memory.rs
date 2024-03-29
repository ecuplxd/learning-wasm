use crate::binary::instruction::MemoryArg;
use crate::execution::errors::{VMState, Trap};
use crate::execution::inst::memory::Memory;
use crate::execution::stack::operand::Operand;
use crate::execution::vm::VM;

impl VM {
    pub fn get_mem_addr(&mut self, memarg: &MemoryArg) -> u64 {
        // cast 到 u64，避免溢出，方便做越界检查
        let v = self.pop_u32() as u64;
        let offset = memarg.offset as u64;

        offset + v
    }
}

impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-load
    pub fn i32_load(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i32(addr)?;

        self.push_i32(data);

        Ok(())
    }

    pub fn i64_load(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i64(addr)?;

        self.push_i64(data);

        Ok(())
    }

    pub fn f32_load(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_f32(addr)?;

        self.push_f32(data);

        Ok(())
    }

    pub fn f64_load(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_f64(addr)?;

        self.push_f64(data);

        Ok(())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-loadn
    pub fn i32_load8_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read(addr)?;

        self.push_i32(data as i8 as i32);

        Ok(())
    }

    pub fn i32_load8_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read(addr)?;

        self.push_u32(data as u32);

        Ok(())
    }

    pub fn i32_load16_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i16(addr)?;

        self.push_i32(data as i32);

        Ok(())
    }

    pub fn i32_load16_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i16(addr)?;

        self.push_u32(data as u16 as u32);

        Ok(())
    }

    pub fn i64_load8_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read(addr)?;

        self.push_i64(data as i8 as i64);

        Ok(())
    }

    pub fn i64_load8_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read(addr)?;

        self.push_u64(data as u64);

        Ok(())
    }

    pub fn i64_load16_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i16(addr)?;

        self.push_i64(data as i64);

        Ok(())
    }

    pub fn i64_load16_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i16(addr)?;

        self.push_u64(data as u16 as u64);

        Ok(())
    }

    pub fn i64_load32_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i32(addr)?;

        self.push_i64(data as i64);

        Ok(())
    }

    pub fn i64_load32_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i32(addr)?;

        self.push_u64(data as u32 as u64);

        Ok(())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-store
    pub fn i32_store(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_i32();
        let addr = self.get_mem_addr(memarg);

        self.mem_writes(addr, &data.to_le_bytes())
    }

    pub fn i64_store(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_i64();
        let addr = self.get_mem_addr(memarg);

        self.mem_writes(addr, &data.to_le_bytes())
    }

    pub fn f32_store(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_f32();
        let addr = self.get_mem_addr(memarg);

        self.mem_writes(addr, &data.to_le_bytes())
    }

    pub fn f64_store(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_f64();
        let addr = self.get_mem_addr(memarg);

        self.mem_writes(addr, &data.to_le_bytes())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-storen
    pub fn i32_store8(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_i32();
        let addr = self.get_mem_addr(memarg);
        let bytes = data.to_le_bytes();

        self.mem_write(addr, bytes[0])
    }

    pub fn i32_store16(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_i32();
        let addr = self.get_mem_addr(memarg);
        let bytes = data.to_le_bytes();

        self.mem_writes(addr, &bytes[0..2])
    }

    pub fn i64_store8(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_i64();
        let addr = self.get_mem_addr(memarg);
        let bytes = data.to_le_bytes();

        self.mem_write(addr, bytes[0])
    }

    pub fn i64_store16(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_i64();
        let addr = self.get_mem_addr(memarg);
        let bytes = data.to_le_bytes();

        self.mem_writes(addr, &bytes[0..2])
    }

    pub fn i64_store32(&mut self, memarg: &MemoryArg) -> VMState {
        let data = self.pop_i64();
        let addr = self.get_mem_addr(memarg);
        let bytes = data.to_le_bytes();

        self.mem_writes(addr, &bytes[0..4])
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-memory-size
    /// mem_idx 默认 0，暂不使用
    pub fn memory_size(&mut self, idx: u8) {
        let size = self.mems[idx as usize].borrow().mem_size();

        self.push_u32(size);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-memory-grow
    /// mem_idx 默认 0，暂不使用
    pub fn memory_grow(&mut self, idx: u8) {
        let size = self.pop_u32();
        let old_size = self.mems[idx as usize].borrow_mut().mem_grow(size);

        self.push_i32(old_size);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-memory-init
    pub fn memory_init(&mut self, segment: u32, idx: u32) -> VMState {
        let n = self.pop_u32() as usize;
        let addr = self.pop_u32() as usize;
        let dst = self.pop_u32() as u64;

        let data = &self.datas[segment as usize];

        if (addr + n) > data.len() {
            Err(Trap::OutofRange)?;
        }

        let bytes = &data[addr..addr + n];

        self.mems[idx as usize].borrow_mut().mem_writes(dst, bytes)
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-data-drop
    pub fn data_drop(&mut self, data_idx: u32) {
        self.datas[data_idx as usize].clear();
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-memory-copy
    pub fn memory_copy(&mut self, src_idx: u32, dst_idx: u32) -> VMState {
        let n = self.pop_u32() as u64;
        let addr = self.pop_u32() as u64;
        let dest = self.pop_u32() as u64;

        let data = self.mems[src_idx as usize].borrow().mem_reads(addr, n)?;

        self.mems[dst_idx as usize].borrow_mut().mem_writes(dest, &data)
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-memory-fill
    pub fn memory_fill(&mut self, idx: u32) -> VMState {
        let n = self.pop_u32() as usize;
        let val = self.pop_u32() as u8;
        let addr = self.pop_u32() as u64;
        let data = vec![val; n];

        self.mems[idx as usize].borrow_mut().mem_writes(addr, &data)
    }
}
