use pest::iterators::Pair;

use x_ast::{GenericParameter, Type};
use x_diagnostics::parser_messages::ExpectedSyntax;

#[derive(Clone, Copy)]
pub(crate) enum LifetimeSpelling {
    Bare,
    Quoted,
}
