use std::collections::{HashMap, HashSet};

use crate::errors::MirLowerError;
use crate::local_declarations::LocalDeclarations;
use crate::lower;
use crate::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

pub(crate) struct ExpressionLowerer<'a> {
    pub(crate) locals: &'a mut LocalDeclarations,
    pub(crate) variable_mappings: &'a HashMap<String, LocalIdentifier>,
    pub(crate) global_mappings: &'a HashMap<String, Type>,
    pub(crate) source_functions: &'a HashSet<String>,
    pub(crate) statements_buffer: &'a mut Vec<MirStatement>,
}
