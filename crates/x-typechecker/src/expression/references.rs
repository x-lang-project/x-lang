use crate::{TypeChecker, errors::TypeError};
use x_ast::{Expression, ExpressionKind, Layout, Type};
use x_diagnostics::frontend_messages::ExecutableTypeReason;

impl TypeChecker {
    pub(super) fn check_dereference(
        &mut self,
        inner: &mut Expression,
        annotation: &mut Type,
    ) -> Result<Type, TypeError> {
        let inner_type = self.check_expression(inner)?;
        let resolved_inner = self.resolve(&inner_type);
        inner.value_type = resolved_inner.clone();
        let target = match resolved_inner {
            Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => *inner,
            value_type => {
                return Err(TypeError::UnresolvedProperty {
                    field: "*deref".into(),
                    value_type,
                });
            }
        };

        *annotation = self.resolve(&target);

        Ok(annotation.clone())
    }

    pub(super) fn check_address_of(
        &mut self,
        inner: &mut Expression,
        is_mut: bool,
        is_unique: bool,
    ) -> Result<Type, TypeError> {
        let inner_type = self.check_expression(inner)?;
        inner.value_type = inner_type.clone();
        self.validate_borrowed_place(inner, is_mut)?;

        Ok(Type::Reference {
            lifetime: None,
            is_mut,
            is_unique,
            inner: Box::new(inner_type),
        })
    }

    pub(in crate::expression) fn validate_borrowed_place(
        &mut self,
        target: &Expression,
        is_mut: bool,
    ) -> Result<(), TypeError> {
        self.check_soa_whole_address(target)?;
        if is_mut {
            self.check_invariant_field_borrow(target)?;
        }

        Ok(())
    }
}

impl TypeChecker {
    fn check_soa_whole_address(&self, target: &Expression) -> Result<(), TypeError> {
        if matches!(target.kind, ExpressionKind::ArrayAccess { .. })
            && self.is_soa_struct_type(&target.value_type)
        {
            return Err(TypeError::ExecutableType {
                value_type: target.value_type.clone(),
                reason: ExecutableTypeReason::SoaWholeValueAddress,
            });
        }

        Ok(())
    }

    fn is_soa_struct_type(&self, value_type: &Type) -> bool {
        let name = match value_type {
            Type::Custom(name) | Type::GenericInstance { name, .. } => name,
            _ => return false,
        };

        self.symbols.records.layout(name) == Some(&Layout::SoA)
    }
}
