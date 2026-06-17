mod part0;
mod part1;
mod part2;
mod part3;
mod part4;
use crate::Rule;

pub(super) fn rules() -> Vec<Rule> {
    let mut rules = Vec::new();
    rules.extend(part0::rules());
    rules.extend(part1::rules());
    rules.extend(part2::rules());
    rules.extend(part3::rules());
    rules.extend(part4::rules());

    rules
}
