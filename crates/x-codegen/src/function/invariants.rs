mod dependencies;
mod expressions;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use super::numeric;
use crate::errors::CodeGenerationError;
use x_ast::Operator;
use x_ast::{Expression, ExpressionKind, Type, UnaryOperator};
use x_diagnostics::backend_messages;
use x_mir::Place;

impl FunctionCompiler<'_, '_> {
    pub(super) fn emit_enclosing_invariants_after_store(
        &self,
        destination: &Place,
    ) -> Result<(), CodeGenerationError> {
        for owner in destination.invariant_owner_chain().into_iter().skip(1) {
            if self.invariant_depends_on_write(owner, destination) {
                self.emit_invariant_for_root(owner)?;
            }
        }

        Ok(())
    }

    pub(super) fn emit_invariant_for_root(&self, root: &Place) -> Result<(), CodeGenerationError> {
        let mut root_type = self.get_place_type(root);

        while let Type::Reference { inner, .. } | Type::RawPointer { inner, .. } = root_type {
            root_type = *inner;
        }

        let name = match root_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return Ok(()),
        };

        self.emit_struct_invariant(&name, root)
    }

    pub(super) fn emit_struct_invariant(
        &self,
        struct_name: &str,
        root: &Place,
    ) -> Result<(), CodeGenerationError> {
        let Some(invariant) = self
            .codegen
            .resolve_struct_name(struct_name)
            .and_then(|name| self.codegen.records.get(name))
            .or_else(|| self.codegen.records.get(struct_name))
            .and_then(|record| record.invariant())
        else {
            return Ok(());
        };

        let condition = self.compile_invariant_expression(
            &invariant.predicate,
            InvariantRoot {
                place: root,
                receiver: &invariant.receiver,
            },
        )?;

        if !condition.is_int_value() || condition.into_int_value().get_type().get_bit_width() != 1 {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::struct_invariant_for_struct_name_did_not_lower_to_boolean(
                    struct_name,
                ),
            ));
        }

        self.trap_if_false(condition.into_int_value(), "struct_invariant")
    }
}

#[derive(Clone, Copy)]
struct InvariantRoot<'a> {
    place: &'a Place,
    receiver: &'a x_ast::InvariantBinding,
}

fn invariant_place(expression: &Expression, root: InvariantRoot<'_>) -> Option<Place> {
    match &expression.kind {
        _ if root.receiver.refers_to(expression) => Some(root.place.clone()),
        ExpressionKind::FieldAccess { object, field } => Some(Place::Field {
            base: Box::new(invariant_place(object, root)?),
            field_name: field.clone(),
            value_type: expression.value_type.clone(),
        }),
        _ => None,
    }
}

fn unsupported(expression: &Expression) -> CodeGenerationError {
    CodeGenerationError::InvalidMir(
        backend_messages::struct_invariant_contains_an_unsupported_executable_expression_value(
            &expression.kind,
        ),
    )
}
