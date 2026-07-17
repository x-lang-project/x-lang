use std::collections::BTreeSet;

use super::model::Step;
use crate::{LocalIdentifier, MirBody, Operand, Place};

fn index(value: &Operand) -> Step {
    Step::Index(if let Operand::ConstantInteger(index) = value {
        Some(*index)
    } else {
        None
    })
}
