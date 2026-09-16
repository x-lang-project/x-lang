use super::*;

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn compile_invariant_unary(
        &self,
        operator: &UnaryOperator,
        expression: &Expression,
        root: InvariantRoot<'_>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let value = self.compile_invariant_expression(expression, root)?;

        match operator {
            UnaryOperator::LogicalNot | UnaryOperator::BitwiseNot => self
                .b()
                .build_not(value.into_int_value(), "invariant_not")
                .map(Into::into)
                .map_err(Into::into),
            UnaryOperator::Negate if value.is_int_value() => self
                .b()
                .build_int_neg(value.into_int_value(), "invariant_neg")
                .map(Into::into)
                .map_err(Into::into),
            UnaryOperator::Negate if value.is_float_value() => self
                .b()
                .build_float_neg(value.into_float_value(), "invariant_fneg")
                .map(Into::into)
                .map_err(Into::into),
            _ => Err(unsupported(expression)),
        }
    }

    pub(super) fn compile_invariant_binary(
        &self,
        left: &Expression,
        operator: &Operator,
        right: &Expression,
        root: InvariantRoot<'_>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if matches!(operator, Operator::And | Operator::Or) {
            return self.compile_invariant_short_circuit(left, operator, right, root);
        }

        let current_left = self.compile_invariant_expression(left, root)?;
        let current_right = self.compile_invariant_expression(right, root)?;

        self.emit_typed_binary(
            operator,
            numeric::TypedValue {
                value: current_left,
                value_type: &left.value_type,
            },
            numeric::TypedValue {
                value: current_right,
                value_type: &right.value_type,
            },
        )
    }

    pub(super) fn compile_invariant_short_circuit(
        &self,
        left: &Expression,
        operator: &Operator,
        right: &Expression,
        root: InvariantRoot<'_>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        let current_left = self
            .compile_invariant_expression(left, root)?
            .into_int_value();
        let first = self.b().get_insert_block().ok_or_else(|| {
            CodeGenerationError::InvalidMir(backend_messages::invariant_has_no_insertion_block())
        })?;
        let function =
            first.get_parent().ok_or_else(|| {
                CodeGenerationError::InvalidMir(
                    backend_messages::invariant_has_no_containing_function(),
                )
            })?;
        let evaluate = self.context().append_basic_block(function, "invariant_rhs");
        let merge = self
            .context()
            .append_basic_block(function, "invariant_merge");
        let is_and = matches!(operator, Operator::And);

        if is_and {
            self.condition_br(current_left, evaluate, merge)?;
        } else {
            self.condition_br(current_left, merge, evaluate)?;
        }

        self.b().position_at_end(evaluate);
        let current_right = self
            .compile_invariant_expression(right, root)?
            .into_int_value();
        let last = self.b().get_insert_block().ok_or_else(|| {
            CodeGenerationError::InvalidMir(
                backend_messages::invariant_right_has_no_insertion_block(),
            )
        })?;
        self.b().build_unconditional_branch(merge)?;

        self.b().position_at_end(merge);
        let phi = self
            .b()
            .build_phi(self.boolean_type(), "invariant_short_circuit")?;
        let skipped = self.boolean_type().const_int(u64::from(!is_and), false);
        phi.add_incoming(&[(&skipped, first), (&current_right, last)]);

        Ok(phi.as_basic_value())
    }

    pub(super) fn compile_invariant_expression(
        &self,
        expression: &Expression,
        root: InvariantRoot<'_>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenerationError> {
        if let Some(place) = invariant_place(expression, root) {
            let value_type = self.get_place_type(&place);
            let pointer = self.resolve_place_pointer(&place)?;
            let llvm_type = self.codegen.resolve_llvm_type(&value_type)?;

            return self.load(llvm_type, pointer, "invariant_field_load");
        }

        match &expression.kind {
            ExpressionKind::Integer(value) => {
                let value_type = &expression.value_type;

                if !value_type.is_integer() {
                    return Err(unsupported(expression));
                }

                let llvm_type = self.codegen.resolve_llvm_type(value_type)?;

                if !llvm_type.is_int_type() {
                    return Err(unsupported(expression));
                }

                Ok(llvm_type
                    .into_int_type()
                    .const_int(*value as u64, true)
                    .into())
            }
            ExpressionKind::Float(value) => match expression.value_type {
                Type::F32 => Ok(self.context().f32_type().const_float(*value).into()),
                Type::F64 => Ok(self.context().f64_type().const_float(*value).into()),
                _ => Err(unsupported(expression)),
            },
            ExpressionKind::Boolean(value) => Ok(self
                .context()
                .bool_type()
                .const_int(u64::from(*value), false)
                .into()),
            ExpressionKind::Character(value) => Ok(self
                .context()
                .i8_type()
                .const_int(*value as u64, false)
                .into()),
            ExpressionKind::BinaryOperator {
                left,
                operator,
                right,
            } => self.compile_invariant_binary(left, operator, right, root),
            ExpressionKind::UnaryOperator {
                operator,
                expression,
            } => self.compile_invariant_unary(operator, expression, root),
            ExpressionKind::Cast { expression, to } if expression.value_type == *to => {
                self.compile_invariant_expression(expression, root)
            }
            _ => Err(unsupported(expression)),
        }
    }
}
