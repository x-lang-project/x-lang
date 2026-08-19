use std::collections::{HashMap, HashSet};

use super::CheckedType;
use crate::TypeChecker;
use crate::purity::{ImpureOperation, LocalProjection};
use x_ast::StructInitialisation;
use x_ast::{Expression, ExpressionKind, Type};
use x_diagnostics::frontend_messages::FrontendMessage;

pub(super) fn find_declared_field<'a>(
    layout: &'a [(String, Type)],
    requested: &str,
) -> Option<(&'a String, &'a Type)> {
    layout
        .iter()
        .find(|(name, _)| name == requested)
        .map(|(name, value_type)| (name, value_type))
}
