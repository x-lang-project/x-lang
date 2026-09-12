mod aggregates;
mod arrays;
use std::collections::HashMap;

use inkwell::types;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use x_ast::Type;
use x_diagnostics::backend_messages;
use x_mir::{LocalIdentifier, MirStatement, Operand, Place, Rvalue};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn constant_aggregate_initializer(
        &self,
        rvalue: &Rvalue,
        target_type: &Type,
        constants: &BlockConstants<'ctx>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        let value = self.literal_rvalue(rvalue, target_type, &constants.values)?;

        Ok(value.filter(|value| value.is_array_value() || value.is_struct_value()))
    }

    fn literal_operand(
        &self,
        operand: &Operand,
        value_type: &Type,
        constants: &HashMap<LocalIdentifier, BasicValueEnum<'ctx>>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => match &**place {
                Place::Local(identifier) => Ok(constants.get(identifier).copied()),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn literal_rvalue(
        &self,
        rvalue: &Rvalue,
        value_type: &Type,
        constants: &HashMap<LocalIdentifier, BasicValueEnum<'ctx>>,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CodeGenerationError> {
        match rvalue {
            Rvalue::Use(operand) => self.literal_operand(operand, value_type, constants),

            Rvalue::Tuple(elements) => self.literal_tuple(elements, value_type, constants),

            _ => Ok(None),
        }
    }
}

#[derive(Default)]
pub(super) struct BlockConstants<'ctx> {
    values: HashMap<LocalIdentifier, BasicValueEnum<'ctx>>,
}

impl<'ctx> BlockConstants<'ctx> {
    fn observe(
        &mut self,
        compiler: &FunctionCompiler<'_, 'ctx>,
        statement: &MirStatement,
    ) -> Result<(), CodeGenerationError> {
        match statement {
            MirStatement::Assign(Place::Local(identifier), rvalue) => {
                if matches!(
                    rvalue,
                    Rvalue::Call { .. }
                        | Rvalue::TryCall { .. }
                        | Rvalue::CatchCall { .. }
                        | Rvalue::AddressOf { .. }
                ) {
                    self.values.clear();
                }

                let value_type = compiler.get_place_type(&Place::Local(*identifier));

                if let Some(value) = compiler.literal_rvalue(rvalue, &value_type, &self.values)? {
                    self.values.insert(*identifier, value);
                } else {
                    self.values.remove(identifier);
                }
            }
            // Projected writes can alias any cached local.
            MirStatement::Assign(_, _) => self.values.clear(),
            MirStatement::ParallelFor { .. } => {
                self.values.clear();
            }
            MirStatement::StorageDead(identifier) => {
                self.values.remove(identifier);
            }
            MirStatement::StorageLive(_) | MirStatement::Comment(_) => {}
        }

        Ok(())
    }
}
