use std::collections::{HashMap, HashSet};

use super::impl_method_owner_name;
use crate::source_names::{
    method_call_function_name as method_call_source_name, normalise_function_name,
};
use x_ast::intrinsics::function_owner_name;
use x_ast::{ENTRY_POINT_NAME, Expression, ExpressionKind, ExpressionVisitor, Statement};
use x_hir::{HirFunction, HirItem, HirProgram};

pub(super) fn compute_reachable_functions(program: &HirProgram) -> HashSet<String> {
    let functions = function_bodies(program);

    if !functions.contains_key(ENTRY_POINT_NAME) {
        return HashSet::new();
    }

    let mut reachable = HashSet::new();
    let mut worklist = vec![ENTRY_POINT_NAME.to_string()];

    for item in &program.items {
        if let HirItem::Function(function) = item
            && function.is_exported
        {
            worklist.push(function.name.clone());
        }
    }

    while let Some(name) = worklist.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }

        let Some(Some(body)) = functions.get(&name) else {
            continue;
        };

        let mut calls = Vec::new();
        collect_calls_from_statements(body, &mut calls);

        for call in calls {
            if let Some(resolved) = resolve_reachable_function(&functions, &name, &call) {
                if !reachable.contains(&resolved) {
                    worklist.push(resolved);
                }
            } else {
                reachable.insert(call);
            }
        }
    }

    reachable
}

pub(super) fn is_reachable_source_function(
    reachable_functions: &HashSet<String>,
    name: &str,
) -> bool {
    reachable_functions.contains(name)
        || name.as_bytes().windows(2).enumerate().any(|(index, pair)| {
            if pair != b"::" {
                return false;
            }

            name.get(index + 2..)
                .is_some_and(|suffix| suffix.contains("::") && reachable_functions.contains(suffix))
        })
}

pub(super) fn resolve_reachable_function(
    functions: &HashMap<String, Option<&Vec<Statement>>>,
    current_function: &str,
    callee: &str,
) -> Option<String> {
    let normalized = normalise_function_name(callee);

    if functions.contains_key(&normalized) {
        return Some(normalized);
    }

    if !normalized.contains("::")
        && let Some(module_path) = function_owner_name(current_function)
    {
        let same_module = format!("{module_path}::{normalized}");

        if functions.contains_key(&same_module) {
            return Some(same_module);
        }
    }

    let suffix = format!("::{normalized}");
    let mut matches = functions.keys().filter(|name| name.ends_with(&suffix));
    let first = matches.next()?;

    matches.next().is_none().then(|| first.clone())
}

pub(super) fn collect_calls_from_statements(statements: &[Statement], output: &mut Vec<String>) {
    for statement in statements {
        CallCollector(output).visit_statement(statement);
    }
}

pub(super) fn collect_calls_from_expression(expression: &Expression, output: &mut Vec<String>) {
    match &expression.kind {
        ExpressionKind::FunctionCall { target, .. } => {
            output.push(target.symbol_name().into_owned())
        }
        ExpressionKind::MethodCall { object, target, .. } => {
            let name = match target.resolved_name() {
                Some(name) => name.to_owned(),
                None => method_call_source_name(&object.value_type, &target.source_path()),
            };

            output.push(name)
        }
        ExpressionKind::String(_) | ExpressionKind::AnonymousFunction { .. } => return,
        _ => {}
    }

    x_ast::walk_expression_children(&mut CallCollector(output), expression);
}

struct CallCollector<'a>(&'a mut Vec<String>);

impl ExpressionVisitor for CallCollector<'_> {
    fn visit_expression(&mut self, expression: &Expression) {
        collect_calls_from_expression(expression, self.0);
    }
}

fn function_bodies(program: &HirProgram) -> HashMap<String, Option<&Vec<Statement>>> {
    let mut functions = HashMap::new();

    for statement in &program.items {
        match statement {
            HirItem::Function(HirFunction { name, body, .. }) => {
                functions.insert(name.clone(), body.as_ref());
            }
            HirItem::Impl(impl_definition) => {
                let Some(owner_name) = impl_method_owner_name(&impl_definition.type_name) else {
                    continue;
                };

                for method in &impl_definition.methods {
                    functions.insert(
                        format!("{owner_name}::{}", method.name),
                        method.body.as_ref(),
                    );
                }
            }
            _ => {}
        }
    }

    functions
}
