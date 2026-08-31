mod records;

use std::{cell::RefCell, collections::HashMap};

use inkwell::targets;
use inkwell::{
    AddressSpace, OptimizationLevel, builder::Builder, context::Context,
    execution_engine::ExecutionEngine, module::Module,
};

use x_logging::info;
use x_mir::MirProgram;
