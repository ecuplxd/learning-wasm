use crate::execution::stack::operand::Operand;
use crate::execution::vm::VM;

pub enum TruncSize {
    N32,
    N64,
}

impl TruncSize {
    fn get_u_max(&self) -> u64 {
        match self {
            Self::N32 => u32::MAX as u64,
            Self::N64 => u64::MAX,
        }
    }

    fn get_i_min(&self) -> i64 {
        match self {
            Self::N32 => i32::MIN as i64,
            Self::N64 => i64::MIN,
        }
    }

    fn get_i_max(&self) -> i64 {
        match self {
            Self::N32 => i32::MAX as i64,
            Self::N64 => i64::MAX,
        }
    }
}

/// https://webassembly.github.io/spec/core/exec/instructions.html#exec-cvtop
impl VM {
    pub fn i32_trunc_sat_f32_u(&mut self) {
        let v1 = self.pop_f32();
        let v1 = trunc_sat_u(v1 as f64, TruncSize::N32);

        self.push_u32(v1 as u32);
    }

    pub fn i32_trunc_sat_f32_s(&mut self) {
        let v1 = self.pop_f32();
        let v1 = trunc_sat_s(v1 as f64, TruncSize::N32);

        self.push_i32(v1 as i32);
    }

    pub fn i32_trunc_sat_f64_u(&mut self) {
        let v1 = self.pop_f64();
        let v1 = trunc_sat_u(v1, TruncSize::N32);

        self.push_u32(v1 as u32);
    }

    pub fn i32_trunc_sat_f64_s(&mut self) {
        let v1 = self.pop_f64();
        let v1 = trunc_sat_s(v1, TruncSize::N32);

        self.push_i32(v1 as i32);
    }

    pub fn i64_trunc_sat_f32_u(&mut self) {
        let v1 = self.pop_f32();
        let v1 = trunc_sat_u(v1 as f64, TruncSize::N64);

        self.push_u64(v1);
    }

    pub fn i64_trunc_sat_f32_s(&mut self) {
        let v1 = self.pop_f32();
        let v1 = trunc_sat_s(v1 as f64, TruncSize::N64);

        self.push_i64(v1);
    }

    pub fn i64_trunc_sat_f64_u(&mut self) {
        let v1 = self.pop_f64();
        let v1 = trunc_sat_u(v1, TruncSize::N64);

        self.push_u64(v1);
    }

    pub fn i64_trunc_sat_f64_s(&mut self) {
        let v1 = self.pop_f64();
        let v1 = trunc_sat_s(v1, TruncSize::N64);

        self.push_i64(v1);
    }
}

pub fn trunc_sat_u(num: f64, n: TruncSize) -> u64 {
    if num.is_nan() || num == f64::NEG_INFINITY {
        return 0;
    }

    let max = n.get_u_max();

    if num == f64::INFINITY {
        return max;
    }

    let num = num.trunc();

    match num < 0.0 {
        true => 0,
        false if num >= (max as f64) => max,
        _ => num as u64,
    }
}

pub fn trunc_sat_s(num: f64, n: TruncSize) -> i64 {
    if num.is_nan() {
        return 0;
    }

    let min = n.get_i_min();
    let max = n.get_i_max();

    if num == f64::NEG_INFINITY {
        return min;
    }

    if num == f64::INFINITY {
        return max;
    }

    let num = num.trunc();

    match num < (min as f64) {
        true => min,
        false if num >= (max as f64) => max,
        _ => num as i64,
    }
}
