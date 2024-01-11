use std::simd::cmp::SimdOrd;
use std::simd::num::{SimdFloat, SimdInt, SimdUint};
use std::simd::{f32x4, f64x2, i16x8, i32x4, i64x2, i8x16, u16x8, u32x4, u64x2, u8x16, StdFloat};

use super::trunc_sat::{trunc_sat_s, trunc_sat_u, TruncSize};
use crate::binary::instruction::{Lane16, LaneIdx, MemoryArg};
use crate::execution::errors::VMState;
use crate::execution::inst::memory::Memory;
use crate::execution::stack::operand::Operand;
use crate::execution::value::{v128, ToV128};
use crate::execution::vm::VM;

macro_rules! saturate {
    ($val:expr, $src:ty => $target:ty) => {{
        let min = <$target>::MIN;
        let max = <$target>::MAX;

        match $val > (max as $src) {
            true => max,
            false if $val < (min as $src) => min,
            _ => $val as $target,
        }
    }};
}

impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-load
    pub fn v128_load(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let v = self.mem_read_v128(addr)?;

        self.push_v128(v);

        Ok(())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-load-extend
    pub fn v128_load8x8_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_8(addr)?;
        let mut v = i16x8::splat(0);

        for i in 0..v.len() {
            v[i] = data[i] as i8 as i16;
        }

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load8x8_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_8(addr)?;
        let mut v = u16x8::splat(0);

        for i in 0..v.len() {
            v[i] = data[i] as u16;
        }

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load16x4_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = [
            self.mem_read_i16(addr)? as i32,
            self.mem_read_i16(addr + 2)? as i32,
            self.mem_read_i16(addr + 4)? as i32,
            self.mem_read_i16(addr + 6)? as i32,
        ];
        let v = v128(data[0], data[1], data[2], data[3]);

        self.push_v128(v);

        Ok(())
    }

    pub fn v128_load16x4_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = [
            self.mem_read_i16(addr)? as u16 as u32,
            self.mem_read_i16(addr + 2)? as u16 as u32,
            self.mem_read_i16(addr + 4)? as u16 as u32,
            self.mem_read_i16(addr + 6)? as u16 as u32,
        ];
        let v = u32x4::from_array(data);

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load32x2_s(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = [
            self.mem_read_i32(addr)? as i64,
            self.mem_read_i32(addr + 4)? as i64,
        ];
        let v = i64x2::from_array(data);

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load32x2_u(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = [
            self.mem_read_i32(addr)? as u32 as u64,
            self.mem_read_i32(addr + 4)? as u32 as u64,
        ];
        let v = u64x2::from_array(data);

        self.push_v128(v.v128());

        Ok(())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-load-splat
    pub fn v128_load8_splat(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let byte = self.mem_read(addr)?;
        let v = u8x16::splat(byte);

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load16_splat(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i16(addr)?;
        let v = i16x8::splat(data);

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load32_splat(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i32(addr)?;
        let v = i32x4::splat(data);

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load64_splat(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i64(addr)?;
        let v = i64x2::splat(data);

        self.push_v128(v.v128());

        Ok(())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-load-lane
    pub fn v128_load8_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let mut v = self.pop_v128().as_u8x16();
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read(addr)?;

        v[laneidx as usize] = data;

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load16_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let mut v = self.pop_v128().as_u16x8();
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i16(addr)?;

        v[laneidx as usize] = data as u16;

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load32_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let mut v = self.pop_v128().as_u32x4();
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i32(addr)?;

        v[laneidx as usize] = data as u32;

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load64_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let mut v = self.pop_v128().as_u64x2();
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i64(addr)?;

        v[laneidx as usize] = data as u64;

        self.push_v128(v.v128());

        Ok(())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-load-zero
    pub fn v128_load32_zero(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i32(addr)? as u32;
        let v = u32x4::from_array([data, 0, 0, 0]);

        self.push_v128(v.v128());

        Ok(())
    }

    pub fn v128_load64_zero(&mut self, memarg: &MemoryArg) -> VMState {
        let addr = self.get_mem_addr(memarg);
        let data = self.mem_read_i64(addr)? as u64;
        let v = u64x2::from_array([data, 0]);

        self.push_v128(v.v128());

        Ok(())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-splat
    pub fn i8x16_splat(&mut self) {
        let data = self.pop_i32() as i8;
        let v = i8x16::splat(data);

        self.push_v128(v.v128());
    }

    pub fn i16x8_splat(&mut self) {
        let data = self.pop_i32() as i16;
        let v = i16x8::splat(data);

        self.push_v128(v.v128());
    }

    pub fn i32x4_splat(&mut self) {
        let data = self.pop_i32();
        let v = i32x4::splat(data);

        self.push_v128(v.v128());
    }

    pub fn i64x2_splat(&mut self) {
        let data = self.pop_i64();
        let v = i64x2::splat(data);

        self.push_v128(v.v128());
    }

    pub fn f32x4_splat(&mut self) {
        let data = self.pop_f32();
        let v = f32x4::splat(data);

        self.push_v128(v.v128());
    }

    pub fn f64x2_splat(&mut self) {
        let data = self.pop_f64();
        let v = f64x2::splat(data);

        self.push_v128(v.v128())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-store
    pub fn v128_store(&mut self, memarg: &MemoryArg) -> VMState {
        let v = self.pop_v128();
        let addr = self.get_mem_addr(memarg);

        self.mem_write_v128(addr, v)
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-store-lane
    pub fn v128_store8_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let v = self.pop_v128().as_u8x16();
        let addr = self.get_mem_addr(memarg);
        let lane_data = v[laneidx as usize];

        self.mem_write(addr, lane_data)
    }

    pub fn v128_store16_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let v = self.pop_v128().as_u16x8();
        let addr = self.get_mem_addr(memarg);
        let lane_data = v[laneidx as usize];

        self.mem_writes(addr, &lane_data.to_le_bytes())
    }

    pub fn v128_store32_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let v = self.pop_v128().as_u32x4();
        let addr = self.get_mem_addr(memarg);
        let lane_data = v[laneidx as usize];

        self.mem_writes(addr, &lane_data.to_le_bytes())
    }

    pub fn v128_store64_lane(&mut self, memarg: &MemoryArg, laneidx: LaneIdx) -> VMState {
        let v = self.pop_v128().as_u64x2();
        let addr = self.get_mem_addr(memarg);
        let lane_data = v[laneidx as usize];

        self.mem_writes(addr, &lane_data.to_le_bytes())
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vconst
    pub fn v128_const(&mut self, v: v128) {
        self.push_v128(v);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-shuffle
    pub fn i8x16_shuffle(&mut self, lane_16: Lane16) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut new = i8x16::splat(0);

        for i in 0..16 {
            let idx = lane_16[i] as usize;

            new[i] = match idx < 16 {
                true => v1[idx],
                false => v2[idx - 16],
            };
        }

        self.push_v128(new.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-swizzle
    pub fn i8x16_swizzle(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u8x16::splat(0);

        for i in 0..16 {
            v[i] = match v2[i] < 16 {
                true => v1[v2[i] as usize],
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-extract-lane
    pub fn i8x16_extract_lane_s(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_i8x16();
        let data = v[laneidx as usize];

        self.push_i32(data as i32);
    }

    pub fn i8x16_extract_lane_u(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_u8x16();
        let data = v[laneidx as usize];

        self.push_u32(data as u32);
    }

    pub fn i16x8_extract_lane_s(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_i16x8();
        let data = v[laneidx as usize];

        self.push_i32(data as i32);
    }

    pub fn i16x8_extract_lane_u(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_u16x8();
        let data = v[laneidx as usize];

        self.push_u32(data as u32);
    }

    pub fn i32x4_extract_lane(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_i32x4();
        let data = v[laneidx as usize];

        self.push_i32(data);
    }

    pub fn i64x2_extract_lane(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_i64x2();
        let data = v[laneidx as usize];

        self.push_i64(data);
    }

    pub fn f32x4_extract_lane(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_f32x4();
        let data = v[laneidx as usize];

        self.push_f32(data);
    }

    pub fn f64x2_extract_lane(&mut self, laneidx: LaneIdx) {
        let v = self.pop_v128().as_f64x2();
        let data = v[laneidx as usize];

        self.push_f64(data);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-replace-lane
    pub fn i8x16_replace_lane(&mut self, laneidx: LaneIdx) {
        let data = self.pop_i32();
        let mut v = self.pop_v128().as_i8x16();

        v[laneidx as usize] = data as i8;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-replace-lane
    pub fn i16x8_replace_lane(&mut self, laneidx: LaneIdx) {
        let data = self.pop_i32();
        let mut v = self.pop_v128().as_i16x8();

        v[laneidx as usize] = data as i16;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-replace-lane
    pub fn i32x4_replace_lane(&mut self, laneidx: LaneIdx) {
        let data = self.pop_i32();
        let mut v = self.pop_v128().as_i32x4();

        v[laneidx as usize] = data;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-replace-lane
    pub fn i64x2_replace_lane(&mut self, laneidx: LaneIdx) {
        let data = self.pop_i64();
        let mut v = self.pop_v128().as_i64x2();

        v[laneidx as usize] = data;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-replace-lane
    pub fn f32x4_replace_lane(&mut self, laneidx: LaneIdx) {
        let data = self.pop_f32();
        let mut v = self.pop_v128().as_f32x4();

        v[laneidx as usize] = data;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-replace-lane
    pub fn f64x2_replace_lane(&mut self, laneidx: LaneIdx) {
        let data = self.pop_f64();
        let mut v = self.pop_v128().as_f64x2();

        v[laneidx as usize] = data;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i8x16_eq(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] == v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i8x16_ne(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] != v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i8x16_lt_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i8x16_lt_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u8>().v128());
    }

    pub fn i8x16_gt_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i8x16_gt_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u8>().v128());
    }

    pub fn i8x16_le_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i8x16_le_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u8>().v128());
    }

    pub fn i8x16_ge_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i8x16_ge_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u8>().v128());
    }

    pub fn i16x8_eq(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] == v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_ne(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] != v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_lt_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_lt_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u16>().v128());
    }

    pub fn i16x8_gt_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_gt_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u16>().v128());
    }

    pub fn i16x8_le_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_le_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u16>().v128());
    }

    pub fn i16x8_ge_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_ge_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u16>().v128());
    }

    pub fn i32x4_eq(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] == v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_ne(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] != v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_lt_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_lt_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u32>().v128());
    }

    pub fn i32x4_gt_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_gt_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u32>().v128());
    }

    pub fn i32x4_le_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_le_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u32>().v128());
    }

    pub fn i32x4_ge_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_ge_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1,
                false => 0,
            }
        }

        self.push_v128(v.cast::<u32>().v128());
    }

    pub fn f32x4_eq(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] == v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i32>().v128());
    }

    pub fn f32x4_ne(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] != v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i32>().v128());
    }

    pub fn f32x4_lt(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i32>().v128());
    }

    pub fn f32x4_gt(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i32>().v128());
    }

    pub fn f32x4_le(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i32>().v128());
    }

    pub fn f32x4_ge(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i32>().v128());
    }

    pub fn f64x2_eq(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] == v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn f64x2_ne(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] != v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn f64x2_lt(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn f64x2_gt(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn f64x2_le(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn f64x2_ge(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vvunop
    pub fn v128_not(&mut self) {
        let v1 = self.pop_v128().as_i64x2();
        let v = !v1;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vvbinop
    pub fn v128_and(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 & v2;

        self.push_v128(v.v128());
    }

    pub fn v128_andnot(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 & (!v2);

        self.push_v128(v.v128());
    }

    pub fn v128_or(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 | v2;

        self.push_v128(v.v128());
    }

    pub fn v128_xor(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 ^ v2;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vvternop
    pub fn v128_bitselect(&mut self) {
        let v3 = self.pop_v128().as_i8x16();
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();

        let mut v = i8x16::splat(0);

        for i in 0..v3.len() {
            v[i] = (v1[i] & v3[i]) | (v2[i] & (!v3[i]));
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vvtestop
    pub fn v128_any_true(&mut self) {
        let v = self.pop_v128().as_u8x16();
        let any_true = v.as_array().iter().any(|data| *data != 0);

        self.push_bool(any_true);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vcvtop
    pub fn f32x4_demote_f64x2_zero(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = v1[i] as f32;
        }

        for i in (v1.len()..v.len()).rev() {
            v[i] = 0.0;
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn i8x16_abs(&mut self) {
        let v1 = self.pop_v128().as_i8x16();
        let v = v1.abs();

        self.push_v128(v.v128());
    }

    pub fn i8x16_neg(&mut self) {
        let v1 = self.pop_v128().as_i8x16();
        let v = -v1;

        self.push_v128(v.v128());
    }

    pub fn i8x16_popcnt(&mut self) {
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = v1[i].count_ones() as u8;
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vtestop
    pub fn i8x16_all_true(&mut self) {
        let v = self.pop_v128().as_i8x16();
        let all_true = v.as_array().iter().all(|data| *data != 0);

        self.push_bool(all_true);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-bitmask
    pub fn i8x16_bitmask(&mut self) {
        let v1 = self.pop_v128().as_i8x16();
        let mut result = 0;

        for i in 0..v1.len() {
            if v1[i] < 0 {
                result |= 1 << i;
            }
        }

        self.push_i32(result);
    }

    pub fn i16x8_bitmask(&mut self) {
        let v1 = self.pop_v128().as_i16x8();
        let mut result = 0;

        for i in 0..v1.len() {
            if v1[i] < 0 {
                result |= 1 << i;
            }
        }

        self.push_i32(result);
    }

    pub fn i32x4_bitmask(&mut self) {
        let v1 = self.pop_v128().as_i32x4();
        let mut result = 0;

        for i in 0..v1.len() {
            if v1[i] < 0 {
                result |= 1 << i;
            }
        }

        self.push_i32(result);
    }

    pub fn i64x2_bitmask(&mut self) {
        let v1 = self.pop_v128().as_i64x2();
        let mut result = 0;

        for i in 0..v1.len() {
            if v1[i] < 0 {
                result |= 1 << i;
            }
        }

        self.push_i32(result);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-narrow
    pub fn i8x16_narrow_i16x8_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = saturate!(v1[i], i16 => i8);
        }

        for i in 0..v1.len() {
            v[v1.len() + i] = saturate!(v2[i], i16 => i8);
        }

        self.push_v128(v.v128());
    }

    pub fn i8x16_narrow_i16x8_u(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = u8x16::splat(0);

        for i in 0..v1.len() {
            v[i] = saturate!(v1[i], i16 => u8);
        }

        for i in 0..v1.len() {
            v[v1.len() + i] = saturate!(v2[i], i16 => u8);
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_narrow_i32x4_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = saturate!(v1[i], i32 => i16);
        }

        for i in 0..v1.len() {
            v[v1.len() + i] = saturate!(v2[i], i32 => i16);
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_narrow_i32x4_u(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = u16x8::splat(0);

        for i in 0..v1.len() {
            v[i] = saturate!(v1[i], i32 => u16);
        }

        for i in 0..v1.len() {
            v[v1.len() + i] = saturate!(v2[i], i32 => u16);
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn f32x4_ceil(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let v = v1.ceil();

        self.push_v128(v.v128());
    }

    pub fn f32x4_floor(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let v = v1.floor();

        self.push_v128(v.v128());
    }

    pub fn f32x4_trunc(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let v = v1.trunc();

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vishiftop
    pub fn i8x16_shl(&mut self) {
        let shift = self.pop_i32() as i8;
        let v1 = self.pop_v128().as_i8x16();
        let v = v1 << shift;

        self.push_v128(v.v128());
    }

    pub fn i8x16_shr_s(&mut self) {
        let shift = self.pop_i32() as i8;
        let v1 = self.pop_v128().as_i8x16();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    pub fn i8x16_shr_u(&mut self) {
        let shift = self.pop_u32() as u8;
        let v1 = self.pop_v128().as_u8x16();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    pub fn i16x8_shl(&mut self) {
        let shift = self.pop_i32() as i16;
        let v1 = self.pop_v128().as_i16x8();
        let v = v1 << shift;

        self.push_v128(v.v128());
    }

    pub fn i16x8_shr_s(&mut self) {
        let shift = self.pop_i32() as i16;
        let v1 = self.pop_v128().as_i16x8();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    pub fn i16x8_shr_u(&mut self) {
        let shift = self.pop_u32() as u16;
        let v1 = self.pop_v128().as_u16x8();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    pub fn i32x4_shl(&mut self) {
        let shift = self.pop_i32();
        let v1 = self.pop_v128().as_i32x4();
        let v = v1 << shift;

        self.push_v128(v.v128());
    }

    pub fn i32x4_shr_s(&mut self) {
        let shift = self.pop_i32();
        let v1 = self.pop_v128().as_i32x4();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    pub fn i32x4_shr_u(&mut self) {
        let shift = self.pop_u32();
        let v1 = self.pop_v128().as_u32x4();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    pub fn i64x2_shl(&mut self) {
        let shift = self.pop_i32() as i64;
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 << shift;

        self.push_v128(v.v128());
    }

    pub fn i64x2_shr_s(&mut self) {
        let shift = self.pop_i32() as i64;
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    pub fn i64x2_shr_u(&mut self) {
        let shift = self.pop_u32() as u64;
        let v1 = self.pop_v128().as_u64x2();
        let v = v1 >> shift;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i8x16_add(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let v = v1 + v2;

        self.push_v128(v.v128());
    }

    pub fn i8x16_add_sat_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let v = v1.saturating_add(v2);

        self.push_v128(v.v128());
    }

    pub fn i8x16_add_sat_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let v = v1.saturating_add(v2);

        self.push_v128(v.v128());
    }

    pub fn i8x16_sub(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let v = v1 - v2;

        self.push_v128(v.v128());
    }

    pub fn i8x16_sub_sat_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let v = v1.saturating_sub(v2);

        self.push_v128(v.v128());
    }

    pub fn i8x16_sub_sat_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let v = v1.saturating_sub(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn f64x2_ceil(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let v = v1.ceil();

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn f64x2_floor(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let v = v1.floor();

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i8x16_min_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let v = v1.simd_min(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i8x16_min_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let v = v1.simd_min(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i8x16_max_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let v = v1.simd_max(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i8x16_max_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let v = v1.simd_max(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn f64x2_trunc(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let v = v1.trunc();

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-extadd-pairwise
    pub fn i16x8_extadd_pairwise_i8x16_s(&mut self) {
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i16x8::splat(0);

        for i in 0..v.len() {
            let lane_idx = i * 2;

            v[i] = (v1[lane_idx] as i16) + (v1[lane_idx + 1] as i16);
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_extadd_pairwise_i8x16_u(&mut self) {
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u16x8::splat(0);

        for i in 0..v.len() {
            let lane_idx = i * 2;

            v[i] = (v1[lane_idx] as u16) + (v1[lane_idx + 1] as u16);
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extadd_pairwise_i16x8_s(&mut self) {
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i32x4::splat(0);

        for i in 0..v.len() {
            let lane_idx = i * 2;

            v[i] = (v1[lane_idx] as i32) + (v1[lane_idx + 1] as i32);
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extadd_pairwise_i16x8_u(&mut self) {
        let v1 = self.pop_v128().as_u16x8();
        let mut v = u32x4::splat(0);

        for i in 0..v.len() {
            let lane_idx = i * 2;

            v[i] = (v1[lane_idx] as u32) + (v1[lane_idx + 1] as u32);
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn i16x8_abs(&mut self) {
        let v1 = self.pop_v128().as_i16x8();
        let v = v1.abs();

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn i16x8_neg(&mut self) {
        let v1 = self.pop_v128().as_i16x8();
        let v = -v1;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i16x8_q15mulr_sat_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i16x8::splat(0);
        const SIZE_IN_BITS: i8 = 16;

        for i in 0..v.len() {
            let a = v1[i] as i64;
            let b = v2[i] as i64;
            let round_const = 1 << (SIZE_IN_BITS - 2);
            let mut product = a * b + round_const;

            product >>= SIZE_IN_BITS - 1;

            v[i] = saturate!(product, i64 => i16);
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vtestop
    pub fn i16x8_all_true(&mut self) {
        let v = self.pop_v128().as_i16x8();
        let all_true = v.as_array().iter().all(|data| *data != 0);

        self.push_bool(all_true);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vcvtop
    pub fn i16x8_extend_low_i8x16_s(&mut self) {
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i16x8::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i] as i16;
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_extend_high_i8x16_s(&mut self) {
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i16x8::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i + v.len()] as i16;
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_extend_low_i8x16_u(&mut self) {
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u16x8::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i] as u16;
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_extend_high_i8x16_u(&mut self) {
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u16x8::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i + v.len()] as u16;
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extend_low_i16x8_s(&mut self) {
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i32x4::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i] as i32;
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extend_high_i16x8_s(&mut self) {
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i32x4::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i + v.len()] as i32;
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extend_low_i16x8_u(&mut self) {
        let v1 = self.pop_v128().as_u16x8();
        let mut v = u32x4::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i] as u32;
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extend_high_i16x8_u(&mut self) {
        let v1 = self.pop_v128().as_u16x8();
        let mut v = u32x4::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i + v.len()] as u32;
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extend_low_i32x4_s(&mut self) {
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i64x2::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i] as i64;
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extend_high_i32x4_s(&mut self) {
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i64x2::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i + v.len()] as i64;
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extend_low_i32x4_u(&mut self) {
        let v1 = self.pop_v128().as_u32x4();
        let mut v = u64x2::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i] as u64;
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extend_high_i32x4_u(&mut self) {
        let v1 = self.pop_v128().as_u32x4();
        let mut v = u64x2::splat(0);

        for i in 0..v.len() {
            v[i] = v1[i + v.len()] as u64;
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i16x8_add(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let v = v1 + v2;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i16x8_add_sat_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let v = v1.saturating_add(v2);

        self.push_v128(v.v128());
    }

    pub fn i16x8_add_sat_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let v = v1.saturating_add(v2);

        self.push_v128(v.v128());
    }

    pub fn i16x8_sub(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let v = v1 - v2;

        self.push_v128(v.v128());
    }

    pub fn i16x8_sub_sat_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let v = v1.saturating_sub(v2);

        self.push_v128(v.v128());
    }

    pub fn i16x8_sub_sat_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let v = v1.saturating_sub(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn f32x4_nearest(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v.len() {
            v[i] = v1[i].round_ties_even();
        }

        self.push_v128(v.v128());
    }

    pub fn f64x2_nearest(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v.len() {
            v[i] = v1[i].round_ties_even();
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i16x8_mul(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let v = v1 * v2;

        self.push_v128(v.v128());
    }

    pub fn i16x8_min_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let v = v1.simd_min(v2);

        self.push_v128(v.v128());
    }

    pub fn i16x8_min_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let v = v1.simd_min(v2);

        self.push_v128(v.v128());
    }

    pub fn i16x8_max_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let v = v1.simd_max(v2);

        self.push_v128(v.v128());
    }

    pub fn i16x8_max_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let v = v1.simd_max(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i8x16_avgr_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u8x16::splat(0);

        for i in 0..v.len() {
            let a = v1[i] as u16;
            let b = v2[i] as u16;

            v[i] = ((a + b + 1) / 2) as u8;
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_avgr_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let mut v = u16x8::splat(0);

        for i in 0..v.len() {
            let a = v1[i] as u32;
            let b = v2[i] as u32;

            v[i] = ((a + b + 1) / 2) as u16;
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-extmul
    pub fn i16x8_extmul_low_i8x16_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i16x8::splat(0);

        for i in 0..v.len() {
            v[i] = (v1[i] as i16) * (v2[i] as i16);
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_extmul_high_i8x16_s(&mut self) {
        let v2 = self.pop_v128().as_i8x16();
        let v1 = self.pop_v128().as_i8x16();
        let mut v = i16x8::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as i16) * (v2[lane_idx] as i16);
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_extmul_low_i8x16_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u16x8::splat(0);

        for i in 0..v.len() {
            v[i] = (v1[i] as u16) * (v2[i] as u16);
        }

        self.push_v128(v.v128());
    }

    pub fn i16x8_extmul_high_i8x16_u(&mut self) {
        let v2 = self.pop_v128().as_u8x16();
        let v1 = self.pop_v128().as_u8x16();
        let mut v = u16x8::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as u16) * (v2[lane_idx] as u16);
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extmul_low_i16x8_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i32x4::splat(0);

        for i in 0..v.len() {
            v[i] = (v1[i] as i32) * (v2[i] as i32);
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extmul_high_i16x8_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i32x4::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as i32) * (v2[lane_idx] as i32);
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extmul_low_i16x8_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let mut v = u32x4::splat(0);

        for i in 0..v.len() {
            v[i] = (v1[i] as u32) * (v2[i] as u32);
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_extmul_high_i16x8_u(&mut self) {
        let v2 = self.pop_v128().as_u16x8();
        let v1 = self.pop_v128().as_u16x8();
        let mut v = u32x4::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as u32) * (v2[lane_idx] as u32);
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extmul_low_i32x4_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i64x2::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as i64) * (v2[lane_idx] as i64);
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extmul_high_i32x4_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let mut v = i64x2::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as i64) * (v2[lane_idx] as i64);
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extmul_low_i32x4_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let mut v = u64x2::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as u64) * (v2[lane_idx] as u64);
        }

        self.push_v128(v.v128());
    }

    pub fn i64x2_extmul_high_i32x4_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let mut v = u64x2::splat(0);

        for i in 0..v.len() {
            let lane_idx = v.len() + i;

            v[i] = (v1[lane_idx] as u64) * (v2[lane_idx] as u64);
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn i32x4_abs(&mut self) {
        let v1 = self.pop_v128().as_i32x4();
        let v = v1.abs();

        self.push_v128(v.v128());
    }

    pub fn i32x4_neg(&mut self) {
        let v1 = self.pop_v128().as_i32x4();
        let v = -v1;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vtestop
    pub fn i32x4_all_true(&mut self) {
        let v = self.pop_v128().as_i32x4();
        let all_true = v.as_array().iter().all(|data| *data != 0);

        self.push_bool(all_true);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i32x4_add(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let v = v1 + v2;

        self.push_v128(v.v128());
    }

    pub fn i32x4_sub(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let v = v1 - v2;

        self.push_v128(v.v128());
    }

    pub fn i32x4_mul(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let v = v1 * v2;

        self.push_v128(v.v128());
    }

    pub fn i32x4_min_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let v = v1.simd_min(v2);

        self.push_v128(v.v128());
    }

    pub fn i32x4_min_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let v = v1.simd_min(v2);

        self.push_v128(v.v128());
    }

    pub fn i32x4_max_s(&mut self) {
        let v2 = self.pop_v128().as_i32x4();
        let v1 = self.pop_v128().as_i32x4();
        let v = v1.simd_max(v2);

        self.push_v128(v.v128());
    }

    pub fn i32x4_max_u(&mut self) {
        let v2 = self.pop_v128().as_u32x4();
        let v1 = self.pop_v128().as_u32x4();
        let v = v1.simd_max(v2);

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vec-dot
    pub fn i32x4_dot_i16x8_s(&mut self) {
        let v2 = self.pop_v128().as_i16x8();
        let v1 = self.pop_v128().as_i16x8();
        let mut v = i32x4::splat(0);

        for i in 0..v.len() {
            let lane_idx = i * 2;
            let lo = (v1[lane_idx] as i32) * (v2[lane_idx] as i32);
            let hi = (v1[lane_idx + 1] as i32) * (v2[lane_idx + 1] as i32);

            v[i] = lo + hi;
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn i64x2_abs(&mut self) {
        let v1 = self.pop_v128().as_i64x2();
        let v = v1.abs();

        self.push_v128(v.v128());
    }

    pub fn i64x2_neg(&mut self) {
        let v1 = self.pop_v128().as_i64x2();
        let v = -v1;

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vtestop
    pub fn i64x2_all_true(&mut self) {
        let v = self.pop_v128().as_i64x2();
        let all_true = v.as_array().iter().all(|data| *data != 0);

        self.push_bool(all_true);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn i64x2_add(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 + v2;

        self.push_v128(v.v128());
    }

    pub fn i64x2_sub(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 - v2;

        self.push_v128(v.v128());
    }

    pub fn i64x2_mul(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let v = v1 * v2;

        self.push_v128(v.v128());
    }

    pub fn i64x2_eq(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] == v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn i64x2_ne(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] != v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn i64x2_lt_s(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] < v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn i64x2_gt_s(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] > v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn i64x2_le_s(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] <= v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    pub fn i64x2_ge_s(&mut self) {
        let v2 = self.pop_v128().as_i64x2();
        let v1 = self.pop_v128().as_i64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i] >= v2[i] {
                true => -1.0,
                false => 0.0,
            }
        }

        self.push_v128(v.cast::<i64>().v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn f32x4_abs(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let v = v1.abs();

        self.push_v128(v.v128());
    }

    pub fn f32x4_neg(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let v = -v1;

        self.push_v128(v.v128());
    }

    pub fn f32x4_sqrt(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let v = v1.sqrt();

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn f32x4_add(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let v = v1 + v2;

        self.push_v128(v.v128());
    }

    pub fn f32x4_sub(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let v = v1 - v2;

        self.push_v128(v.v128());
    }

    pub fn f32x4_mul(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let v = v1 * v2;

        self.push_v128(v.v128());
    }

    pub fn f32x4_div(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let v = v1 / v2;

        self.push_v128(v.v128());
    }

    pub fn f32x4_min(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            match v1[i].is_nan() {
                true => v[i] = v1[i],
                false if v2[i].is_nan() => v[i] = v2[i],
                _ => v[i] = v1[i].min(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    pub fn f32x4_max(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            match v1[i].is_nan() {
                true => v[i] = v1[i],
                false if v2[i].is_nan() => v[i] = v2[i],
                _ => v[i] = v1[i].max(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    pub fn f32x4_pmin(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i].is_nan() || v2[i].is_nan() {
                true => v1[i],
                _ => v1[i].min(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    pub fn f32x4_pmax(&mut self) {
        let v2 = self.pop_v128().as_f32x4();
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f32x4::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i].is_nan() || v2[i].is_nan() {
                true => v1[i],
                _ => v1[i].max(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vunop
    pub fn f64x2_abs(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let v = v1.abs();

        self.push_v128(v.v128());
    }

    pub fn f64x2_neg(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let v = -v1;

        self.push_v128(v.v128());
    }

    pub fn f64x2_sqrt(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let v = v1.sqrt();

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vbinop
    pub fn f64x2_add(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let v = v1 + v2;

        self.push_v128(v.v128());
    }

    pub fn f64x2_sub(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let v = v1 - v2;

        self.push_v128(v.v128());
    }

    pub fn f64x2_mul(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let v = v1 * v2;

        self.push_v128(v.v128());
    }

    pub fn f64x2_div(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let v = v1 / v2;

        self.push_v128(v.v128());
    }

    pub fn f64x2_min(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            match v1[i].is_nan() {
                true => v[i] = v1[i],
                false if v2[i].is_nan() => v[i] = v2[i],
                _ => v[i] = v1[i].min(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    pub fn f64x2_max(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            match v1[i].is_nan() {
                true => v[i] = v1[i],
                false if v2[i].is_nan() => v[i] = v2[i],
                _ => v[i] = v1[i].max(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    pub fn f64x2_pmin(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i].is_nan() || v2[i].is_nan() {
                true => v1[i],
                _ => v1[i].min(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    pub fn f64x2_pmax(&mut self) {
        let v2 = self.pop_v128().as_f64x2();
        let v1 = self.pop_v128().as_f64x2();
        let mut v = f64x2::splat(0.0);

        for i in 0..v1.len() {
            v[i] = match v1[i].is_nan() || v2[i].is_nan() {
                true => v1[i],
                _ => v1[i].max(v2[i]),
            }
        }

        self.push_v128(v.v128());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-vcvtop
    pub fn i32x4_trunc_sat_f32x4_s(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = trunc_sat_s(v1[i] as f64, TruncSize::N32) as i32;
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_trunc_sat_f32x4_u(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let mut v = u32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = trunc_sat_u(v1[i] as f64, TruncSize::N32) as u32;
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_trunc_sat_f64x2_s_zero(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let mut v = i32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = trunc_sat_s(v1[i], TruncSize::N32) as i32;
        }

        for i in (v1.len()..v.len()).rev() {
            v[i] = 0;
        }

        self.push_v128(v.v128());
    }

    pub fn i32x4_trunc_sat_f64x2_u_zero(&mut self) {
        let v1 = self.pop_v128().as_f64x2();
        let mut v = u32x4::splat(0);

        for i in 0..v1.len() {
            v[i] = trunc_sat_u(v1[i], TruncSize::N32) as u32;
        }

        for i in (v1.len()..v.len()).rev() {
            v[i] = 0;
        }

        self.push_v128(v.v128());
    }

    pub fn f32x4_convert_i32x4_s(&mut self) {
        let v1 = self.pop_v128().as_i32x4();
        let v = v1.cast::<f32>();

        self.push_v128(v.v128());
    }

    pub fn f32x4_convert_i32x4_u(&mut self) {
        let v1 = self.pop_v128().as_u32x4();
        let v = v1.cast::<f32>();

        self.push_v128(v.v128());
    }

    pub fn f64x2_promote_low_f32x4(&mut self) {
        let v1 = self.pop_v128().as_f32x4();
        let mut v = f64x2::splat(0.0);

        for i in 0..v.len() {
            let data = v1[i] as f64;

            v[i] = data;
        }

        self.push_v128(v.v128());
    }

    pub fn f64x2_convert_low_i32x4_s(&mut self) {
        let v1 = self.pop_v128().as_i32x4();
        let mut v = f64x2::splat(0.0);

        for i in 0..v.len() {
            v[i] = v1[i] as f64;
        }

        self.push_v128(v.v128());
    }

    pub fn f64x2_convert_low_i32x4_u(&mut self) {
        let v1 = self.pop_v128().as_u32x4();
        let mut v = f64x2::splat(0.0);

        for i in 0..v.len() {
            v[i] = v1[i] as f64;
        }

        self.push_v128(v.v128());
    }
}
