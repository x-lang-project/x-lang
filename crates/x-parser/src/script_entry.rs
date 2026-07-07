use std::mem;

use x_ast::{ENTRY_POINT_NAME, Expression, ExpressionKind, Program, Statement, Type};

pub(super) fn normalise_script_entrypoint(program: &mut Program) {
    if program
        .statements
        .iter()
        .any(|statement| matches!(statement, Statement::Function { name, .. } if name == ENTRY_POINT_NAME))
    {
        return;
    }

    let mut main_body = Vec::new();
    let mut top_level = Vec::new();

    for statement in mem::take(&mut program.statements) {
        if is_script_statement(&statement) {
            main_body.push(statement);
        } else {
            top_level.push(statement);
        }
    }

    if main_body.is_empty() {
        program.statements = top_level;
        return;
    }

    main_body.push(Statement::Return {
        value: Some(Expression {
            span: None,
            value_type: Type::I32,
            kind: ExpressionKind::Integer(0),
        }),
    });

    top_level.push(Statement::Function {
        name: ENTRY_POINT_NAME.to_string(),
        generic_parameters: Vec::new(),
        parameters: Vec::new(),
        return_type: Type::I32,
        body: Some(Box::new(main_body)),
        is_pure: false,
        is_memoised: false,
        is_static: false,
        is_multi: false,
        is_throws: false,
        is_exported: false,
    });

    program.statements = top_level;
}

fn is_script_statement(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Expression { .. }
            | Statement::VariableDeclaration { .. }
            | Statement::Assert { .. }
            | Statement::Panic { .. }
            | Statement::Block { .. }
            | Statement::Become { .. }
            | Statement::ForRangeLoop { .. }
            | Statement::ForEachLoop { .. }
            | Statement::If { .. }
            | Statement::WhileLoop { .. }
            | Statement::Return { .. }
            | Statement::Break
            | Statement::Continue
    )
}
