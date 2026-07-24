use std::collections::HashMap;

use super::{
    Analysis, access,
    model::{State, entry},
    values,
};
use crate::{BlockIdentifier, MirStatement, MirTerminator};

fn rvalue_call(value: &crate::Rvalue) -> Option<crate::CallSite<'_>> {
    match value {
        crate::Rvalue::Call {
            function,
            arguments: current_arguments,
        }
        | crate::Rvalue::TryCall {
            function,
            arguments: current_arguments,
        }
        | crate::Rvalue::CatchCall {
            function,
            arguments: current_arguments,
            ..
        } => Some(crate::CallSite {
            name: function,
            arguments: current_arguments,
            destination: None,
        }),
        _ => None,
    }
}

fn terminator_call(value: &MirTerminator) -> Option<crate::CallSite<'_>> {
    match value {
        MirTerminator::Call {
            function,
            arguments: current_arguments,
            ..
        }
        | MirTerminator::TryCall {
            function,
            arguments: current_arguments,
            ..
        }
        | MirTerminator::CatchCall {
            function,
            arguments: current_arguments,
            ..
        } => Some(crate::CallSite {
            name: function,
            arguments: current_arguments,
            destination: None,
        }),
        _ => None,
    }
}
