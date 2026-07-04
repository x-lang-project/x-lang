
pub multi pure fn min(left: i8, right: i8) -> i8 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: i8, right: i8) -> i8 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: i8, lower: i8, upper: i8) -> i8 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn signum(value: i8) -> i8 {
    if value < 0 as i8 { return -1 as i8; }
    if value > 0 as i8 { return 1 as i8; }
    return 0 as i8;
}

pub multi pure fn min(left: u8, right: u8) -> u8 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: u8, right: u8) -> u8 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: u8, lower: u8, upper: u8) -> u8 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn is_power_of_two(value: u8) -> bool {
    if value == 0 as u8 { return false; }
    return (value & (value - 1 as u8)) == 0 as u8;
}

pub multi pure fn gcd(left: u8, right: u8) -> u8 {
    let mut dividend: u8 = left;
    let mut divisor: u8 = right;
    while divisor != 0 as u8 {
        let remainder: u8 = dividend % divisor;
        dividend = divisor;
        divisor = remainder;
    }
    return dividend;
}

pub multi pure fn min(left: i16, right: i16) -> i16 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: i16, right: i16) -> i16 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: i16, lower: i16, upper: i16) -> i16 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn signum(value: i16) -> i16 {
    if value < 0 as i16 { return -1 as i16; }
    if value > 0 as i16 { return 1 as i16; }
    return 0 as i16;
}

pub multi pure fn min(left: u16, right: u16) -> u16 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: u16, right: u16) -> u16 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: u16, lower: u16, upper: u16) -> u16 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn is_power_of_two(value: u16) -> bool {
    if value == 0 as u16 { return false; }
    return (value & (value - 1 as u16)) == 0 as u16;
}

pub multi pure fn gcd(left: u16, right: u16) -> u16 {
    let mut dividend: u16 = left;
    let mut divisor: u16 = right;
    while divisor != 0 as u16 {
        let remainder: u16 = dividend % divisor;
        dividend = divisor;
        divisor = remainder;
    }
    return dividend;
}

pub multi pure fn min(left: i32, right: i32) -> i32 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: i32, right: i32) -> i32 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: i32, lower: i32, upper: i32) -> i32 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn signum(value: i32) -> i32 {
    if value < 0 as i32 { return -1 as i32; }
    if value > 0 as i32 { return 1 as i32; }
    return 0 as i32;
}

pub multi pure fn min(left: u32, right: u32) -> u32 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: u32, right: u32) -> u32 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: u32, lower: u32, upper: u32) -> u32 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn is_power_of_two(value: u32) -> bool {
    if value == 0 as u32 { return false; }
    return (value & (value - 1 as u32)) == 0 as u32;
}

pub multi pure fn gcd(left: u32, right: u32) -> u32 {
    let mut dividend: u32 = left;
    let mut divisor: u32 = right;
    while divisor != 0 as u32 {
        let remainder: u32 = dividend % divisor;
        dividend = divisor;
        divisor = remainder;
    }
    return dividend;
}

pub multi pure fn min(left: i64, right: i64) -> i64 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: i64, right: i64) -> i64 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: i64, lower: i64, upper: i64) -> i64 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn signum(value: i64) -> i64 {
    if value < 0 as i64 { return -1 as i64; }
    if value > 0 as i64 { return 1 as i64; }
    return 0 as i64;
}

