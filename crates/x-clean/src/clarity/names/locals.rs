use super::*;

fn uses(text: &str, name: &str) -> bool {
    text.split(|character: char| !character.is_alphanumeric() && character != '_')
        .any(|word| word == name)
}

fn local_name(text: &syn::Stmt) -> Option<&syn::Ident> {
    if let syn::Stmt::Local(left_value) = text {
        match &left_value.pat {
            syn::Pat::Ident(pointer) => Some(&pointer.ident),
            syn::Pat::Type(type_value) => {
                if let syn::Pat::Ident(pointer) = &*type_value.pat {
                    Some(&pointer.ident)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

fn distance(first_value: &str, second_value: &str) -> usize {
    let mut previous: Vec<_> = (0..=second_value.chars().count()).collect();

    for (index, character) in first_value.chars().enumerate() {
        let mut next = vec![index + 1];

        for (inner_index, definition) in second_value.chars().enumerate() {
            next.push(
                (previous[inner_index + 1] + 1)
                    .min(next[inner_index] + 1)
                    .min(previous[inner_index] + usize::from(character != definition)),
            );
        }
        previous = next;
    }

    *previous.last().unwrap_or(&0)
}

impl Names<'_> {
    pub(super) fn check_block(&mut self, second_value: &syn::Block) {
        let locals: Vec<_> = second_value
            .stmts
            .iter()
            .enumerate()
            .filter_map(|(index, text)| local_name(text).map(|count| (index, count)))
            .collect();

        for (at, count) in &locals {
            let name = count.to_string();

            for (_, other) in &locals {
                let o = other.to_string();

                if name < o && name.len() >= 4 && o.len() >= 4 && distance(&name, &o) == 1 {
                    self.flag(
                        "confusable-name-pair",
                        count.span(),
                        &format!("Nearby bindings differ by one character: {name} / {o}."),
                    );
                }
            }

            let base =
                name.trim_end_matches(|call_expression: char| call_expression.is_ascii_digit());

            if base != name
                && !base.is_empty()
                && locals.iter().any(|(_, o)| {
                    o != count
                        && o.to_string().trim_end_matches(|call_expression: char| {
                            call_expression.is_ascii_digit()
                        }) == base
                })
            {
                self.flag(
                    "opaque-numbered-name",
                    count.span(),
                    "Sibling bindings differ only by numeric suffix.",
                );
            }

            let first = second_value
                .stmts
                .iter()
                .enumerate()
                .skip(at + 1)
                .find(|(_, text)| uses(&tokens(*text), &name));

            if first.is_some_and(|(index, _)| index - at >= 8) {
                self.flag(
                    "binding-definition-use-distance",
                    count.span(),
                    "First syntactic use occurs at least eight statements after declaration.",
                );
            }

            for (index, text) in second_value.stmts.iter().enumerate().skip(at + 1) {
                if let syn::Stmt::Expr(syn::Expr::Block(inner), _) = text {
                    let shadow = inner
                        .block
                        .stmts
                        .iter()
                        .find_map(local_name)
                        .is_some_and(|value| *value == name);
                    let resumed = second_value
                        .stmts
                        .iter()
                        .skip(index + 1)
                        .any(|text| uses(&tokens(text), &name));

                    if shadow && resumed {
                        self.flag(
                            "binding-shadow-distance",
                            inner.span(),
                            "Nested block shadows a local that is used again after the block.",
                        );
                    }
                }
            }
        }

        let mut assignments: HashMap<String, HashSet<String>> = HashMap::new();

        for text in &second_value.stmts {
            if let syn::Stmt::Expr(syn::Expr::Assign(first_value), _) = text
                && let syn::Expr::Call(call_expression) = &*first_value.right
            {
                assignments
                    .entry(tokens(&first_value.left))
                    .or_default()
                    .insert(tokens(&call_expression.func));
            }
        }

        for (name, calls) in assignments {
            if calls.len() >= 3 {
                self.flag("mutable-role-drift",second_value.span(),&format!("{name} receives results from at least three differently named callees; review role drift."));
            }
        }

        visit::visit_block(self, second_value);
    }
}
