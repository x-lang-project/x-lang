use x_ast::{StructDefinition, Type};
use x_mir::{FunctionMeta, GlobalItem, MirBody, MirProgram, Operand};

#[derive(Clone, Copy)]
pub struct FunctionDeclaration<'a> {
    pub name: &'a str,
    pub parameter_types: &'a [Type],
    pub return_type: &'a Type,
    pub body: Option<&'a MirBody>,
    pub meta: &'a FunctionMeta,
}

pub(super) struct StaticItem<'a> {
    pub name: &'a str,
    pub value_type: &'a Type,
    pub initializer: Option<&'a Operand>,
    pub is_constant: bool,
}

pub(super) trait MirProgramItems {
    fn structs(&self) -> impl Iterator<Item = &StructDefinition>;
    fn statics(&self) -> impl Iterator<Item = StaticItem<'_>>;
    fn functions(&self) -> impl Iterator<Item = FunctionDeclaration<'_>>;
    fn function_bodies(&self) -> impl Iterator<Item = (&str, &MirBody)>;
}

impl MirProgramItems for MirProgram {
    fn structs(&self) -> impl Iterator<Item = &StructDefinition> {
        self.items.iter().filter_map(|item| match item {
            GlobalItem::Struct(definition) => Some(definition),
            _ => None,
        })
    }

    fn statics(&self) -> impl Iterator<Item = StaticItem<'_>> {
        self.items.iter().filter_map(|item| match item {
            GlobalItem::Static {
                name,
                value_type,
                initializer,
                is_constant,
            } => Some(StaticItem {
                name,
                value_type,
                initializer: initializer.as_ref(),
                is_constant: *is_constant,
            }),
            _ => None,
        })
    }

    fn functions(&self) -> impl Iterator<Item = FunctionDeclaration<'_>> {
        self.items.iter().filter_map(|item| match item {
            GlobalItem::Function {
                name,
                parameter_types,
                return_type,
                body,
                meta,
            } => Some(FunctionDeclaration {
                name,
                parameter_types,
                return_type,
                body: body.as_ref(),
                meta,
            }),
            _ => None,
        })
    }

    fn function_bodies(&self) -> impl Iterator<Item = (&str, &MirBody)> {
        self.functions()
            .filter_map(|function| function.body.map(|body| (function.name, body)))
    }
}
