/// Visited absolute values, totals and the encoded result must fit i32.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatefulCase {
    pub values: [i32; 4],
    /// One-based iteration to skip. Values outside `1..=4` never skip.
    pub continue_after: usize,
    /// One-based iteration to stop after accumulation; a skipped iteration does not stop.
    pub break_after: usize,
}
