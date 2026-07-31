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
impl MirProgram {
    pub fn resolve_call_targets(&mut self) {
        fn names(items: &[GlobalItem], output: &mut HashMap<String, x_ast::Type>) {
            for item in items {
                match item {
                    GlobalItem::Function {
                        name, return_type, ..
                    } => {
                        output.insert(name.clone(), return_type.clone());
                    }
                    GlobalItem::Impl { methods, .. } => names(methods, output),
                    _ => {}
                }
            }
        }
        fn resolve(items: &mut [GlobalItem], sources: &HashMap<String, x_ast::Type>) {
            for item in items {
                match item {
                    GlobalItem::Function {
                        body: Some(body), ..
                    } => body.resolve_call_targets(sources),
                    GlobalItem::Impl { methods, .. } => resolve(methods, sources),
                    _ => {}
                }
            }
        }
        let mut sources = HashMap::new();
        names(&self.items, &mut sources);
        resolve(&mut self.items, &sources);
    }
}

impl MirBody {
    fn resolve_call_targets(&mut self, sources: &HashMap<String, x_ast::Type>) {
        let mut targets = HashMap::new();
        let mut record = |name: &str,
                          current_arguments: &[crate::Operand],
                          destination: Option<&crate::Place>| {
            if let Some(return_type) = sources.get(name) {
                targets.insert(
                    name.to_string(),
                    ResolvedCall::Source {
                        name: name.into(),
                        return_type: Some(return_type.clone()),
                    },
                );
                return;
            }

            let arguments = current_arguments
                .iter()
                .map(|operand| self.operand_type(operand))
                .collect::<Vec<_>>();
            let result = Some(destination.map_or_else(
                || self.place_type(&crate::Place::Local(self.return_local)),
                |place| self.place_type(place),
            ));

            if let Some(identity) =
                intrinsics::BuiltinCall::resolve(name, &arguments, result.as_ref())
            {
                let target = targets
                    .entry(name.to_string())
                    .or_insert_with(|| ResolvedCall::Builtin(Vec::new()));

                if let ResolvedCall::Builtin(signatures) = target
                    && !signatures.contains(&identity)
                {
                    signatures.push(identity);
                }
            } else {
                targets.insert(
                    name.to_string(),
                    ResolvedCall::Source {
                        name: name.into(),
                        return_type: result,
                    },
                );
            }
        };

        for call in self.call_sites() {
            record(call.name, call.arguments, call.destination);
        }
        self.call_targets = targets;
    }
}

impl MirBody {
    pub fn builtin_call(
        &self,
        name: &str,
        current_arguments: &[crate::Operand],
        value_type: Option<&x_ast::Type>,
    ) -> Option<&intrinsics::BuiltinCall> {
        let ResolvedCall::Builtin(signatures) = self.call_targets.get(name)? else {
            return None;
        };

        let arguments = current_arguments
            .iter()
            .map(|operand| self.operand_type(operand))
            .collect::<Vec<_>>();
        let mut matches = signatures.iter().filter(|identity| {
            identity.argument_types == arguments
                && value_type
                    .is_none_or(|value_type| identity.return_type.as_ref() == Some(value_type))
        });
        let first = matches.next()?;

        if matches.next().is_some() {
            None
        } else {
            Some(first)
        }
    }
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
