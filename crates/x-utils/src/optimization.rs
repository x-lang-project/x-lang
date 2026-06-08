#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeOptimization(u8);

impl NativeOptimization {
    pub const MAX: u8 = 3;
}
