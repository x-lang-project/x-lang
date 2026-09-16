mod calls;
use std::borrow::Cow;

use inkwell::module;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values;

use super::FunctionCompiler;
use super::constants::BlockConstants;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{MirStatement, Operand, Place, Rvalue};

struct AssignmentTarget<'place, 'ctx> {
    place: Cow<'place, Place>,
    pointer: values::PointerValue<'ctx>,
    ast_type: Type,
    llvm_type: BasicTypeEnum<'ctx>,
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub fn compile_statement(
        &mut self,
        statement: &MirStatement,
    ) -> Result<(), CodeGenerationError> {
        self.compile_statement_with_constants(statement, None)
    }

    pub(super) fn compile_statement_with_constants(
        &mut self,
        statement: &MirStatement,
        constants: Option<&BlockConstants<'ctx>>,
    ) -> Result<(), CodeGenerationError> {
        match statement {
            MirStatement::StorageLive(_)
            | MirStatement::StorageDead(_)
            | MirStatement::Comment(_) => {}
            MirStatement::ParallelFor {
                start,
                end,
                worker,
                captures,
            } => {
                self.emit_parallel_for(start, end, worker, captures)?;
            }
            MirStatement::Assign(place, rvalue) => {
                self.compile_assignment(place, rvalue, constants)?
            }
        }

        Ok(())
    }

    fn prepare_assignment_target<'place>(
        &mut self,
        place: &'place Place,
        rvalue: &Rvalue,
    ) -> Result<AssignmentTarget<'place, 'ctx>, CodeGenerationError> {
        let mut expected_ast_type = self.get_place_type(place);
        let inferred_rvalue_type = self.infer_rvalue_type(rvalue);
        let should_specialize_target =
            should_specialize_assignment_target(place, &expected_ast_type);

        if should_specialize_target
            && let Some(inferred_type) = inferred_rvalue_type
            && let Place::Local(identifier) = place
        {
            let current_llvm_type = self.codegen.resolve_llvm_type(&inferred_type)?;
            self.replace_local_storage(*identifier, inferred_type.clone(), current_llvm_type)?;
            expected_ast_type = inferred_type;
        }

        if matches!(expected_ast_type, Type::Infer(_) | Type::Unknown)
            && let Some(inferred_type) = self.infer_rvalue_type(rvalue)
        {
            expected_ast_type = inferred_type;
        }

        let effective_place = match place {
            Place::Dereference {
                base,
                value_type: Type::Infer(_) | Type::Unknown,
            } => Cow::Owned(Place::Dereference {
                base: base.clone(),
                value_type: expected_ast_type.clone(),
            }),
            other => Cow::Borrowed(other),
        };

        let target_pointer = self.resolve_place_pointer(place)?;
        let expected_llvm_type = self.codegen.resolve_llvm_type(&expected_ast_type)?;

        Ok(AssignmentTarget {
            place: effective_place,
            pointer: target_pointer,
            ast_type: expected_ast_type,
            llvm_type: expected_llvm_type,
        })
    }

    fn compile_assignment(
        &mut self,
        place: &Place,
        rvalue: &Rvalue,
        constants: Option<&BlockConstants<'ctx>>,
    ) -> Result<(), CodeGenerationError> {
        if let Some(pointers) = self.soa_element_pointers(place)? {
            let current_ast_type = self.get_place_type(place);
            let current_llvm_type = self.codegen.resolve_llvm_type(&current_ast_type)?;
            let value = self.compile_rvalue(rvalue, place)?;
            let value = self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value,
                    value_type: &self.infer_rvalue_type(rvalue).unwrap_or(Type::Unknown),
                },
                &current_ast_type,
                current_llvm_type,
                "soa_assignment_adjust",
            )?;

            return self.store_soa_element(place, &pointers, value);
        }

        if let Rvalue::Call {
            function: callee,
            arguments,
        } = rvalue
        {
            return self.compile_call_assignment(callee, arguments, place);
        }

        let target = self.prepare_assignment_target(place, rvalue)?;

        if matches!(place, Place::Dereference { .. })
            && let Rvalue::Use(
                operand @ (Operand::ConstantInteger(_)
                | Operand::ConstantBoolean(_)
                | Operand::ConstantCharacter(_)
                | Operand::ConstantFloat(_)),
            ) = rvalue
        {
            let value = self.load_operand_value_with_type(operand, &target.ast_type)?;
            let value = self.normalize_value_for_store(
                crate::function::numeric::TypedValue {
                    value,
                    value_type: &target.ast_type,
                },
                &target.ast_type,
                target.llvm_type,
                "deref_scalar_store_adjust",
            )?;
            self.store(target.pointer, value)?;
            self.emit_enclosing_invariants_after_store(place)?;

            return Ok(());
        }

        let constant_initializer = if (target.llvm_type.is_array_type()
            || target.llvm_type.is_struct_type())
            && let Some(constants) = constants
        {
            self.constant_aggregate_initializer(rvalue, &target.ast_type, constants)?
        } else {
            None
        };

        let output_value = if let Some(value) = constant_initializer {
            value
        } else {
            self.compile_rvalue(rvalue, &target.place)?
        };

        if has_incompatible_struct_result(
            target.llvm_type.is_struct_type(),
            output_value.is_struct_value(),
        ) {
            return Err(CodeGenerationError::InvalidMir(
                backend_messages::aggregate_assignment_requires_declared_representation(),
            ));
        }

        let output_value = self.normalize_value_for_store(
            crate::function::numeric::TypedValue {
                value: output_value,
                value_type: &self.infer_rvalue_type(rvalue).unwrap_or(Type::Unknown),
            },
            &target.ast_type,
            target.llvm_type,
            "assign_bitwidth_adjust",
        )?;

        if constant_initializer.is_some() {
            self.copy_constant_initializer(output_value, &target)?;
        } else {
            self.store(target.pointer, output_value)?;
        }

        if let Rvalue::StructInstantiate { name, .. } = rvalue {
            self.emit_struct_invariant(name, &target.place)?;
        }

        self.emit_enclosing_invariants_after_store(place)?;

        Ok(())
    }

    fn copy_constant_initializer(
        &self,
        value: values::BasicValueEnum<'ctx>,
        target: &AssignmentTarget<'_, 'ctx>,
    ) -> Result<(), CodeGenerationError> {
        let initializer = self.m().add_global(
            value.get_type(),
            Some(self.codegen.pointer_address_space()),
            "__x_constant_initializer",
        );
        initializer.set_linkage(module::Linkage::Private);
        initializer.set_constant(true);
        initializer.set_unnamed_address(values::UnnamedAddress::Global);
        initializer.set_initializer(&value);
        let size = value
            .get_type()
            .size_of()
            .ok_or_else(|| CodeGenerationError::TypeHasNoSize(target.ast_type.clone()))?;
        self.b()
            .build_memcpy(target.pointer, 1, initializer.as_pointer_value(), 1, size)?;

        Ok(())
    }
}

fn should_reallocate_call_target(place: &Place, original_place_type: &Type) -> bool {
    should_specialize_assignment_target(place, original_place_type)
}

fn has_incompatible_struct_result(expected_is_struct: bool, output_is_struct: bool) -> bool {
    expected_is_struct && !output_is_struct
}

fn is_aggregate_value(value: values::BasicValueEnum<'_>) -> bool {
    value.is_struct_value() || value.is_array_value()
}

fn should_specialize_assignment_target(place: &Place, expected_type: &Type) -> bool {
    matches!(place, Place::Local(_)) && matches!(expected_type, Type::Infer(_) | Type::Unknown)
}
