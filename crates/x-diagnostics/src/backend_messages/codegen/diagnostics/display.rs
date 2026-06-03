use super::*;

pub fn codegen_llvm_verification_failure_display(detail: impl Display) -> String {
    Error::new(format!("Generated LLVM IR failed verification: {detail}")).render_detail()
}
