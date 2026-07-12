use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::MAX_SUMMARY_DEPTH;
use super::ReturnSummary;
use super::ScalarSource;
use super::ScalarValue;
use super::SummaryFunctions;
use super::ValueSummary;
use super::expressions::{scalar_source, summarize_expression, summarize_local};
use crate::nominal_owner_name;
use x_ast::{Parameter, Program, Statement, Type};

struct FunctionDefinition<'a> {
    name: String,
    parameters: &'a [Parameter],
    body: &'a [Statement],
}

#[derive(Default)]
struct FunctionInventory<'a> {
    declared_names: HashSet<String>,
    definitions: Vec<FunctionDefinition<'a>>,
}

impl<'a> FunctionInventory<'a> {
    fn collect(program: &'a Program) -> Self {
        let mut inventory = Self::default();

        for statement in &program.statements {
            if let Statement::ImplDeclaration(definition) = statement {
                let owner = nominal_owner_name(&definition.type_name);

                for method in &definition.methods {
                    inventory.register(method, Some(&owner));
                }
            } else {
                inventory.register(statement, None);
            }
        }

        inventory
    }

    fn register(&mut self, statement: &'a Statement, owner: Option<&str>) {
        match statement {
            Statement::Function {
                name,
                parameters,
                body,
                ..
            } => {
                let name = owner.map_or_else(|| name.clone(), |owner| format!("{owner}::{name}"));
                self.declared_names.insert(name.clone());

                if let Some(body) = body {
                    self.definitions.push(FunctionDefinition {
                        name,
                        parameters,
                        body,
                    });
                }
            }
            Statement::ExternFunctionDeclaration { name, .. } if owner.is_none() => {
                self.declared_names.insert(name.clone());
            }
            _ => {}
        }
    }
}

struct SummaryContext<'a> {
    parameters: &'a [Parameter],
    known: &'a SummaryFunctions,
    remaining_work: Cell<usize>,
}

impl SummaryContext<'_> {
    fn summarize_body(
        &self,
        body: &[Statement],
        bindings: &HashMap<String, ValueSummary>,
        depth: usize,
    ) -> Option<ReturnSummary> {
        if depth >= MAX_SUMMARY_DEPTH {
            return None;
        }

        let cost = body.len().max(1);
        let work = self.remaining_work.get().checked_sub(cost)?;
        self.remaining_work.set(work);

        let mut executable = body
            .iter()
            .filter(|statement| !matches!(statement, Statement::Comment { .. }))
            .peekable();
        let mut bindings = bindings.clone();
        let mut depth = depth;

        while let Some(statement @ Statement::VariableDeclaration { .. }) = executable.peek() {
            summarize_local(statement, &mut bindings, &mut depth, self.known)?;
            executable.next();
        }

        let statement = executable.next()?;

        let remaining = executable.cloned().collect::<Vec<_>>();

        match statement {
            Statement::Throw { .. } | Statement::Panic { .. } => Some(ReturnSummary::Never),
            _ => None,
        }
    }
}
