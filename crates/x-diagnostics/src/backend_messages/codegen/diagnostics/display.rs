use super::*;

pub fn codegen_builder_error_display(error: impl Debug) -> String {
    Error::new(format!("LLVM builder failed: {error:?}")).render_detail()
}

pub fn codegen_type_not_found_display(type_name: impl Display) -> String {
    Error::new(format!(
        "LLVM structure layout '{type_name}' is not registered"
    ))
    .render_detail()
}

pub fn codegen_unsupported_type_display(value_type: impl Debug) -> String {
    Error::new(format!("unsupported backend type: {value_type:?}")).render_detail()
}

pub fn codegen_function_signature_missing_display(function_name: impl Display) -> String {
    Error::new(format!(
        "function '{function_name}' has no registered LLVM signature"
    ))
    .render_detail()
}

pub fn codegen_missing_terminator_display(block: impl Display) -> String {
    Error::new(format!("MIR block {block} has no terminator")).render_detail()
}

pub fn codegen_missing_entry_block_display() -> String {
    Error::new("MIR function has no entry block 0").render_detail()
}

pub fn codegen_missing_local_allocation_display(local: impl Display) -> String {
    Error::new(format!("local {local} has no stack allocation")).render_detail()
}

pub fn codegen_missing_hardware_argument_display(argument_index: impl Display) -> String {
    Error::new(format!("missing argument {argument_index}")).render_detail()
}

pub fn codegen_unsupported_operator_display(operator: impl Debug) -> String {
    Error::new(format!(
        "unsupported operator {operator:?} for these operand types"
    ))
    .render_detail()
}

pub fn codegen_unsupported_unary_operator_display(operator: impl Debug) -> String {
    Error::new(format!("unsupported unary operator: {operator:?}")).render_detail()
}

pub fn codegen_unsupported_cast_display(to: impl Debug) -> String {
    Error::new(format!("unsupported cast to {to:?}")).render_detail()
}

pub fn codegen_type_has_no_size_display(value_type: impl Debug) -> String {
    Error::new(format!("type has no known layout: {value_type:?}")).render_detail()
}

pub fn codegen_missing_callee_symbol_display(callee: impl Display) -> String {
    Error::new(format!("callee '{callee}' has no resolved LLVM symbol")).render_detail()
}

pub fn codegen_jit_execution_failure_display(entry_function: impl Display) -> String {
    Error::new(format!(
        "JIT entry '{entry_function}' is missing or has an incompatible signature"
    ))
    .render_detail()
}

pub fn codegen_llvm_internal_error_display(detail: impl Display) -> String {
    Error::new(format!("LLVM backend failed: {detail}")).render_detail()
}

pub fn codegen_jit_initialization_failure_display(detail: impl Display) -> String {
    Error::new(format!("LLVM JIT initialisation failed: {detail}")).render_detail()
}

pub fn codegen_missing_mir_block_display(block: impl Display) -> String {
    Error::new(format!("referenced MIR block {block} is missing")).render_detail()
}

pub fn codegen_missing_llvm_block_display(block: impl Display) -> String {
    Error::new(format!("MIR block {block} has no LLVM mapping")).render_detail()
}

pub fn codegen_llvm_verification_failure_display(detail: impl Display) -> String {
    Error::new(format!("Generated LLVM IR failed verification: {detail}")).render_detail()
}

pub fn codegen_invalid_mir_display(detail: impl Display) -> String {
    Error::new(format!(
        "MIR validation failed before code generation: {detail}"
    ))
    .render_detail()
}
