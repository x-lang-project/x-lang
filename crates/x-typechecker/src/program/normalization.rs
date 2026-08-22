use crate::{TypeChecker, errors::TypeError};
use x_ast::{Expression, ExpressionKind, Statement, Type, TypeVisitorMut, UnaryOperator};

impl TypeChecker {
    // Executable types must use resolved storage identities.
    pub(super) fn normalize_checked_types(
        &mut self,
        statements: &mut [Statement],
    ) -> Result<(), TypeError> {
        let mut normalizer = CheckedTypeNormalizer {
            checker: self,
            error: None,
        };
        normalizer.visit_statements(statements);
        if let Some(error) = normalizer.error {
            return Err(error);
        }

        self.validate_record_storage_cycles()
    }
}

struct CheckedTypeNormalizer<'a> {
    checker: &'a mut TypeChecker,
    error: Option<TypeError>,
}

impl TypeVisitorMut for CheckedTypeNormalizer<'_> {
    fn visit_type(&mut self, value_type: &mut Type) {
        if self.error.is_some() {
            return;
        }

        match self.checker.resolve_type(value_type) {
            Ok(resolved) => *value_type = resolved,
            Err(error) => self.error = Some(error),
        }
    }

    fn visit_call_target(&mut self, target: &mut x_ast::CallTarget) {
        if let Some(x_ast::CallResolution::Builtin { type_arguments, .. }) = &mut target.resolution
        {
            for argument in type_arguments {
                self.visit_type(argument);
            }
        }
    }

    fn visit_statement(&mut self, statement: &mut Statement) {
        match statement {
            // Normalise templates after their generic arguments are bound.
            Statement::Function {
                generic_parameters, ..
            } if generic_parameters.iter().any(|parameter| {
                !matches!(parameter, x_ast::GenericParameter::Lifetime { .. })
            }) => {}
            Statement::StructDeclaration(definition) if definition.generic_parameters.is_some() => {
            }
            Statement::StructDeclaration(definition) => {
                for (_, value_type) in &mut definition.fields {
                    self.visit_type(value_type);
                }

                if let Some(invariant) = &mut definition.invariant {
                    self.visit_expression(&mut invariant.predicate);
                }
            }
            Statement::ImplDeclaration(definition) if definition.generic_parameters.is_empty() => {
                self.visit_statements(&mut definition.methods);
            }
            _ => x_ast::walk_statement_types_mut(self, statement),
        }
    }

    fn visit_expression(&mut self, expression: &mut Expression) {
        if let ExpressionKind::UnaryOperator {
            operator: UnaryOperator::Negate,
            expression: inner,
        } = &mut expression.kind
            && let ExpressionKind::Integer(value) = inner.kind
        {
            self.visit_type(&mut expression.value_type);
            inner.value_type = expression.value_type.clone();
            self.validate_integer_literal(-i128::from(value), &expression.value_type);
            return;
        }

        x_ast::walk_expression_types_mut(self, expression);
        if let ExpressionKind::Integer(value) = expression.kind {
            self.validate_integer_literal(i128::from(value), &expression.value_type);
        }

        if let ExpressionKind::StructInstantiate(initialise) = &mut expression.kind
            && let Type::Custom(name) = &expression.value_type
            && self.checker.symbols.records.fields().contains_key(name)
        {
            initialise.name = name.clone();
        }
    }
}

impl CheckedTypeNormalizer<'_> {
    fn validate_integer_literal(&mut self, value: i128, value_type: &Type) {
        let range = match value_type {
            Type::I8 => Some((i128::from(i8::MIN), i128::from(i8::MAX))),
            Type::I16 => Some((i128::from(i16::MIN), i128::from(i16::MAX))),
            Type::I32 => Some((i128::from(i32::MIN), i128::from(i32::MAX))),
            Type::I64 => Some((i128::from(i64::MIN), i128::from(i64::MAX))),
            Type::U8 | Type::Character => Some((0, i128::from(u8::MAX))),
            Type::U16 => Some((0, i128::from(u16::MAX))),
            Type::U32 => Some((0, i128::from(u32::MAX))),
            Type::U64 => Some((0, i128::from(u64::MAX))),
            // Pointer-sized bounds are checked with the target layout.
            _ => None,
        };

        if let Some((minimum, maximum)) = range
            && (value < minimum || value > maximum)
            && self.error.is_none()
        {
            self.error = Some(crate::type_mismatch!(
                format!("integer literal in {minimum}..={maximum} for {value_type}"),
                value.to_string()
            ));
        }
    }
}
