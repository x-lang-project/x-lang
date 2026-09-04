mod arguments;

mod call_dispatch;

mod frame;

mod invariants;
mod llvm;

mod parallel;
mod pointers;
mod printing;
mod references;
mod rvalues;
mod slices;
mod terminator;
mod typing;

use std::collections::HashMap;

use inkwell::basic_block;

use crate::codegen::CodeGeneration;
use x_mir::MirBody;

pub use frame::LocalFrame;

pub struct FunctionCompiler<'a, 'ctx> {
    pub codegen: &'a CodeGeneration<'ctx>,
    pub function_name: &'a str,
    pub body: &'a MirBody,
    pub frame: LocalFrame<'ctx>,
    pub llvm_blocks: HashMap<usize, basic_block::BasicBlock<'ctx>>,
}
