use std::collections::{HashMap, HashSet};

use crate::errors::MirLowerError;
use crate::local_declarations::LocalDeclarations;
use crate::lower;
use crate::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;

mod places;
mod values;

pub(crate) struct ExpressionLowerer<'a> {
    pub(crate) locals: &'a mut LocalDeclarations,
    pub(crate) variable_mappings: &'a HashMap<String, LocalIdentifier>,
    pub(crate) global_mappings: &'a HashMap<String, Type>,
    pub(crate) source_functions: &'a HashSet<String>,
    pub(crate) statements_buffer: &'a mut Vec<MirStatement>,
}

impl ExpressionLowerer<'_> {
    fn allocate_temporary(&mut self, value_type: Type) -> LocalIdentifier {
        self.locals.allocate(None, value_type)
    }
}

impl ExpressionLowerer<'_> {
    fn emit_statement(&mut self, statement: MirStatement) {
        self.statements_buffer.push(statement);
    }
}

impl ExpressionLowerer<'_> {
    fn emit_temporary(&mut self, value_type: Type, value: Rvalue) -> Operand {
        let local = self.allocate_temporary(value_type);
        self.emit_statement(MirStatement::Assign(Place::Local(local), value));

        Operand::Move(Box::new(Place::Local(local)))
    }
}
