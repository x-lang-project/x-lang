use std::collections::HashMap;
use std::sync::Arc;

use crate::return_summaries::ReturnSummary;
use x_ast::{StructDefinition, Type};

#[derive(Clone, Debug)]
pub(crate) struct FunctionLifetimeContext {
    pub(crate) parameter_references: HashMap<String, Type>,
    pub(crate) return_type: Type,
}
