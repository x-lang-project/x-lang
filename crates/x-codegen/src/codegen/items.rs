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
