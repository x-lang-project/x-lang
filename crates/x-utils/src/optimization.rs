#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeOptimization(u8);

impl NativeOptimization {
    pub const MAX: u8 = 3;

    pub fn new(level: u8) -> Self {
        Self(level.min(Self::MAX))
    }

    pub fn level(self) -> u8 {
        self.0
    }

    pub fn argument(self) -> String {
        format!("-O{}", self.0)
    }
}
