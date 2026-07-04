
pub multi pure fn sqrt(value: f32) -> f32 {
    return intrinsic::f32_sqrt(value);
}

pub multi pure fn floor(value: f32) -> f32 {
    return intrinsic::f32_floor(value);
}

pub multi pure fn ceil(value: f32) -> f32 {
    return intrinsic::f32_ceil(value);
}

pub multi pure fn round(value: f32) -> f32 {
    return intrinsic::f32_round(value);
}

pub multi pure fn is_nan(value: f32) -> bool {
    return intrinsic::f32_is_nan(value);
}

pub multi pure fn is_infinite(value: f32) -> bool {
    return intrinsic::f32_is_infinite(value);
}

pub multi pure fn abs(value: f32) -> f32 {
    if value == 0.0 as f32 { return 0.0 as f32; }
    if value < 0.0 as f32 { return -value; }
    return value;
}

pub multi pure fn is_finite(value: f32) -> bool {
    return !intrinsic::f32_is_nan(value) && !intrinsic::f32_is_infinite(value);
}

pub multi pure fn sqrt(value: f64) -> f64 {
    return intrinsic::f64_sqrt(value);
}

pub multi pure fn floor(value: f64) -> f64 {
    return intrinsic::f64_floor(value);
}

pub multi pure fn ceil(value: f64) -> f64 {
    return intrinsic::f64_ceil(value);
}

pub multi pure fn round(value: f64) -> f64 {
    return intrinsic::f64_round(value);
}

pub multi pure fn is_nan(value: f64) -> bool {
    return intrinsic::f64_is_nan(value);
}

pub multi pure fn is_infinite(value: f64) -> bool {
    return intrinsic::f64_is_infinite(value);
}

pub multi pure fn sin(value: f64) -> f64 {
    return intrinsic::f64_sin(value);
}

pub multi pure fn cos(value: f64) -> f64 {
    return intrinsic::f64_cos(value);
}

pub multi pure fn tan(value: f64) -> f64 {
    return intrinsic::f64_tan(value);
}

pub multi pure fn asin(value: f64) -> f64 {
    return intrinsic::f64_asin(value);
}

pub multi pure fn acos(value: f64) -> f64 {
    return intrinsic::f64_acos(value);
}

pub multi pure fn atan(value: f64) -> f64 {
    return intrinsic::f64_atan(value);
}

pub multi pure fn exp(value: f64) -> f64 {
    return intrinsic::f64_exp(value);
}

pub multi pure fn ln(value: f64) -> f64 {
    return intrinsic::f64_ln(value);
}

pub pure fn atan2(left: f64, right: f64) -> f64 {
    return intrinsic::f64_atan2(left, right);
}

pub pure fn powf(left: f64, right: f64) -> f64 {
    return intrinsic::f64_powf(left, right);
}

pub multi pure fn abs(value: f64) -> f64 {
    if value == 0.0 as f64 { return 0.0 as f64; }
    if value < 0.0 as f64 { return -value; }
    return value;
}

pub multi pure fn is_finite(value: f64) -> bool {
    return !intrinsic::f64_is_nan(value) && !intrinsic::f64_is_infinite(value);
}
