/// Visited absolute values, totals and the encoded result must fit i32.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatefulCase {
    pub values: [i32; 4],
    /// One-based iteration to skip. Values outside `1..=4` never skip.
    pub continue_after: usize,
    /// One-based iteration to stop after accumulation; a skipped iteration does not stop.
    pub break_after: usize,
}

impl StatefulCase {
    #[must_use]
    pub fn evaluate(&self) -> i32 {
        let values = self.values;
        let mut index = 0;
        let mut total = 0;

        while index < values.len() {
            let current = values[index];
            index += 1;
            if index == self.continue_after {
                continue;
            }

            total += current.abs();

            if index == self.break_after {
                break;
            }
        }

        total * 100 + values[0] * 10 + index as i32
    }
}
