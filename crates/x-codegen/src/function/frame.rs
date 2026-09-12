use std::borrow::Cow;
use std::collections::HashMap;

use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;

use super::FunctionCompiler;
use crate::errors::CodeGenerationError;
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
    pub(crate) fn storage_name(body: &MirBody, identifier: LocalIdentifier) -> Cow<'_, str> {
        match body
            .locals
            .get(&identifier)
            .and_then(|local| local.name.as_deref())
        {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(format!("mir_temp_{}", identifier.0)),
        }
    }

    pub fn insert(
        &mut self,
        identifier: LocalIdentifier,
        value_type: Type,
        pointer: PointerValue<'ctx>,
    ) {
        self.locals.insert(
            identifier,
            LocalStorage {
                pointer,
                value_type,
            },
        );
    }

    pub fn pointer(&self, identifier: LocalIdentifier) -> Option<PointerValue<'ctx>> {
        self.locals.get(&identifier).map(|local| local.pointer)
    }

    pub fn local_type(&self, identifier: LocalIdentifier) -> Option<&Type> {
        self.locals.get(&identifier).map(|local| &local.value_type)
    }
}

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    pub(super) fn replace_local_storage(
        &mut self,
        identifier: LocalIdentifier,
        value_type: Type,
        llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<(), CodeGenerationError> {
        let name = LocalFrame::storage_name(self.body, identifier);
        let pointer = self.alloca(llvm_type, &name)?;

        self.frame.insert(identifier, value_type, pointer);

        Ok(())
    }
}
