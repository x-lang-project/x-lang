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

    /// Control fields must fit X i32 literals.
    #[must_use]
    pub fn render_assertion(&self, case: usize, failure_code: usize) -> String {
        let [first, second, third, fourth] = self.values;
        let expected = self.evaluate();
        let continue_after = self.continue_after;
        let break_after = self.break_after;

        format!(
            r#"    {{
        let mut values_{case}: [i32; 4] = [{first}, {second}, {third}, {fourth}];
        let mut index_{case}: i32 = 0;
        let mut total_{case}: i32 = 0;
        while index_{case} < 4 {{
            let current_{case}: i32 = values_{case}[index_{case}];
            index_{case} = index_{case} + 1;
            if index_{case} == {continue_after} {{ continue; }}
            if current_{case} < 0 {{
                total_{case} = total_{case} + -current_{case};
            }} else {{
                total_{case} = total_{case} + current_{case};
            }}
            if index_{case} == {break_after} {{ break; }}
        }}
        values_{case}[2] = total_{case};
        let pair_{case}: GeneratedPair = GeneratedPair {{ left: total_{case}, right: values_{case}[0] }};
        let tuple_{case}: (i32, i32, i32) = (pair_{case}.left, pair_{case}.right, index_{case});
        let (sum_{case}, first_{case}, iterations_{case}) = tuple_{case};
        let observed_{case}: i32 = sum_{case} * 100 + first_{case} * 10 + iterations_{case};
        if observed_{case} != {expected} {{ return {failure_code}; }}
    }}
"#,
        )
    }
}
