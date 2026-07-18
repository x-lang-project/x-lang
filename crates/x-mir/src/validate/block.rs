use super::{MirValidationError, validate_type};
use crate::{
    BasicBlock, BlockIdentifier, LocalIdentifier, MirBody, MirStatement, MirTerminator, Operand,
    Place, Rvalue,
};
use x_diagnostics::backend_messages;

pub(super) struct BlockValidator<'a> {
    pub(super) function: &'a str,
    pub(super) block: BlockIdentifier,
    pub(super) body: &'a MirBody,
}

impl BlockValidator<'_> {
    pub(super) fn validate_rvalue(&self, value: &Rvalue) -> Result<(), MirValidationError> {
        match value {
            Rvalue::Use(value) | Rvalue::UnaryOperator(_, value) => self.validate_operand(value),
            Rvalue::BinaryOperator(_, left, right)
            | Rvalue::PointerOffset {
                base: left,
                offset: right,
            } => {
                self.validate_operand(left)?;

                self.validate_operand(right)
            }
            Rvalue::Cast { expression, to } => {
                self.validate_operand(expression)?;

                validate_type(to, &|| {
                    backend_messages::function_function_block_value_cast(
                        self.function,
                        self.block.0,
                    )
                })
            }
            Rvalue::AddressOf { expression, .. } => self.validate_place(expression),
            Rvalue::Tuple(values) | Rvalue::Array(values) => self.validate_operands(values),
            Rvalue::SizeOf(value_type) => validate_type(value_type, &|| {
                backend_messages::function_function_block_value_sizeof(self.function, self.block.0)
            }),
            Rvalue::Call { arguments, .. } | Rvalue::TryCall { arguments, .. } => {
                self.validate_operands(arguments)
            }
            Rvalue::CatchCall {
                arguments,
                fallback,
                ..
            } => {
                self.validate_operands(arguments)?;

                self.validate_operand(fallback)
            }
            Rvalue::StructInstantiate { fields, .. } => {
                for (_, value) in fields {
                    self.validate_operand(value)?;
                }

                Ok(())
            }
        }
    }

    fn validate_operands(&self, values: &[Operand]) -> Result<(), MirValidationError> {
        for value in values {
            self.validate_operand(value)?;
        }

        Ok(())
    }

    pub(super) fn validate_operand(&self, value: &Operand) -> Result<(), MirValidationError> {
        match value {
            Operand::Copy(place) | Operand::Move(place) => self.validate_place(place),
            Operand::ConstantCharacter(value) if u8::try_from(u32::from(*value)).is_err() => {
                Err(MirValidationError::ByteCharacterOutOfRange {
                    function: self.function.to_string(),
                    block: self.block.0,
                })
            }
            Operand::ConstantFloat(value) if !value.is_finite() => {
                Err(MirValidationError::NonFiniteFloat {
                    function: self.function.to_string(),
                    block: self.block.0,
                })
            }
            Operand::TypeLiteral(value_type) => validate_type(value_type, &|| {
                backend_messages::function_function_block_value_type_literal(
                    self.function,
                    self.block.0,
                )
            }),
            _ => Ok(()),
        }
    }

    pub(super) fn validate_place(&self, place: &Place) -> Result<(), MirValidationError> {
        match place {
            Place::Local(local) => self.validate_local(*local),
            Place::Global { value_type, .. } => validate_type(value_type, &|| {
                backend_messages::function_function_block_value_global(self.function, self.block.0)
            }),
            Place::Field {
                base, value_type, ..
            }
            | Place::Dereference { base, value_type } => {
                self.validate_place(base)?;

                validate_type(value_type, &|| {
                    backend_messages::function_function_block_value_place(
                        self.function,
                        self.block.0,
                    )
                })
            }
            Place::Index {
                base,
                index,
                value_type,
            } => {
                self.validate_place(base)?;
                self.validate_operand(index)?;

                validate_type(value_type, &|| {
                    backend_messages::function_function_block_value_index(
                        self.function,
                        self.block.0,
                    )
                })
            }
        }
    }

    fn validate_local(&self, local: LocalIdentifier) -> Result<(), MirValidationError> {
        if self.body.locals.contains_key(&local) {
            Ok(())
        } else {
            Err(MirValidationError::MissingLocal {
                function: self.function.to_string(),
                block: self.block.0,
                local: local.0,
            })
        }
    }

    fn validate_target(&self, target: BlockIdentifier) -> Result<(), MirValidationError> {
        if self.body.basic_blocks.contains_key(&target) {
            Ok(())
        } else {
            Err(MirValidationError::MissingTargetBlock {
                function: self.function.to_string(),
                block: self.block.0,
                target: target.0,
            })
        }
    }
}
