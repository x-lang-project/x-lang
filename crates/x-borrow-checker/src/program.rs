use std::collections::HashMap;
use std::sync::Arc;

use super::{BorrowCheckError, BorrowChecker, return_summaries};
use x_ast::{
    DeclarationKey, ExpressionKind, Pattern, Program, SourceError, SourceSpan, Statement, Type,
};

impl BorrowChecker {
    pub fn check(&mut self, program: &Program) -> Result<(), BorrowCheckError> {
        self.check_with_declaration_sources(program, &HashMap::new())
    }

    pub fn check_with_declaration_sources(
        &mut self,
        program: &Program,
        sources: &HashMap<DeclarationKey, SourceSpan>,
    ) -> Result<(), BorrowCheckError> {
        self.prepare_program(program);
        self.declare_program_bindings(program)?;

        for statement in &program.statements {
            self.check_statement(statement).map_err(|error| {
                error.at_source(
                    statement
                        .declaration_key()
                        .and_then(|key| sources.get(&key).copied()),
                )
            })?;
        }

        Ok(())
    }

    fn prepare_program(&mut self, program: &Program) {
        let context = Arc::make_mut(&mut self.context);
        context.return_summaries = Arc::new(return_summaries::build_summaries(program));
        context.method_receivers = Arc::new(method_receivers(program));
        context.struct_layouts = Arc::new(
            program
                .statements
                .iter()
                .filter_map(|statement| {
                    if let Statement::StructDeclaration(definition) = statement {
                        Some((definition.name.clone(), definition.clone()))
                    } else {
                        None
                    }
                })
                .collect(),
        );
    }

    fn declare_program_bindings(&mut self, program: &Program) -> Result<(), BorrowCheckError> {
        for current_statement in &program.statements {
            match current_statement {
                Statement::Function { name, .. }
                | Statement::ExternFunctionDeclaration { name, .. } => {
                    self.declare_variable(name, Type::Unknown, true)?;
                }
                Statement::ImplDeclaration(definition) => {
                    let owner = nominal_owner_name(&definition.type_name);

                    for statement in &definition.methods {
                        if let Statement::Function { name, .. } = statement {
                            self.declare_variable(
                                &format!("{owner}::{name}"),
                                Type::Unknown,
                                true,
                            )?;
                        }
                    }
                }
                Statement::Constant {
                    name,
                    type_annotation,
                    value,
                } => {
                    self.declare_variable(
                        name,
                        type_annotation
                            .clone()
                            .unwrap_or_else(|| value.value_type.clone()),
                        true,
                    )?;
                }
                Statement::VariableDeclaration {
                    pattern: Pattern::Identifier(name),
                    type_annotation,
                    value,
                } => {
                    self.declare_variable(
                        name,
                        type_annotation
                            .clone()
                            .unwrap_or_else(|| value.value_type.clone()),
                        !matches!(value.kind, ExpressionKind::TypeLiteral(_)),
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

pub(super) fn nominal_owner_name(value_type: &Type) -> String {
    match value_type {
        Type::Custom(name) | Type::GenericInstance { name, .. } => name.clone(),
        _ => value_type.to_string(),
    }
}

fn method_receivers(program: &Program) -> HashMap<(String, String), (String, Type)> {
    let mut methods = HashMap::new();

    for statement in &program.statements {
        match statement {
            Statement::ImplDeclaration(definition) => {
                let owner = nominal_owner_name(&definition.type_name);

                for statement in &definition.methods {
                    if let Statement::Function {
                        name, parameters, ..
                    } = statement
                        && let Some(receiver) = parameters
                            .first()
                            .filter(|parameter| parameter.is_receiver())
                    {
                        methods.insert(
                            (owner.clone(), name.clone()),
                            (format!("{owner}::{name}"), receiver.value_type.clone()),
                        );
                    }
                }
            }
            Statement::Function {
                name, parameters, ..
            } => {
                if let Some(owner) = x_ast::intrinsics::function_owner_name(name)
                    && let Some(receiver) = parameters
                        .first()
                        .filter(|parameter| parameter.is_receiver())
                {
                    methods.insert(
                        (
                            owner.into(),
                            x_ast::intrinsics::function_leaf_name(name).into(),
                        ),
                        (name.clone(), receiver.value_type.clone()),
                    );
                }
            }
            _ => {}
        }
    }

    methods
}
