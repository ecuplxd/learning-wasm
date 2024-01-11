use std::ops::{Shl, Shr};

use crate::execution::errors::{Trap, VMState};
use crate::execution::stack::operand::Operand;
use crate::execution::vm::VM;

impl VM {
    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-const
    /// 常量指令：立即数压栈
    pub fn i32_const(&mut self, v: i32) {
        self.push_i32(v);
    }

    pub fn i64_const(&mut self, v: i64) {
        self.push_i64(v);
    }

    pub fn f32_const(&mut self, v: f32) {
        self.push_f32(v);
    }

    pub fn f64_const(&mut self, v: f64) {
        self.push_f64(v);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-testop
    /// 测试指令：测试是否为 0，测试结果压栈
    pub fn i32_eqz(&mut self) {
        let v1 = self.pop_i32();

        self.push_bool(v1 == 0);
    }

    pub fn i64_eqz(&mut self) {
        let v1 = self.pop_i64();

        self.push_bool(v1 == 0);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-relop
    /// 比较指令：比较结果压栈
    pub fn i32_eq(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_bool(v1 == v2);
    }

    pub fn i64_eq(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_bool(v1 == v2);
    }

    pub fn f32_eq(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_bool(v1 == v2);
    }

    pub fn f64_eq(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_bool(v1 == v2);
    }

    pub fn i32_ne(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_bool(v1 != v2);
    }

    pub fn i64_ne(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_bool(v1 != v2);
    }

    pub fn f32_ne(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_bool(v1 != v2);
    }

    pub fn f64_ne(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_bool(v1 != v2);
    }

    pub fn i32_lt_u(&mut self) {
        let v2 = self.pop_u32();
        let v1 = self.pop_u32();

        self.push_bool(v1 < v2);
    }

    pub fn i32_lt_s(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_bool(v1 < v2);
    }

    pub fn i64_lt_u(&mut self) {
        let v2 = self.pop_u64();
        let v1 = self.pop_u64();

        self.push_bool(v1 < v2);
    }

    pub fn i64_lt_s(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_bool(v1 < v2);
    }

    pub fn f32_lt(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_bool(v1 < v2);
    }

    pub fn f64_lt(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_bool(v1 < v2);
    }

    pub fn i32_gt_u(&mut self) {
        let v2 = self.pop_u32();
        let v1 = self.pop_u32();

        self.push_bool(v1 > v2);
    }

    pub fn i32_gt_s(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_bool(v1 > v2);
    }

    pub fn i64_gt_u(&mut self) {
        let v2 = self.pop_u64();
        let v1 = self.pop_u64();

        self.push_bool(v1 > v2);
    }

    pub fn i64_gt_s(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_bool(v1 > v2);
    }

    pub fn f32_gt(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_bool(v1 > v2);
    }

    pub fn f64_gt(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_bool(v1 > v2);
    }

    pub fn i32_le_u(&mut self) {
        let v2 = self.pop_u32();
        let v1 = self.pop_u32();

        self.push_bool(v1 <= v2);
    }

    pub fn i32_le_s(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_bool(v1 <= v2);
    }

    pub fn i64_le_u(&mut self) {
        let v2 = self.pop_u64();
        let v1 = self.pop_u64();

        self.push_bool(v1 <= v2);
    }

    pub fn i64_le_s(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_bool(v1 <= v2);
    }

    pub fn f32_le(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_bool(v1 <= v2);
    }

    pub fn f64_le(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_bool(v1 <= v2);
    }

    pub fn i32_ge_u(&mut self) {
        let v2 = self.pop_u32();
        let v1 = self.pop_u32();

        self.push_bool(v1 >= v2);
    }

    pub fn i32_ge_s(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_bool(v1 >= v2);
    }

    pub fn i64_ge_u(&mut self) {
        let v2 = self.pop_u64();
        let v1 = self.pop_u64();

        self.push_bool(v1 >= v2);
    }

    pub fn i64_ge_s(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_bool(v1 >= v2);
    }

    pub fn f32_ge(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_bool(v1 >= v2);
    }

    pub fn f64_ge(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_bool(v1 >= v2);
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-unop
    /// 一元算术指令
    /// 前导 0 比特数
    pub fn i32_clz(&mut self) {
        let v1 = self.pop_i32();

        self.push_i32(v1.leading_zeros() as i32);
    }

    pub fn i64_clz(&mut self) {
        let v1 = self.pop_i64();

        self.push_i64(v1.leading_zeros() as i64);
    }

    /// 后置 0 比特数
    pub fn i32_ctz(&mut self) {
        let v1 = self.pop_i32();

        self.push_i32(v1.trailing_zeros() as i32);
    }

    pub fn i64_ctz(&mut self) {
        let v1 = self.pop_i64();

        self.push_i64(v1.trailing_zeros() as i64);
    }

    /// 1 比特数
    pub fn i32_popcnt(&mut self) {
        let v1 = self.pop_i32();

        self.push_i32(v1.count_ones() as i32);
    }

    pub fn i64_popcnt(&mut self) {
        let v1 = self.pop_i64();

        self.push_i64(v1.count_ones() as i64);
    }

    pub fn f32_abs(&mut self) {
        let v1 = self.pop_f32();

        self.push_f32(v1.abs());
    }

    pub fn f64_abs(&mut self) {
        let v1 = self.pop_f64();

        self.push_f64(v1.abs());
    }

    pub fn f32_neg(&mut self) {
        let v1 = self.pop_f32();

        self.push_f32(-v1);
    }

    pub fn f64_neg(&mut self) {
        let v1 = self.pop_f64();

        self.push_f64(-v1);
    }

    pub fn f32_ceil(&mut self) {
        let v1 = self.pop_f32();

        self.push_f32(v1.ceil());
    }

    pub fn f64_ceil(&mut self) {
        let v1 = self.pop_f64();

        self.push_f64(v1.ceil());
    }

    pub fn f32_floor(&mut self) {
        let v1 = self.pop_f32();

        self.push_f32(v1.floor());
    }

    pub fn f64_floor(&mut self) {
        let v1 = self.pop_f64();

        self.push_f64(v1.floor());
    }

    pub fn f32_trunc(&mut self) {
        let v1 = self.pop_f32();

        self.push_f32(v1.trunc());
    }

    pub fn f64_trunc(&mut self) {
        let v1 = self.pop_f64();

        self.push_f64(v1.trunc());
    }

    pub fn f32_nearest(&mut self) {
        let v1 = self.pop_f32();

        self.push_f32(v1.round_ties_even());
    }

    pub fn f64_nearest(&mut self) {
        let v1 = self.pop_f64();

        self.push_f64(v1.round_ties_even());
    }

    pub fn f32_sqrt(&mut self) {
        let v1 = self.pop_f32();

        self.push_f32(v1.sqrt());
    }

    pub fn f64_sqrt(&mut self) {
        let v1 = self.pop_f64();

        self.push_f64(v1.sqrt());
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-binop
    pub fn i32_add(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1 + v2);
    }

    pub fn i64_add(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1 + v2);
    }

    pub fn f32_add(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_f32(v1 + v2);
    }

    pub fn f64_add(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_f64(v1 + v2);
    }

    pub fn i32_sub(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1 - v2);
    }

    pub fn i64_sub(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1 - v2);
    }

