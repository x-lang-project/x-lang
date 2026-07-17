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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoiseCallContract {
    pub capacity: u32,
    pub key_encoding: MemoiseKeyEncoding,
    pub indexing: MemoiseIndexing,
    pub synchronization: MemoiseSynchronization,
    pub unlock_before_proceed: bool,
    pub per_function_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallContract {
    Memoise(MemoiseCallContract),
}
