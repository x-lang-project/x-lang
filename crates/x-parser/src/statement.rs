use pest::iterators::Pair;

use x_ast::{Expression, ExpressionKind, Pattern, Statement, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;
use x_diagnostics::parser_messages::ParserIssue;

pub(crate) mod generics;
