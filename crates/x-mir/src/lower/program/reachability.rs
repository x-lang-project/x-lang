use std::collections::{HashMap, HashSet};

use super::impl_method_owner_name;
use crate::source_names::{
    method_call_function_name as method_call_source_name, normalise_function_name,
};
use x_ast::intrinsics::function_owner_name;
use x_ast::{ENTRY_POINT_NAME, Expression, ExpressionKind, ExpressionVisitor, Statement};
use x_hir::{HirFunction, HirItem, HirProgram};

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

struct CallCollector<'a>(&'a mut Vec<String>);

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
