mod lowering;

use inkwell::IntPredicate;
use inkwell::values::BasicValueEnum;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
use lowering::{LlvmIntrinsicDefinition, ZeroInput};
use x_ast::Type;
use x_ast::intrinsics::{AlignOperation, NumberOperation};
use x_diagnostics::backend_messages;
use x_mir::Operand;

const BASE_PAGE_ALIGNMENT: u64 = 4 * 1024;

fn align_minmax_predicate(operation: AlignOperation) -> IntPredicate {
    match operation {
        AlignOperation::Maximum => IntPredicate::UGT,
        _ => IntPredicate::ULT,
    }
}

fn number_minmax_predicate(operation: NumberOperation) -> IntPredicate {
    match operation {
        NumberOperation::Minimum => IntPredicate::SLT,
        _ => IntPredicate::SGT,
    }
}
