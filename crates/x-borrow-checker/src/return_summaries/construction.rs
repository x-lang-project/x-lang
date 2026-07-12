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
