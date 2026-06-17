mod duplicates;
mod naming;
mod registry;
use crate::{Finding, Source};

pub use registry::rules;

pub fn analyze(source: &Source, output: &mut Vec<Finding>) {
    naming::analyze(source, output);
}

pub fn analyze_project(sources: &[Source], output: &mut Vec<Finding>) {
    duplicates::analyze(sources, output);
}
