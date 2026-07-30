use super::Contexts;
use super::Invariants;
use super::rewrite;
use super::unsupported;
use crate::errors::MirLowerError;
use crate::{MirBody, MirStatement, MirTerminator, Operand, Place, Rvalue};

fn contextual_place(place: &Place, contexts: &Contexts) -> bool {
    let mut identifiers = Vec::new();
    rewrite::local_identifiers(place, &mut identifiers);

    identifiers
        .iter()
        .any(|identifier| contexts.contains_key(identifier))
}

fn contextual_operand(operand: &Operand, contexts: &Contexts) -> bool {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => contextual_place(place, contexts),
        _ => false,
    }
}

fn contextual_rvalue(value: &Rvalue, contexts: &Contexts) -> bool {
    let operator = |value| contextual_operand(value, contexts);

    match value {
        Rvalue::Use(value)
        | Rvalue::UnaryOperator(_, value)
        | Rvalue::Cast {
            expression: value, ..
        } => operator(value),
        Rvalue::BinaryOperator(_, left, right)
        | Rvalue::PointerOffset {
            base: left,
            offset: right,
        } => operator(left) || operator(right),
        Rvalue::AddressOf { expression, .. } => contextual_place(expression, contexts),
        Rvalue::Tuple(values)
        | Rvalue::Array(values)
        | Rvalue::Call {
            arguments: values, ..
        }
        | Rvalue::TryCall {
            arguments: values, ..
        } => values.iter().any(operator),
        Rvalue::CatchCall {
            arguments,
            fallback,
            ..
        } => arguments.iter().any(operator) || operator(fallback),
        Rvalue::StructInstantiate { fields, .. } => fields.iter().any(|(_, value)| operator(value)),
        Rvalue::SizeOf(_) => false,
    }
}

fn validate_terminator(
    terminator: &MirTerminator,
    contexts: &Contexts,
) -> Result<(), MirLowerError> {
    let invalid = match terminator {
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
            contextual_place(destination, contexts)
                || arguments
                    .iter()
                    .any(|value| contextual_operand(value, contexts))
                || matches!(terminator, MirTerminator::CatchCall { fallback, .. } if contextual_operand(fallback, contexts))
        }
        MirTerminator::TailCall { arguments, .. } => arguments
            .iter()
            .any(|value| contextual_operand(value, contexts)),
        MirTerminator::SwitchInteger { discriminant, .. } => {
            contextual_operand(discriminant, contexts)
        }
        MirTerminator::Throw { error } => contextual_operand(error, contexts),
        MirTerminator::Exit { code } => contextual_operand(code, contexts),
        _ => false,
    };

    if invalid {
        Err(unsupported(
            "dependent references cannot cross control-flow or call ABI boundaries",
        ))
    } else {
        Ok(())
    }
}
