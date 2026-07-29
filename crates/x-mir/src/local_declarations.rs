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
    pub(crate) fn allocate(&mut self, name: Option<String>, value_type: Type) -> LocalIdentifier {
        let identifier = LocalIdentifier(self.next_identifier);
        self.next_identifier += 1;
        self.declarations.insert(
            identifier,
            LocalDeclaration {
                identifier,
                name,
                value_type,
            },
        );

        identifier
    }

    pub(crate) fn get(&self, identifier: &LocalIdentifier) -> Option<&LocalDeclaration> {
        self.declarations.get(identifier)
    }

    pub(crate) fn declarations(&self) -> &HashMap<LocalIdentifier, LocalDeclaration> {
        &self.declarations
    }

    pub(crate) fn reset(&mut self) {
        self.next_identifier = 0;
        self.declarations.clear();
    }

    pub(crate) fn take_declarations(&mut self) -> HashMap<LocalIdentifier, LocalDeclaration> {
        mem::take(&mut self.declarations)
    }
}
