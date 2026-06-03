use super::*;

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

pub fn codegen_llvm_verification_failure_display(detail: impl Display) -> String {
    Error::new(format!("Generated LLVM IR failed verification: {detail}")).render_detail()
}
