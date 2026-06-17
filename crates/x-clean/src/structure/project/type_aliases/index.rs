use std::collections::{HashMap, HashSet};

use super::super::module_path;

// Unqualified lookup inherits block scopes and stops at a module boundary.
struct Scope {
    module: String,
    lexical_parent: Option<usize>,
    unresolved_glob: bool,
    bindings: HashMap<String, TypeBinding>,
}

pub(super) struct Alias {
    pub(super) source: usize,
    pub(super) offset: usize,
    scope: usize,
    name: String,
    target: Option<Vec<String>>,
}

#[derive(Clone, Copy, PartialEq)]
enum TypeBinding {
    Alias(usize),
    Unresolved,
}

impl TypeBinding {
    fn alias(self) -> Option<usize> {
        match self {
            Self::Alias(index) => Some(index),
            Self::Unresolved => None,
        }
    }
}

#[derive(Default)]
pub(super) struct AliasIndex {
    scopes: Vec<Scope>,
    modules: HashMap<String, usize>,
    aliases: Vec<Alias>,
}

impl AliasIndex {
    pub(super) fn module_scope(&mut self, module: String) -> usize {
        if let Some(scope) = self.modules.get(&module) {
            return *scope;
        }

        let scope = self.scopes.len();
        self.modules.insert(module.clone(), scope);
        self.scopes.push(Scope {
            module,
            lexical_parent: None,
            unresolved_glob: false,
            bindings: HashMap::new(),
        });

        scope
    }

    pub(super) fn module(&self, scope: usize) -> &str {
        &self.scopes[scope].module
    }

    pub(super) fn child_module_scope(&mut self, parent: usize, name: &str) -> usize {
        let scope = &self.scopes[parent];
        let module = if scope.lexical_parent.is_some() {
            format!("{}::<block-{parent}>::{name}", scope.module)
        } else {
            format!("{}::{name}", scope.module)
        };

        self.module_scope(module)
    }

    pub(super) fn block_scope(&mut self, parent: usize) -> usize {
        let scope = self.scopes.len();
        self.scopes.push(Scope {
            module: self.scopes[parent].module.clone(),
            lexical_parent: Some(parent),
            unresolved_glob: false,
            bindings: HashMap::new(),
        });

        scope
    }

    pub(super) fn add(&mut self, alias: Alias) {
        let index = self.aliases.len();
        self.scopes[alias.scope]
            .bindings
            .entry(alias.name.clone())
            .and_modify(|owner| *owner = TypeBinding::Unresolved)
            .or_insert(TypeBinding::Alias(index));
        self.aliases.push(alias);
    }

    pub(super) fn unresolved(&mut self, scope: usize, name: String) {
        self.scopes[scope]
            .bindings
            .insert(name, TypeBinding::Unresolved);
    }

    pub(super) fn unresolved_glob(&mut self, scope: usize) {
        self.scopes[scope].unresolved_glob = true;
    }

    fn lexical_alias(&self, mut scope: usize, name: &str) -> Option<usize> {
        loop {
            if let Some(alias) = self.scopes[scope].bindings.get(name) {
                return alias.alias();
            }

            if self.scopes[scope].unresolved_glob {
                return None;
            }

            scope = self.scopes[scope].lexical_parent?;
        }
    }

    fn qualified_alias(&self, scope: usize, path: &[String]) -> Option<usize> {
        let module = self.module(scope);
        let mut target: Vec<_> = if path[0] == "crate" {
            vec![module_path::root(module).to_owned()]
        } else {
            module.split("::").map(str::to_owned).collect()
        };

        let mut position = 0;

        while let Some(part) = path.get(position) {
            match part.as_str() {
                "crate" | "self" if position == 0 => {}
                "super" if target.len() > 1 => {
                    target.pop();
                }
                "super" => return None,
                _ => break,
            }
            position += 1;
        }

        let (name, modules) = path[position..].split_last()?;
        target.extend(modules.iter().cloned());
        let owner = self.modules.get(&target.join("::"))?;

        self.scopes[*owner]
            .bindings
            .get(name)
            .and_then(|binding| binding.alias())
    }

    fn target(&self, alias: &Alias) -> Option<usize> {
        let path = alias.target.as_ref()?;

        if path.len() == 1 && !matches!(path[0].as_str(), "crate" | "self" | "super") {
            self.lexical_alias(alias.scope, &path[0])
        } else {
            self.qualified_alias(alias.scope, path)
        }
    }

    pub(super) fn chains(&self) -> impl Iterator<Item = (&Alias, usize)> {
        self.aliases
            .iter()
            .enumerate()
            .filter(|(index, alias)| {
                self.scopes[alias.scope].bindings.get(&alias.name)
                    == Some(&TypeBinding::Alias(*index))
            })
            .map(|(_, alias)| {
                let mut current = alias;
                let mut seen = HashSet::new();

                while let Some(target) = self.target(current) {
                    if !seen.insert(target) {
                        break;
                    }

                    current = &self.aliases[target];
                }

                (alias, seen.len())
            })
    }
}

impl Alias {
    pub(super) fn new(source: usize, scope: usize, item: &syn::ItemType, offset: usize) -> Self {
        Self {
            source,
            scope,
            name: item.ident.to_string(),
            target: reference_path(item),
            offset,
        }
    }
}

fn reference_path(item: &syn::ItemType) -> Option<Vec<String>> {
    let syn::Type::Path(value_type) = &*item.ty else {
        return None;
    };

    if value_type.qself.is_some()
        || value_type.path.leading_colon.is_some()
        || value_type
            .path
            .segments
            .iter()
            .any(|part| !matches!(part.arguments, syn::PathArguments::None))
    {
        return None;
    }

    let first = &value_type.path.segments.first()?.ident;
    let refers_to_parameter = item
        .generics
        .params
        .iter()
        .any(|parameter| match parameter {
            syn::GenericParam::Type(parameter) => parameter.ident == *first,
            syn::GenericParam::Const(parameter) => parameter.ident == *first,
            syn::GenericParam::Lifetime(_) => false,
        });

    if refers_to_parameter {
        return None;
    }

    Some(
        value_type
            .path
            .segments
            .iter()
            .map(|part| part.ident.to_string())
            .collect(),
    )
}
