mod completion;
mod dispatch;
mod literals;
mod references;

use std::collections::HashMap;

use crate::purity::ImpureOperation;
use crate::symbols::mangle_type_components;
use crate::{
    FunctionSignature, TypeChecker, errors::TypeError, helpers::sanitize_symbol_component,
};
use completion::CheckedType;
use expression_helpers::*;
use generic_types::*;
use x_ast::MAX_AST_EXPRESSION_DEPTH;
use x_ast::MatchArm;
use x_ast::SourceError;
use x_ast::{
    Expression, ExpressionKind, Operator, Parameter, Pattern, Statement, Type, UnaryOperator,
};
use x_diagnostics::frontend_messages::ExecutableTypeReason;
use x_diagnostics::frontend_messages::FrontendMessage;

mod assignment;
mod evaluation;
mod functions;

pub(crate) use expression_helpers::is_integer_literal;
mod generic_types;
pub(crate) use generic_types::substitute_types_in_expression;
mod call_lifetimes;
mod calls;
mod collections;
mod control;
mod expression_helpers;
mod intrinsics;
mod methods;
mod operators;
mod places;
pub(crate) use expression_helpers::is_integer_literal as is_integer_literal_for_context;
pub const MAX_EXPRESSION_DEPTH: usize = MAX_AST_EXPRESSION_DEPTH;

impl TypeChecker {
    pub fn check_expression(&mut self, expression: &mut Expression) -> Result<Type, TypeError> {
        if self.expression_depth >= MAX_EXPRESSION_DEPTH {
            return Err(TypeError::ResourceLimit {
                resource: crate::errors::TypeResource::ExpressionDepth,
                maximum: MAX_EXPRESSION_DEPTH,
            });
        }

        self.expression_depth += 1;
        let result = self.check_expression_inner(expression);
        self.expression_depth -= 1;

        result.map_err(|error| error.at_source(expression.span))
    }
}
