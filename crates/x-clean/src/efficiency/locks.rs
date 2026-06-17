use std::collections::{BTreeMap, BTreeSet};

use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

use super::ast_checks::tokens;

struct Locks {
    pairs: Vec<(String, String, proc_macro2::Span)>,
}
impl<'ast> Visit<'ast> for Locks {
    fn visit_block(&mut self, node: &'ast syn::Block) {
        let mut acquired = Vec::new();

        for statement in &node.stmts {
            if let syn::Stmt::Local(local) = statement
                && let Some(initialise) = &local.init
            {
                let mut text = tokens(&initialise.expr);

                for method in [".lock(", ".read(", ".write("] {
                    if let Some(position) = text.find(method) {
                        text.truncate(position);
                        acquired.push((text.clone(), local.span()));
                        break;
                    }
                }
            }
        }

        for (index, (first_value, _)) in acquired.iter().enumerate() {
            for (second_value, span) in &acquired[index + 1..] {
                if first_value != second_value {
                    self.pairs
                        .push((first_value.clone(), second_value.clone(), *span));
                }
            }
        }

        visit::visit_block(self, node);
    }
}

pub(super) fn lock_order(sources: &[crate::Source], output: &mut Vec<crate::Finding>) {
    let mut pairs = BTreeMap::new();

    for (index, source) in sources.iter().enumerate() {
        let mut locks = Locks { pairs: Vec::new() };
        locks.visit_file(&source.ast);

        for (first_value, second_value, span) in locks.pairs {
            pairs
                .entry((first_value, second_value))
                .or_insert((index, span));
        }
    }

    let mut emitted = BTreeSet::new();

    for ((first_value, second_value), (index, span)) in &pairs {
        if pairs.contains_key(&(second_value.clone(), first_value.clone()))
            && emitted.insert((first_value.clone(), second_value.clone()))
        {
            sources[*index].emit(output,"nested-lock-order",sources[*index].offset(*span),format!("Lock-shaped receivers {first_value} and {second_value} are also acquired in reverse order; identity and lifetime are unresolved."));
        }
    }
}
