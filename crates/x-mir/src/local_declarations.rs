use std::collections::HashMap;
use std::mem;

use crate::{LocalDeclaration, LocalIdentifier};
use x_ast::Type;

#[derive(Default)]
pub(crate) struct LocalDeclarations {
    next_identifier: usize,
    declarations: HashMap<LocalIdentifier, LocalDeclaration>,
}

impl LocalDeclarations {
    pub(crate) fn declarations(&self) -> &HashMap<LocalIdentifier, LocalDeclaration> {
        &self.declarations
    }
}