    pub fn f32_sub(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_f32(v1 - v2);
    }

    pub fn f64_sub(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_f64(v1 - v2);
    }

    pub fn i32_mul(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1 * v2);
    }

    pub fn i64_mul(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1 * v2);
    }

    pub fn f32_mul(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_f32(v1 * v2);
    }

    pub fn f64_mul(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_f64(v1 * v2);
    }

    pub fn i32_div_u(&mut self) -> VMState {
        let v2 = self.pop_u32();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_u32();

        self.push_u32(v1 / v2);

        Ok(())
    }

    pub fn i32_div_s(&mut self) -> VMState {
        let v2 = self.pop_i32();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_i32();
        let (v, o) = v1.overflowing_div(v2);

        if o {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_i32(v);

        Ok(())
    }

    pub fn i64_div_u(&mut self) -> VMState {
        let v2 = self.pop_u64();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_u64();

        self.push_u64(v1 / v2);

        Ok(())
    }

    pub fn i64_div_s(&mut self) -> VMState {
        let v2 = self.pop_i64();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_i64();
        let (v, o) = v1.overflowing_div(v2);

        if o {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_i64(v);

        Ok(())
    }

    pub fn f32_div(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_f32(v1 / v2);
    }

    pub fn f64_div(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_f64(v1 / v2);
    }

    pub fn i32_rem_u(&mut self) -> VMState {
        let v2 = self.pop_u32();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_u32();
        let (v, _) = v1.overflowing_rem(v2);

        self.push_u32(v);

        Ok(())
    }

    pub fn i32_rem_s(&mut self) -> VMState {
        let v2 = self.pop_i32();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_i32();
        let (v, _) = v1.overflowing_rem(v2);

        self.push_i32(v);

        Ok(())
    }

    pub fn i64_rem_u(&mut self) -> VMState {
        let v2 = self.pop_u64();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_u64();
        let (v, _) = v1.overflowing_rem(v2);

        self.push_u64(v);

        Ok(())
    }

    pub fn i64_rem_s(&mut self) -> VMState {
        let v2 = self.pop_i64();

        if v2 == 0 {
            Err(Trap::DivZero)?;
        }

        let v1 = self.pop_i64();
        let (v, _) = v1.overflowing_rem(v2);

        self.push_i64(v);

        Ok(())
    }

    pub fn i32_and(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1 & v2);
    }

    pub fn i64_and(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1 & v2);
    }

    pub fn i32_or(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1 | v2);
    }

    pub fn i64_or(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1 | v2);
    }

    pub fn i32_xor(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1 ^ v2);
    }

    pub fn i64_xor(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1 ^ v2);
    }

    pub fn i32_shl(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1.shl(v2));
    }

    pub fn i64_shl(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1.shl(v2));
    }

    pub fn i32_shr_u(&mut self) {
        let v2 = self.pop_u32();
        let v1 = self.pop_u32();

        self.push_u32(v1.shr(v2));
    }

    pub fn i32_shr_s(&mut self) {
        let v2 = self.pop_i32();
        let v1 = self.pop_i32();

        self.push_i32(v1.shr(v2));
    }

    pub fn i64_shr_u(&mut self) {
        let v2 = self.pop_u64();
        let v1 = self.pop_u64();

        self.push_u64(v1.shr(v2));
    }

    pub fn i64_shr_s(&mut self) {
        let v2 = self.pop_i64();
        let v1 = self.pop_i64();

        self.push_i64(v1.shr(v2));
    }

    pub fn i32_rotl(&mut self) {
        let v2 = self.pop_u32();
        let v1 = self.pop_u32();

        self.push_u32(v1.rotate_left(v2));
    }

    pub fn i64_rotl(&mut self) {
        let v2 = self.pop_u64();
        let v1 = self.pop_u64();

        self.push_u64(v1.rotate_left(v2 as u32));
    }

    pub fn i32_rotr(&mut self) {
        let v2 = self.pop_u32();
        let v1 = self.pop_u32();

        self.push_u32(v1.rotate_right(v2));
    }

    pub fn i64_rotr(&mut self) {
        let v2 = self.pop_u64();
        let v1 = self.pop_i64();

        self.push_i64(v1.rotate_right(v2 as u32));
    }

    pub fn f32_min(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        match v1.is_nan() {
            true => self.push_f32(v1),
            false if v2.is_nan() => self.push_f32(v2),
            _ => self.push_f32(v1.min(v2)),
        }
    }

    pub fn f64_min(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        match v1.is_nan() {
            true => self.push_f64(v1),
            false if v2.is_nan() => self.push_f64(v2),
            _ => self.push_f64(v1.min(v2)),
        }
    }

    pub fn f32_max(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        match v1.is_nan() {
            true => self.push_f32(v1),
            false if v2.is_nan() => self.push_f32(v2),
            _ => self.push_f32(v1.max(v2)),
        }
    }

    pub fn f64_max(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        match v1.is_nan() {
            true => self.push_f64(v1),
            false if v2.is_nan() => self.push_f64(v2),
            _ => self.push_f64(v1.max(v2)),
        }
    }

    pub fn f32_copysign(&mut self) {
        let v2 = self.pop_f32();
        let v1 = self.pop_f32();

        self.push_f32(v1.copysign(v2));
    }

    pub fn f64_copysign(&mut self) {
        let v2 = self.pop_f64();
        let v1 = self.pop_f64();

        self.push_f64(v1.copysign(v2));
    }

    /// https://webassembly.github.io/spec/core/exec/instructions.html#exec-cvtop
    /// 类型转换：整数拉升
    pub fn i32_extend8_s(&mut self) {
        let v1 = self.pop_i32();

        self.push_i32(v1 << 24 >> 24);
    }

    pub fn i32_extend16_s(&mut self) {
        let v1 = self.pop_i32();

        self.push_i32(v1 << 16 >> 16);
    }

    pub fn i64_extend8_s(&mut self) {
        let v1 = self.pop_i64();

        self.push_i64(v1 << 56 >> 56);
    }

    pub fn i64_extend16_s(&mut self) {
        let v1 = self.pop_i64();

        self.push_i64(v1 << 48 >> 48);
    }

    pub fn i64_extend32_s(&mut self) {
        let v1 = self.pop_i64();

        self.push_i64(v1 << 32 >> 32);
    }

    pub fn i64_extend_i32_u(&mut self) {
        let v1 = self.pop_u32();

        self.push_u64(v1 as u64);
    }

    pub fn i64_extend_i32_s(&mut self) {
        let v1 = self.pop_i32();

        self.push_i64(v1 as i64);
    }

    /// 类型转换：整数截断 64 -> 32
    pub fn i32_wrap_i64(&mut self) {
        let v1 = self.pop_i64();

        self.push_i32(v1 as i32);
    }

    /// 类型转换：浮点数截断 f32 -> i32
    pub fn i32_trunc_f32_u(&mut self) -> VMState {
        let v1 = self.pop_f32();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 <= -1.0f32 || v1 >= 4294967296.0f32 {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_u32(v1.trunc() as u32);

        Ok(())
    }

    pub fn i32_trunc_f32_s(&mut self) -> VMState {
        let v1 = self.pop_f32();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 < -2147483648.0f32 || v1 >= 2147483648.0f32 {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_i32(v1.trunc() as i32);

        Ok(())
    }

    pub fn i32_trunc_f64_u(&mut self) -> VMState {
        let v1 = self.pop_f64();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 <= -1. || v1 >= 4294967296. {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_u32(v1.trunc() as u32);

        Ok(())
    }

    pub fn i32_trunc_f64_s(&mut self) -> VMState {
        let v1 = self.pop_f64();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 <= -2147483649. || v1 >= 2147483648. {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_i32(v1.trunc() as i32);

        Ok(())
    }

    pub fn i64_trunc_f32_u(&mut self) -> VMState {
        let v1 = self.pop_f32();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 <= -1.0f32 || v1 >= 18446744073709551616.0f32 {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_u64(v1.trunc() as u64);

        Ok(())
    }

    pub fn i64_trunc_f32_s(&mut self) -> VMState {
        let v1 = self.pop_f32();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 < -9223372036854775808.0f32 || v1 >= 9223372036854775808.0f32 {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_i64(v1.trunc() as i64);

        Ok(())
    }

    pub fn i64_trunc_f64_u(&mut self) -> VMState {
        let v1 = self.pop_f64();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 <= -1. || v1 >= 18446744073709551616. {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_u64(v1.trunc() as u64);

        Ok(())
    }

    pub fn i64_trunc_f64_s(&mut self) -> VMState {
        let v1 = self.pop_f64();

        if v1.is_nan() {
            Err(Trap::InvalidConversionToInteger)?;
        }

        if v1 < -9223372036854775808. || v1 >= 9223372036854775808. {
            Err(Trap::IntegerOverflow)?;
        }

        self.push_i64(v1.trunc() as i64);

        Ok(())
    }

    /// 类型转换：浮点数精度调整
    pub fn f32_demote_f64(&mut self) {
        let v1 = self.pop_f64();

        self.push_f32(v1 as f32);
    }

    pub fn f64_promote_f32(&mut self) {
        let v1 = self.pop_f32();

        self.push_f64(v1 as f64);
    }

    /// 类型转换：整数转浮点数
    pub fn f32_convert_i32_u(&mut self) {
        let v1 = self.pop_u32();

        self.push_f32(v1 as f32);
    }

    pub fn f32_convert_i32_s(&mut self) {
        let v1 = self.pop_i32();

        self.push_f32(v1 as f32);
    }

    pub fn f32_convert_i64_u(&mut self) {
        let v1 = self.pop_u64();

        self.push_f32(v1 as f32);
    }

    pub fn f32_convert_i64_s(&mut self) {
        let v1 = self.pop_i64();

        self.push_f32(v1 as f32);
    }

    pub fn f64_convert_i32_u(&mut self) {
        let v1 = self.pop_u32();

        self.push_f64(v1 as f64);
    }

    pub fn f64_convert_i32_s(&mut self) {
        let v1 = self.pop_i32();

        self.push_f64(v1 as f64);
    }

    pub fn f64_convert_i64_u(&mut self) {
        let v1 = self.pop_u64();

        self.push_f64(v1 as f64);
    }

    pub fn f64_convert_i64_s(&mut self) {
        let v1 = self.pop_i64();

        self.push_f64(v1 as f64);
    }

    /// 类型转换：重解释 f32 -> i32
    pub fn i32_reinterpret_f32(&mut self) {
        let v1 = self.pop_f32();

        self.push_i32(v1.to_bits() as i32);
    }

    pub fn i64_reinterpret_f64(&mut self) {
        let v1 = self.pop_f64();

        self.push_i64(v1.to_bits() as i64);
    }

    pub fn f32_reinterpret_i32(&mut self) {
        let v1 = self.pop_i32() as u32;

        self.push_f32(f32::from_bits(v1));
    }

    pub fn f64_reinterpret_i64(&mut self) {
        let v1 = self.pop_i64() as u64;

        self.push_f64(f64::from_bits(v1));
    }
}
