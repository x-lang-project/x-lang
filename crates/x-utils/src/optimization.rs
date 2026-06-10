#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeOptimization(u8);

impl NativeOptimization {
    pub const MAX: u8 = 3;

    pub fn argument(self) -> String {
        format!("-O{}", self.0)
    }
}
