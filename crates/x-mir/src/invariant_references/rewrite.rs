use super::Contexts;
use crate::{MirBody, MirStatement, MirTerminator, Operand, Place, Rvalue};

pub(super) fn place(place: &Place, contexts: &Contexts) -> Place {
    match place {
        Place::Dereference { base, value_type } => {
            if let Place::Local(identifier) = base.as_ref()
                && let Some(target) = contexts.get(identifier)
            {
                return target.clone();
            }

            Place::Dereference {
                base: Box::new(self::place(base, contexts)),
                value_type: value_type.clone(),
            }
        }
        Place::Field {
            base,
            field_name,
            value_type,
        } => Place::Field {
            base: Box::new(implicit_base(base, contexts)),
            field_name: field_name.clone(),
            value_type: value_type.clone(),
        },
        Place::Index {
            base,
            index,
            value_type,
        } => Place::Index {
            base: Box::new(implicit_base(base, contexts)),
            index: operand(index, contexts),
            value_type: value_type.clone(),
        },
        other => other.clone(),
    }
}

fn implicit_base(base: &Place, contexts: &Contexts) -> Place {
    if let Place::Local(identifier) = base
        && let Some(target) = contexts.get(identifier)
    {
        target.clone()
    } else {
        place(base, contexts)
    }
}

pub(super) fn operand(value: &Operand, contexts: &Contexts) -> Operand {
    match value {
        Operand::Copy(target) => Operand::Copy(Box::new(place(target, contexts))),
        Operand::Move(target) => Operand::Move(Box::new(place(target, contexts))),
        other => other.clone(),
    }
}

pub(super) fn rvalue(value: &Rvalue, contexts: &Contexts) -> Rvalue {
    let current_operator = |value: &Operand| operand(value, contexts);
    let operators = |values: &[Operand]| {
        values
            .iter()
            .map(|value| operand(value, contexts))
            .collect()
    };

    match value {
        Rvalue::Use(value) => Rvalue::Use(current_operator(value)),
        Rvalue::UnaryOperator(operator, value) => {
            Rvalue::UnaryOperator(operator.clone(), current_operator(value))
        }
        Rvalue::BinaryOperator(operator, left, right) => Rvalue::BinaryOperator(
            operator.clone(),
            current_operator(left),
            current_operator(right),
        ),
        Rvalue::PointerOffset { base, offset } => Rvalue::PointerOffset {
            base: current_operator(base),
            offset: current_operator(offset),
        },
        Rvalue::Cast { expression, to } => Rvalue::Cast {
            expression: current_operator(expression),
            to: to.clone(),
        },
        Rvalue::AddressOf {
            is_mut,
            is_unique,
            expression,
        } => Rvalue::AddressOf {
            is_mut: *is_mut,
            is_unique: *is_unique,
            expression: place(expression, contexts),
        },
        Rvalue::Tuple(values) => Rvalue::Tuple(operators(values)),
        Rvalue::Array(values) => Rvalue::Array(operators(values)),
        Rvalue::Call {
            function,
            arguments,
        } => Rvalue::Call {
            function: function.clone(),
            arguments: operators(arguments),
        },
        Rvalue::TryCall {
            function,
            arguments,
        } => Rvalue::TryCall {
            function: function.clone(),
            arguments: operators(arguments),
        },
        Rvalue::CatchCall {
            function,
            arguments,
            fallback,
        } => Rvalue::CatchCall {
            function: function.clone(),
            arguments: operators(arguments),
            fallback: current_operator(fallback),
        },
        Rvalue::StructInstantiate { name, fields } => Rvalue::StructInstantiate {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), current_operator(value)))
                .collect(),
        },
        Rvalue::SizeOf(value_type) => Rvalue::SizeOf(value_type.clone()),
    }
}

pub(super) fn rewrite_body(body: &mut MirBody, contexts: &Contexts) {
    for block in body.basic_blocks.values_mut() {
        for statement in &mut block.statements {
            match statement {
                MirStatement::Assign(destination, value) => {
                    *destination = place(destination, contexts);
                    *value = rvalue(value, contexts);
                }
                MirStatement::ParallelFor {
                    start,
                    end,
                    captures,
                    ..
                } => {
                    *start = operand(start, contexts);
                    *end = operand(end, contexts);

                    for capture in captures {
                        *capture = operand(capture, contexts);
                    }
                }
                _ => {}
            }
        }

        if let Some(terminator) = &mut block.terminator {
            rewrite_terminator(terminator, contexts);
        }
    }
}

fn rewrite_terminator(terminator: &mut MirTerminator, contexts: &Contexts) {
    match terminator {
        MirTerminator::Call {
            arguments,
            destination,
            ..
        }
        | MirTerminator::TryCall {
            arguments,
            destination,
            ..
        }
        | MirTerminator::CatchCall {
            arguments,
            destination,
            ..
        } => {
            for argument in arguments {
                *argument = operand(argument, contexts);
            }

            let _ = destination;

            if let MirTerminator::CatchCall { fallback, .. } = terminator {
                *fallback = operand(fallback, contexts);
            }
        }
        MirTerminator::TailCall { arguments, .. } => {
            for argument in arguments {
                *argument = operand(argument, contexts);
            }
        }
        MirTerminator::SwitchInteger { discriminant, .. } => {
            *discriminant = operand(discriminant, contexts)
        }
        MirTerminator::Throw { error } => *error = operand(error, contexts),
        MirTerminator::Exit { code } => *code = operand(code, contexts),
        _ => {}
    }
}

pub(super) fn local_identifiers(place: &Place, identifiers: &mut Vec<crate::LocalIdentifier>) {
    match place {
        Place::Local(identifier) => identifiers.push(*identifier),
        Place::Field { base, .. } | Place::Dereference { base, .. } => {
            local_identifiers(base, identifiers)
        }
        Place::Index { base, index, .. } => {
            local_identifiers(base, identifiers);

            if let Operand::Copy(place) | Operand::Move(place) = index {
                local_identifiers(place, identifiers);
            }
        }
        _ => {}
    }
}
