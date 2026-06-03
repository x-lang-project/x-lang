use super::*;

pub fn codegen_missing_terminator_display(block: impl Display) -> String {
    Error::new(format!("MIR block {block} has no terminator")).render_detail()
}

pub fn codegen_missing_entry_block_display() -> String {
    Error::new("MIR function has no entry block 0").render_detail()
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

pub fn codegen_missing_mir_block_display(block: impl Display) -> String {
    Error::new(format!("referenced MIR block {block} is missing")).render_detail()
}

pub fn codegen_missing_llvm_block_display(block: impl Display) -> String {
    Error::new(format!("MIR block {block} has no LLVM mapping")).render_detail()
}

pub fn codegen_llvm_verification_failure_display(detail: impl Display) -> String {
    Error::new(format!("Generated LLVM IR failed verification: {detail}")).render_detail()
}
