use std::collections::HashMap;

use crate::MirBody;
use crate::MirStatement;
use crate::MirTerminator;
use crate::Rvalue;
use x_ast::intrinsics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedCall {
    Source {
        name: String,
        return_type: Option<x_ast::Type>,
    },
    Builtin(Vec<intrinsics::BuiltinCall>),
}
