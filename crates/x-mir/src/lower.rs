use std::collections::{HashMap, HashSet};

use crate::errors::MirLowerError;
use crate::local_declarations::LocalDeclarations;
use crate::{BasicBlock, BlockIdentifier, LocalIdentifier, MirStatement, MirTerminator};
use x_ast::Type;

pub struct MirLowerer {
    pub(crate) next_block: usize,
    pub(crate) locals: LocalDeclarations,
    pub(crate) variable_mappings: HashMap<String, LocalIdentifier>,
    pub(crate) global_mappings: HashMap<String, Type>,
    pub(crate) source_functions: HashSet<String>,
    pub(crate) basic_blocks: HashMap<BlockIdentifier, BasicBlock>,
    pub(crate) current_block: Option<BlockIdentifier>,
    loop_targets: Vec<LoopTargets>,
    expression_buffer: Vec<MirStatement>,
}

#[derive(Clone, Copy)]
struct LoopTargets {
    break_target: BlockIdentifier,
    continue_target: BlockIdentifier,
}

impl MirLowerer {
    fn make_block(&mut self) -> BlockIdentifier {
        let identifier = BlockIdentifier(self.next_block);
        self.next_block += 1;
        self.basic_blocks.insert(
            identifier,
            BasicBlock {
                statements: Vec::new(),
                terminator: None,
            },
        );

        identifier
    }
}
