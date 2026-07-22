use std::collections::HashMap;

use crate::{GlobalItem, MirBody, MirProgram, MirStatement, MirTerminator, Rvalue};
use x_ast::intrinsics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedCall {
    Source {
        name: String,
        return_type: Option<x_ast::Type>,
    },
    Builtin(Vec<intrinsics::BuiltinCall>),
}

pub struct CallSite<'a> {
    pub name: &'a str,
    pub arguments: &'a [crate::Operand],
    pub destination: Option<&'a crate::Place>,
}
impl MirBody {
    pub fn call_sites(&self) -> impl Iterator<Item = CallSite<'_>> {
        self.basic_blocks.values().flat_map(|block| {
            block
                .statements
                .iter()
                .filter_map(|statement| match statement {
                    MirStatement::Assign(
                        destination,
                        Rvalue::Call {
                            function,
                            arguments: current_arguments,
                        }
                        | Rvalue::TryCall {
                            function,
                            arguments: current_arguments,
                        }
                        | Rvalue::CatchCall {
                            function,
                            arguments: current_arguments,
                            ..
                        },
                    ) => Some(CallSite {
                        name: function,
                        arguments: current_arguments,
                        destination: Some(destination),
                    }),
                    _ => None,
                })
                .chain(
                    block
                        .terminator
                        .iter()
                        .filter_map(|terminator| match terminator {
                            MirTerminator::Call {
                                function,
                                arguments: current_arguments,
                                destination,
                                ..
                            }
                            | MirTerminator::TryCall {
                                function,
                                arguments: current_arguments,
                                destination,
                                ..
                            }
                            | MirTerminator::CatchCall {
                                function,
                                arguments: current_arguments,
                                destination,
                                ..
                            } => Some(CallSite {
                                name: function,
                                arguments: current_arguments,
                                destination: Some(destination),
                            }),
                            MirTerminator::TailCall {
                                function,
                                arguments: current_arguments,
                            } => Some(CallSite {
                                name: function,
                                arguments: current_arguments,
                                destination: None,
                            }),
                            _ => None,
                        }),
                )
        })
    }
}
