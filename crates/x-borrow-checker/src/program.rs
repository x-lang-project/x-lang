use std::collections::HashMap;
use std::sync::Arc;

use super::return_summaries;
use x_ast::{
    DeclarationKey, ExpressionKind, Pattern, Program, SourceError, SourceSpan, Statement, Type,
};

pub(super) fn nominal_owner_name(value_type: &Type) -> String {
    match value_type {
        _ => value_type.to_string(),
    }
}
