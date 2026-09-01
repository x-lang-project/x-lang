mod frame;

mod invariants;

use std::collections::HashMap;

use inkwell::basic_block;

use crate::codegen::CodeGeneration;
use x_mir::MirBody;

pub use frame::LocalFrame;
