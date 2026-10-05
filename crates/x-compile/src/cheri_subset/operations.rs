use super::*;

pub(super) fn validate_statement(
    statement: &MirStatement,
    function: &str,
) -> Result<(), CompilationFailure> {
    match statement {
        MirStatement::Assign(place, value) => {
            validate_place(place, function)?;

            validate_rvalue(value, function)
        }
        MirStatement::StorageLive(_) | MirStatement::StorageDead(_) | MirStatement::Comment(_) => {
            Ok(())
        }
        MirStatement::ParallelFor { .. } => {
            reject(contract_messages::cheri_parallel_loop(function))
        }
    }
}

pub(super) fn validate_rvalue(value: &Rvalue, function: &str) -> Result<(), CompilationFailure> {
    match value {
        Rvalue::Use(operand) | Rvalue::UnaryOperator(_, operand) => {
            validate_operand(operand, function)
        }
        Rvalue::BinaryOperator(_, left, right) => {
            validate_operand(left, function)?;

            validate_operand(right, function)
        }
        Rvalue::PointerOffset { .. } => reject(
            contract_messages::pointer_arithmetic_in_function_function(function),
        ),
        Rvalue::Cast { expression, to } => {
            validate_operand(expression, function)?;
            if matches!(to, Type::Reference { .. } | Type::RawPointer { .. }) {
                return reject(
                    contract_messages::cast_to_pointer_or_safe_reference_in_function_function(
                        function,
                    ),
                );
            }

            validate_type(to, &contract_messages::cast_in_function_function(function))
        }
        Rvalue::AddressOf { expression, .. } => validate_place(expression, function),
        Rvalue::Tuple(_) => reject(contract_messages::tuple_construction_in_function_function(
            function,
        )),
        Rvalue::Array(operands)
        | Rvalue::Call {
            arguments: operands,
            ..
        } => validate_operands(operands, function),
        Rvalue::SizeOf(value_type) => validate_type(
            value_type,
            &contract_messages::size_expression_in_function(function),
        ),
        Rvalue::TryCall { .. } | Rvalue::CatchCall { .. } => reject(
            contract_messages::exception_operation_in_function_function(function),
        ),
        Rvalue::StructInstantiate { fields, .. } => {
            for (_, operand) in fields {
                validate_operand(operand, function)?;
            }

            Ok(())
        }
    }
}

pub(super) fn validate_terminator(
    terminator: &MirTerminator,
    function: &str,
) -> Result<(), CompilationFailure> {
    match terminator {
        MirTerminator::Return
        | MirTerminator::Goto(_)
        | MirTerminator::Abort
        | MirTerminator::Unreachable => Ok(()),
        MirTerminator::SwitchInteger { discriminant, .. } => {
            validate_operand(discriminant, function)
        }
        MirTerminator::Exit { code } => validate_operand(code, function),
        MirTerminator::Call {
            arguments,
            destination,
            ..
        } => {
            validate_operands(arguments, function)?;

            validate_place(destination, function)
        }
        MirTerminator::TailCall { arguments, .. } => validate_operands(arguments, function),
        MirTerminator::Throw { .. }
        | MirTerminator::TryCall { .. }
        | MirTerminator::CatchCall { .. } => {
            reject(contract_messages::exception_control_flow_in_function_function(function))
        }
    }
}

pub(super) fn validate_operands(
    operands: &[Operand],
    function: &str,
) -> Result<(), CompilationFailure> {
    for operand in operands {
        validate_operand(operand, function)?;
    }

    Ok(())
}

pub(super) fn validate_operand(
    operand: &Operand,
    function: &str,
) -> Result<(), CompilationFailure> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => validate_place(place, function),
        Operand::ConstantString(_) => reject(contract_messages::string_value_in_function_function(
            function,
        )),
        Operand::TypeLiteral(value_type) => validate_type(
            value_type,
            &contract_messages::type_literal_in_function_function(function),
        ),
        Operand::ConstantInteger(_)
        | Operand::ConstantFloat(_)
        | Operand::ConstantBoolean(_)
        | Operand::ConstantCharacter(_) => Ok(()),
    }
}

pub(super) fn validate_place(place: &Place, function: &str) -> Result<(), CompilationFailure> {
    match place {
        Place::Local(_) => Ok(()),
        Place::Global { name, .. } => reject(contract_messages::cheri_global_place(name, function)),
        Place::Field {
            base, value_type, ..
        }
        | Place::Index {
            base, value_type, ..
        }
        | Place::Dereference { base, value_type } => {
            validate_type(
                value_type,
                &contract_messages::projected_place_in_function_function(function),
            )?;

            validate_place(base, function)
        }
    }
}
