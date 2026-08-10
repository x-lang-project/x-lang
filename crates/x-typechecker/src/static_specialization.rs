mod substitution;
mod traversal;
use std::collections::{HashMap, HashSet};
use std::slice;

use crate::TypeChecker;
use crate::specialization_key::StaticValue;
use substitution::*;
use traversal::*;
use x_ast::DeclarationKey;
use x_ast::Parameter;
use x_ast::SourceError;
use x_ast::SourceSpan;
use x_ast::{Expression, ExpressionKind, Pattern, Statement, StringPart};
use x_diagnostics::frontend_messages::FrontendMessage;

struct StaticTemplate {
    statement: Statement,
    parameters: Vec<Parameter>,
    origin: Option<SourceSpan>,
}
