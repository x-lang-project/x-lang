pub multi pure fn count_ones(value: u8) -> u32 { return intrinsic::ctpop_u32(value as u32); }
pub multi pure fn count_ones(value: u16) -> u32 { return intrinsic::ctpop_u32(value as u32); }
pub multi pure fn count_ones(value: u32) -> u32 { return intrinsic::ctpop_u32(value); }
pub multi pure fn count_ones(value: u64) -> u32 {
    return intrinsic::ctpop_u32(value as u32) + intrinsic::ctpop_u32((value >> 32) as u32);
}
pub multi pure fn leading_zeros(value: u8) -> u32 { return intrinsic::ctlz_u32(value as u32) - 24 as u32; }
pub multi pure fn leading_zeros(value: u16) -> u32 { return intrinsic::ctlz_u32(value as u32) - 16 as u32; }
pub multi pure fn leading_zeros(value: u32) -> u32 { return intrinsic::ctlz_u32(value); }
pub multi pure fn leading_zeros(value: u64) -> u32 {
    let high: u32 = (value >> 32) as u32;
    if high != 0 as u32 { return intrinsic::ctlz_u32(high); }
    return 32 as u32 + intrinsic::ctlz_u32(value as u32);
}
pub multi pure fn trailing_zeros(value: u8) -> u32 {
    if value == 0 as u8 { return 8 as u32; }
    return intrinsic::cttz_u32(value as u32);
}
pub multi pure fn trailing_zeros(value: u16) -> u32 {
    if value == 0 as u16 { return 16 as u32; }
    return intrinsic::cttz_u32(value as u32);
}
pub multi pure fn trailing_zeros(value: u32) -> u32 { return intrinsic::cttz_u32(value); }
pub multi pure fn trailing_zeros(value: u64) -> u32 {
    let low: u32 = value as u32;
    if low != 0 as u32 { return intrinsic::cttz_u32(low); }
    return 32 as u32 + intrinsic::cttz_u32((value >> 32) as u32);
}
pub multi pure fn swap_bytes(value: u8) -> u8 { return value; }
pub multi pure fn swap_bytes(value: u16) -> u16 { return intrinsic::bswap_u16(value); }
pub multi pure fn swap_bytes(value: u32) -> u32 { return intrinsic::bswap_u32(value); }
pub multi pure fn swap_bytes(value: u64) -> u64 { return intrinsic::bswap_u64(value); }
pub multi pure fn rotate_left(value: u8, count: u32) -> u8 {
    let shift: u32 = count % 8 as u32;
    return ((value as u32 << shift) | (value as u32 >> ((8 as u32 - shift) % 8 as u32))) as u8;
}
pub multi pure fn rotate_left(value: u16, count: u32) -> u16 {
    let shift: u32 = count % 16 as u32;
    return ((value as u32 << shift) | (value as u32 >> ((16 as u32 - shift) % 16 as u32))) as u16;
}
pub multi pure fn rotate_left(value: u32, count: u32) -> u32 { return intrinsic::rotl_u32(value, count); }
pub multi pure fn rotate_left(value: u64, count: u32) -> u64 {
    let shift: u64 = (count % 64 as u32) as u64;
    return (value << shift) | (value >> ((64 as u64 - shift) % 64 as u64));
}
pub multi pure fn rotate_right(value: u8, count: u32) -> u8 {
    let shift: u32 = count % 8 as u32;
    return ((value as u32 >> shift) | (value as u32 << ((8 as u32 - shift) % 8 as u32))) as u8;
}
pub multi pure fn rotate_right(value: u16, count: u32) -> u16 {
    let shift: u32 = count % 16 as u32;
    return ((value as u32 >> shift) | (value as u32 << ((16 as u32 - shift) % 16 as u32))) as u16;
}
pub multi pure fn rotate_right(value: u32, count: u32) -> u32 { return intrinsic::rotr_u32(value, count); }
pub multi pure fn rotate_right(value: u64, count: u32) -> u64 {
    let shift: u64 = (count % 64 as u32) as u64;
    return (value >> shift) | (value << ((64 as u64 - shift) % 64 as u64));
}
