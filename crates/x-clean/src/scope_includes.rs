use syn::{spanned::Spanned, visit::Visit};

use crate::{Source, scope::Scope};

struct Includes<'a> {
    source: &'a Source,
    paths: Vec<(String, bool)>,
}

impl<'ast> Visit<'ast> for Includes<'_> {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.is_ident("include")
            && let Ok(path) = syn::parse2::<syn::LitStr>(node.tokens.clone())
        {
            let start = node.span().start();
            self.paths.push((
                path.value(),
                self.source.ast_scope_at(start.line, start.column + 1) == Scope::Test,
            ));
        }

        syn::visit::visit_macro(self, node);
    }
}

pub(crate) fn literal_includes(source: &Source) -> Vec<(String, bool)> {
    let mut visitor = Includes {
        source,
        paths: vec![],
    };
    visitor.visit_file(&source.ast);

    visitor.paths
}
