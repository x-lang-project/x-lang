use std::collections::HashMap;
use std::slice;

use functions::*;
use x_ast::{Expression, ExpressionKind, Program, Statement, Type};
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;

const MAX_SHAREABLE_TYPE_DEPTH: usize = 32;

#[derive(Clone)]
struct FunctionShape {
    pure: bool,
    throws: bool,
    parameters: Vec<Type>,
    result: Type,
}

pub fn validate_executable_contracts(program: &Program) -> Result<(), CompilationFailure> {
    let mut functions = HashMap::new();
    let mut structures = HashMap::new();

    collect_structures(&program.statements, &mut structures);
    collect_functions(&program.statements, &mut functions);

    validate_statements(&program.statements, &functions, &structures, false)
}

fn collect_structures(statements: &[Statement], structures: &mut HashMap<String, Vec<Type>>) {
    for statement in statements {
        if let Statement::StructDeclaration(definition) = statement {
            structures.insert(
                definition.name.clone(),
                definition
                    .fields
                    .iter()
                    .map(|(_, value_type)| value_type.clone())
                    .collect(),
            );
        }
    }
}

fn shareable_storage(
    value_type: &Type,
    structures: &HashMap<String, Vec<Type>>,
    depth: usize,
) -> bool {
    if depth > MAX_SHAREABLE_TYPE_DEPTH {
        return false;
    }

    if value_type.is_scalar() {
        return true;
    }

    match value_type {
        _ => false,
    }
}

fn shareable_capture(value_type: &Type, structures: &HashMap<String, Vec<Type>>) -> bool {
    value_type.is_scalar()
        || matches!(value_type, Type::Reference { is_mut: false, inner, .. }
        if shareable_storage(inner, structures, 0))
}

fn collect_functions(statements: &[Statement], functions: &mut HashMap<String, FunctionShape>) {
    for statement in statements {
        match statement {
            Statement::Function {
                name,
                parameters: current_parameters,
                return_type,
                is_pure,
                is_throws,
                ..
            } => {
                functions.insert(
                    name.clone(),
                    FunctionShape {
                        pure: *is_pure,
                        throws: *is_throws,
                        parameters: current_parameters
                            .iter()
                            .map(|pointer| pointer.value_type.clone())
                            .collect(),
                        result: return_type.clone(),
                    },
                );
            }
            Statement::ImplDeclaration(definition) => {
                collect_functions(&definition.methods, functions)
            }
            Statement::TraitDeclaration(definition) => {
                for method in definition.methods.values().flatten() {
                    collect_functions(slice::from_ref(method), functions);
                }
            }
            _ => {}
        }
    }
}

fn validate_statements(
    statements: &[Statement],
    functions: &HashMap<String, FunctionShape>,
    structures: &HashMap<String, Vec<Type>>,
    in_expression: bool,
) -> Result<(), CompilationFailure> {
    for (index, statement) in statements.iter().enumerate() {
        match statement {
            _ => {}
        }

        let mut expressions = ExpressionContracts {
            functions,
            structures,
            depth: 0,
            error: None,
        };
        x_ast::ExpressionVisitor::visit_statement(&mut expressions, statement);

        if let Some(error) = expressions.error {
            return Err(error);
        }
    }

    Ok(())
}

struct ExpressionContracts<'a> {
    functions: &'a HashMap<String, FunctionShape>,
    structures: &'a HashMap<String, Vec<Type>>,
    depth: usize,
    error: Option<CompilationFailure>,
}

impl x_ast::ExpressionVisitor for ExpressionContracts<'_> {
    fn visit_expression(&mut self, expression: &Expression) {
        if self.error.is_none() {
            self.depth += 1;
            x_ast::walk_expression_children(self, expression);
            self.depth -= 1;
        }
    }

    fn visit_statements(&mut self, statements: &[Statement]) {
        if self.depth != 0 && self.error.is_none() {
            self.error =
                validate_statements(statements, self.functions, self.structures, true).err();
        }
    }
}

mod functions;
