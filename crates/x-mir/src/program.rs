use std::collections::HashMap;

use crate::{
    BasicBlock, BlockIdentifier, CallContract, LocalDeclaration, LocalIdentifier, Operand,
    ResolvedCall,
};
use x_ast::{StructDefinition, TraitDefinition, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct MirBody {
    pub call_targets: HashMap<String, ResolvedCall>,
    pub locals: HashMap<LocalIdentifier, LocalDeclaration>,
    pub basic_blocks: HashMap<BlockIdentifier, BasicBlock>,
    pub argument_count: usize,
    pub return_local: LocalIdentifier,
    pub is_throws: bool,
    pub is_memoised: bool,
    pub call_contract: Option<CallContract>,
}
