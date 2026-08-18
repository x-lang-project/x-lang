mod references;

use std::collections::HashMap;

use crate::purity::ImpureOperation;

use crate::FunctionSignature;
use crate::TypeChecker;

use x_ast::MAX_AST_EXPRESSION_DEPTH;
use x_ast::MatchArm;
use x_ast::SourceError;
use x_ast::{
    Expression, ExpressionKind, Operator, Parameter, Pattern, Statement, Type, UnaryOperator,
};
use x_diagnostics::frontend_messages::ExecutableTypeReason;
use x_diagnostics::frontend_messages::FrontendMessage;
