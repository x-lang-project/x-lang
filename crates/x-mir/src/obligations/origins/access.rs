use std::collections::BTreeSet;

use super::Analysis;
use super::Records;
use super::model::Origin;
use super::model::State;
use super::model::Step;
use super::model::Value;
use crate::{LocalIdentifier, MirBody, Operand, Place};

fn index(value: &Operand) -> Step {
    Step::Index(if let Operand::ConstantInteger(index) = value {
        Some(*index)
    } else {
        None
    })
}