pub multi pure fn min(left: u64, right: u64) -> u64 {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: u64, right: u64) -> u64 {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: u64, lower: u64, upper: u64) -> u64 {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn is_power_of_two(value: u64) -> bool {
    if value == 0 as u64 { return false; }
    return (value & (value - 1 as u64)) == 0 as u64;
}

pub multi pure fn gcd(left: u64, right: u64) -> u64 {
    let mut dividend: u64 = left;
    let mut divisor: u64 = right;
    while divisor != 0 as u64 {
        let remainder: u64 = dividend % divisor;
        dividend = divisor;
        divisor = remainder;
    }
    return dividend;
}

pub multi pure fn min(left: isize, right: isize) -> isize {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: isize, right: isize) -> isize {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: isize, lower: isize, upper: isize) -> isize {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn signum(value: isize) -> isize {
    if value < 0 as isize { return -1 as isize; }
    if value > 0 as isize { return 1 as isize; }
    return 0 as isize;
}

pub multi pure fn min(left: usize, right: usize) -> usize {
    if left < right { return left; }
    return right;
}

pub multi pure fn max(left: usize, right: usize) -> usize {
    if left > right { return left; }
    return right;
}

pub multi fn clamp(value: usize, lower: usize, upper: usize) -> usize {
    if lower > upper { intrinsic::trap(); }
    if value < lower { return lower; }
    if value > upper { return upper; }
    return value;
}

pub multi pure fn is_power_of_two(value: usize) -> bool {
    if value == 0 as usize { return false; }
    return (value & (value - 1 as usize)) == 0 as usize;
}

pub multi pure fn gcd(left: usize, right: usize) -> usize {
    let mut dividend: usize = left;
    let mut divisor: usize = right;
    while divisor != 0 as usize {
        let remainder: usize = dividend % divisor;
        dividend = divisor;
        divisor = remainder;
    }
    return dividend;
}

pub multi pure fn overflowing_add(left: i32, right: i32) -> (i32, bool) {
    return intrinsic::add_with_overflow_i32(left, right);
}

pub multi pure fn overflowing_sub(left: i32, right: i32) -> (i32, bool) {
    return intrinsic::sub_with_overflow_i32(left, right);
}

pub multi pure fn overflowing_mul(left: i32, right: i32) -> (i32, bool) {
    return intrinsic::mul_with_overflow_i32(left, right);
}

pub multi pure fn overflowing_add(left: usize, right: usize) -> (usize, bool) {
    return intrinsic::add_with_overflow_usize(left, right);
}

pub multi pure fn overflowing_sub(left: usize, right: usize) -> (usize, bool) {
    return intrinsic::sub_with_overflow_usize(left, right);
}

pub multi pure fn overflowing_mul(left: usize, right: usize) -> (usize, bool) {
    return intrinsic::mul_with_overflow_usize(left, right);
}

pub multi pure fn unsigned_abs(value: i8) -> u8 {
    if value < 0 as i8 {
        return (-(value + 1 as i8)) as u8 + 1 as u8;
    }
    return value as u8;
}

pub multi pure fn unsigned_abs(value: i16) -> u16 {
    if value < 0 as i16 {
        return (-(value + 1 as i16)) as u16 + 1 as u16;
    }
    return value as u16;
}

pub multi pure fn unsigned_abs(value: i32) -> u32 {
    if value < 0 as i32 {
        return (-(value + 1 as i32)) as u32 + 1 as u32;
    }
    return value as u32;
}

pub multi pure fn unsigned_abs(value: i64) -> u64 {
    if value < 0 as i64 {
        return (-(value + 1 as i64)) as u64 + 1 as u64;
    }
    return value as u64;
}

pub multi pure fn unsigned_abs(value: isize) -> usize {
    if value < 0 as isize {
        return (-(value + 1 as isize)) as usize + 1 as usize;
    }
    return value as usize;
}

pub multi pure fn saturating_add(left: i32, right: i32) -> i32 {
    let (value, overflow): (i32, bool) = intrinsic::add_with_overflow_i32(left, right);
    if !overflow { return value; }
    if left < 0 { return -2147483647 - 1; }
    return 2147483647;
}

pub multi pure fn saturating_sub(left: i32, right: i32) -> i32 {
    let (value, overflow): (i32, bool) = intrinsic::sub_with_overflow_i32(left, right);
    if !overflow { return value; }
    if left < 0 { return -2147483647 - 1; }
    return 2147483647;
}

pub multi pure fn saturating_mul(left: i32, right: i32) -> i32 {
    let (value, overflow): (i32, bool) = intrinsic::mul_with_overflow_i32(left, right);
    if !overflow { return value; }
    if (left < 0) != (right < 0) { return -2147483647 - 1; }
    return 2147483647;
}

pub multi pure fn saturating_add(left: usize, right: usize) -> usize {
    let (value, overflow): (usize, bool) = intrinsic::add_with_overflow_usize(left, right);
    if !overflow { return value; }
    return ~(0 as usize);
}

pub multi pure fn saturating_sub(left: usize, right: usize) -> usize {
    let (value, overflow): (usize, bool) = intrinsic::sub_with_overflow_usize(left, right);
    if !overflow { return value; }
    return 0 as usize;
}

pub multi pure fn saturating_mul(left: usize, right: usize) -> usize {
    let (value, overflow): (usize, bool) = intrinsic::mul_with_overflow_usize(left, right);
    if !overflow { return value; }
    return ~(0 as usize);
}

