mod completion;

mod literals;
mod references;

use std::collections::HashMap;

use crate::purity::ImpureOperation;

use crate::FunctionSignature;
use crate::TypeChecker;
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

mod evaluation;

pub(crate) use expression_helpers::is_integer_literal;
mod generic_types;
pub(crate) use generic_types::substitute_types_in_expression;
mod call_lifetimes;
mod calls;

mod expression_helpers;
mod intrinsics;
mod methods;
mod operators;
mod places;
pub(crate) use expression_helpers::is_integer_literal as is_integer_literal_for_context;
pub const MAX_EXPRESSION_DEPTH: usize = MAX_AST_EXPRESSION_DEPTH;
