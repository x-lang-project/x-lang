use std::collections::BTreeMap;

use super::Function;
use crate::{MirStatement, MirTerminator, Operand, Place, Rvalue};
use x_ast::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LoweredMemoryEffects {
    pub cache: bool,
    pub arguments: bool,
    pub globals: bool,
    pub opaque_calls: bool,
    pub unproved_memory: bool,
}

impl LoweredMemoryEffects {
    pub const fn permits_memory_none(self) -> bool {
        !(self.cache || self.arguments || self.globals || self.opaque_calls || self.unproved_memory)
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            cache: self.cache || other.cache,
            arguments: self.arguments || other.arguments,
            globals: self.globals || other.globals,
            opaque_calls: self.opaque_calls || other.opaque_calls,
            unproved_memory: self.unproved_memory || other.unproved_memory,
        }
    }
}

fn memory_parameter(value_type: &Type) -> bool {
    match value_type {
        Type::Reference { .. }
        | Type::RawPointer { .. }
        | Type::String
        | Type::Array(_)
        | Type::Custom(_)
        | Type::GenericInstance { .. }
        | Type::Function { .. }
        | Type::Unknown
        | Type::Infer(_)
        | Type::TypeParameter(_) => true,
        Type::FixedSizedArray(inner, _) | Type::Simd { element: inner, .. } => {
            memory_parameter(inner)
        }
        Type::Tuple(fields) => fields.iter().any(memory_parameter),
        _ => false,
    }
}

fn global_place(place: &Place) -> bool {
    match place {
        Place::Global { .. } => true,
        Place::Field { base, .. } | Place::Dereference { base, .. } => global_place(base),
        Place::Index { base, index, .. } => global_place(base) || global_operand(index),
        Place::Local(_) => false,
    }
}

fn global_terminator(term: &MirTerminator) -> bool {
    match term {
        MirTerminator::SwitchInteger { discriminant, .. } => global_operand(discriminant),
        MirTerminator::Call {
            arguments,
            destination,
            ..
        }
        | MirTerminator::TryCall {
            arguments,
            destination,
            ..
        } => arguments.iter().any(global_operand) || global_place(destination),
        MirTerminator::CatchCall {
            arguments,
            destination,
            fallback,
            ..
        } => {
            arguments.iter().any(global_operand)
                || global_place(destination)
                || global_operand(fallback)
        }
        MirTerminator::TailCall { arguments, .. } => arguments.iter().any(global_operand),
        MirTerminator::Throw { error } => global_operand(error),
        MirTerminator::Exit { code } => global_operand(code),
        _ => false,
    }
}

fn global_operand(value: &Operand) -> bool {
    match value {
        Operand::Copy(place) | Operand::Move(place) => global_place(place),
        _ => false,
    }
}

fn global_rvalue(value: &Rvalue) -> bool {
    match value {
        Rvalue::Use(value)
        | Rvalue::UnaryOperator(_, value)
        | Rvalue::Cast {
            expression: value, ..
        } => global_operand(value),
        Rvalue::BinaryOperator(_, first_value, second_value)
        | Rvalue::PointerOffset {
            base: first_value,
            offset: second_value,
        } => global_operand(first_value) || global_operand(second_value),
        Rvalue::AddressOf { expression, .. } => global_place(expression),
        Rvalue::Tuple(items) | Rvalue::Array(items) => items.iter().any(global_operand),
        Rvalue::StructInstantiate { fields, .. } => {
            fields.iter().any(|(_, value)| global_operand(value))
        }
        Rvalue::Call { arguments, .. } | Rvalue::TryCall { arguments, .. } => {
            arguments.iter().any(global_operand)
        }
        Rvalue::CatchCall {
            arguments,
            fallback,
            ..
        } => arguments.iter().any(global_operand) || global_operand(fallback),
        Rvalue::SizeOf(_) => false,
    }
}

pub(super) fn transitive_effects(
    functions: &[Function<'_>],
    unproved: &BTreeMap<String, bool>,
) -> BTreeMap<String, LoweredMemoryEffects> {
    let mut direct = BTreeMap::new();

    for function in functions {
        let globals = function.body.is_some_and(|body| {
            body.basic_blocks.values().any(|block| {
                block.statements.iter().any(|statement| match statement {
                    MirStatement::Assign(place, value) => {
                        global_place(place) || global_rvalue(value)
                    }
                    MirStatement::ParallelFor {
                        start,
                        end,
                        captures,
                        ..
                    } => {
                        global_operand(start)
                            || global_operand(end)
                            || captures.iter().any(global_operand)
                    }
                    _ => false,
                }) || block.terminator.as_ref().is_some_and(global_terminator)
            })
        });
        direct.insert(
            function.name.to_owned(),
            LoweredMemoryEffects {
                cache: function.memoised || function.body.is_some_and(|body| body.is_memoised),
                arguments: function.parameters.iter().any(memory_parameter),
                globals,
                opaque_calls: function.body.is_none() || !function.pure,
                unproved_memory: unproved.get(function.name).copied().unwrap_or(true),
            },
        );
    }

    let mut summaries = direct.clone();

    loop {
        let previous = summaries.clone();

        for function in functions {
            let Some(body) = function.body else { continue };

            let mut effect = direct.get(function.name).copied().unwrap_or_default();

            for call in body.call_sites() {
                effect = effect.union(previous.get(call.name).copied().unwrap_or(
                    LoweredMemoryEffects {
                        opaque_calls: true,
                        ..LoweredMemoryEffects::default()
                    },
                ));
            }
            // Generated parallel workers have effects without a Call node.
            for block in body.basic_blocks.values() {
                for statement in &block.statements {
                    if let MirStatement::ParallelFor { worker, .. } = statement {
                        effect = effect.union(previous.get(worker).copied().unwrap_or(
                            LoweredMemoryEffects {
                                opaque_calls: true,
                                ..LoweredMemoryEffects::default()
                            },
                        ));
                    }
                }
            }

            summaries.insert(function.name.into(), effect);
        }

        if summaries == previous {
            return summaries;
        }
    }
}
