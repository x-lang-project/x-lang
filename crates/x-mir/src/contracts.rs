#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoiseKeyEncoding {
    CompleteBitwiseU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoiseIndexing {
    UnsignedRemainder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoiseSynchronization {
    AcquireReleaseLock,
}

pub const DEFAULT_MEMOISE_CAPACITY: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoiseCallContract {
    pub capacity: u32,
    pub key_encoding: MemoiseKeyEncoding,
    pub indexing: MemoiseIndexing,
    pub synchronization: MemoiseSynchronization,
    pub unlock_before_proceed: bool,
    pub per_function_state: bool,
}

impl MemoiseCallContract {
    pub fn checked(capacity: u32) -> Self {
        Self {
            capacity,
            key_encoding: MemoiseKeyEncoding::CompleteBitwiseU64,
            indexing: MemoiseIndexing::UnsignedRemainder,
            synchronization: MemoiseSynchronization::AcquireReleaseLock,
            unlock_before_proceed: true,
            per_function_state: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallContract {
    Memoise(MemoiseCallContract),
}
