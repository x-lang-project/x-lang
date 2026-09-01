use std::borrow::Cow;
use std::collections::HashMap;

use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;

use x_ast::Type;
use x_mir::{LocalIdentifier, MirBody};

struct LocalStorage<'ctx> {
    pointer: PointerValue<'ctx>,
    value_type: Type,
}

#[derive(Default)]
pub struct LocalFrame<'ctx> {
    locals: HashMap<LocalIdentifier, LocalStorage<'ctx>>,
}

impl<'ctx> LocalFrame<'ctx> {
    pub fn pointer(&self, identifier: LocalIdentifier) -> Option<PointerValue<'ctx>> {
        self.locals.get(&identifier).map(|local| local.pointer)
    }

    pub fn local_type(&self, identifier: LocalIdentifier) -> Option<&Type> {
        self.locals.get(&identifier).map(|local| &local.value_type)
    }
}
