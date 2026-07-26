use super::MirLowerer;
use crate::errors::MirLowerError;
use crate::lower::is_codegen_concrete_struct;
use crate::{GlobalItem, Operand};
use x_ast::{Expression, ExpressionKind, Pattern, Type};
use x_diagnostics::backend_messages;
use x_diagnostics::mir_messages;
use x_hir::{HirItem, HirLet};

pub(super) fn checked_static_usize(value: i64, name: &str) -> Result<usize, MirLowerError> {
    usize::try_from(value).map_err(|_| {
        MirLowerError::InvalidStaticInitializer(
            backend_messages::StaticInitializerError::InvalidRepeatLength {
                name: name.to_owned(),
                value,
            },
        )
    })
}

pub(super) fn static_array_length_from_initializer(
    name: &str,
    value: &Expression,
) -> Result<usize, MirLowerError> {
    use backend_messages::StaticInitializerError;

    match &value.kind {
        ExpressionKind::Array(items) => Ok(items.len()),
        ExpressionKind::ArrayRepeat { count, .. } => match &count.kind {
            ExpressionKind::Integer(count) => checked_static_usize(*count, name),
            _ => Err(MirLowerError::InvalidStaticInitializer(
                StaticInitializerError::NonConstantRepeatLength {
                    name: name.to_owned(),
                },
            )),
        },
        _ => Err(MirLowerError::InvalidStaticInitializer(
            StaticInitializerError::MissingArrayInitializer {
                name: name.to_owned(),
            },
        )),
    }
}

pub(super) fn canonical_static_storage_type(
    name: &str,
    type_annotation: Option<&Type>,
    value: &Expression,
) -> Result<Type, MirLowerError> {
    let declared_type = type_annotation.unwrap_or(&value.value_type);

    match declared_type {
        Type::Array(inner) => {
            let length = static_array_length_from_initializer(name, value)?;

            Ok(Type::FixedSizedArray(inner.clone(), length))
        }

        _ => Ok(declared_type.clone()),
    }
}

pub(super) fn static_initializer_operand(value: &Expression) -> Option<Operand> {
    match &value.kind {
        ExpressionKind::Integer(value) => Some(Operand::ConstantInteger(*value)),
        ExpressionKind::Float(value) => Some(Operand::ConstantFloat(*value)),
        ExpressionKind::Boolean(value) => Some(Operand::ConstantBoolean(*value)),
        ExpressionKind::Character(value) => Some(Operand::ConstantCharacter(*value)),
        ExpressionKind::String(value) => {
            crate::lower::literal_string_value(value).map(Operand::ConstantString)
        }
        _ => None,
    }
}

pub(super) fn lower_global_item(
    lowerer: &mut MirLowerer,
    item: &HirItem,
) -> Result<Option<GlobalItem>, MirLowerError> {
    match item {
        HirItem::Struct(definition) if is_codegen_concrete_struct(definition) => {
            Ok(Some(GlobalItem::Struct(definition.clone())))
        }
        HirItem::GlobalLet(HirLet {
            pattern: Pattern::Identifier(name),
            type_annotation,
            value,
        })
        | HirItem::Const {
            name,
            value_type: type_annotation,
            value,
        } => {
            let storage_type =
                canonical_static_storage_type(name, type_annotation.as_ref(), value)?;
            let initializer = static_initializer_operand(value).ok_or_else(|| {
                MirLowerError::UnsupportedExpression(mir_messages::unsupported_static_initializer(
                    name,
                ))
            })?;
            lowerer
                .global_mappings
                .insert(name.clone(), storage_type.clone());

            Ok(Some(GlobalItem::Static {
                name: name.clone(),
                value_type: storage_type,
                initializer: Some(initializer),
                is_constant: matches!(item, HirItem::Const { .. }),
            }))
        }
        _ => Ok(None),
    }
}
