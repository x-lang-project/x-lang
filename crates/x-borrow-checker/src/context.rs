use std::collections::HashMap;
use std::sync::Arc;

use crate::return_summaries::ReturnSummary;
use x_ast::{StructDefinition, Type};

#[derive(Clone, Debug)]
pub(crate) struct FunctionLifetimeContext {
    pub(crate) parameter_references: HashMap<String, Type>,
    pub(crate) return_type: Type,
}

#[derive(Clone, Default)]
pub(crate) struct AnalysisContext {
    pub(crate) function_contexts: Vec<FunctionLifetimeContext>,
    pub(crate) struct_layouts: Arc<HashMap<String, StructDefinition>>,
    pub(crate) method_receivers: Arc<HashMap<(String, String), (String, Type)>>,
    pub(crate) return_summaries: Arc<HashMap<String, Arc<ReturnSummary>>>,
}
