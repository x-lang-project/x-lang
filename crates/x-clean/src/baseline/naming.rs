use std::collections;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use crate::{Finding, Source};

pub fn analyze(source: &Source, output: &mut Vec<Finding>) {
    Scanner { source, output }.visit_file(&source.ast);

    for (index, line) in source.text.lines().enumerate() {
        if line.trim_start().starts_with("//")
            && ["TODO", "FIXME", "HACK", "XXX"]
                .iter()
                .any(|tag| line.contains(tag))
        {
            source.emit(
                output,
                "todo-marker",
                source.line_offset(index + 1),
                "Deferred-work comment.",
            );
        }
    }
}

struct Scanner<'a> {
    source: &'a Source,
    output: &'a mut Vec<Finding>,
}
impl Scanner<'_> {
    fn function(&mut self, signature: &syn::Signature, block: &syn::Block) {
        let start = signature.span().start().line;
        let end = block.span().end().line;

        if end.saturating_sub(start) + 1 > 80 {
            self.source.emit(
                self.output,
                "long-function",
                self.source.offset(signature.span()),
                "Function exceeds 80 lines.",
            );
        }

        let count = signature
            .inputs
            .iter()
            .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
            .count();

        if count > 6 {
            self.source.emit(
                self.output,
                "excessive-parameters",
                self.source.offset(signature.span()),
                format!("Function has {count} parameters."),
            );
        }

        let name = signature.ident.to_string();
        let mut words = collections::BTreeSet::new();

        if name
            .split('_')
            .filter(|part| !part.is_empty())
            .any(|part| !words.insert(part))
        {
            self.source.emit(
                self.output,
                "repeated-name-word",
                self.source.offset(signature.ident.span()),
                "Function name repeats a word.",
            );
        }
    }
}

impl<'ast> Visit<'ast> for Scanner<'_> {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        let name = node.ident.to_string();
        let clean = name.trim_start_matches("r#").trim_matches('_');
        let at = self.source.offset(node.ident.span());

        if clean.chars().count() <= 2 && clean != "self" && !clean.is_empty() {
            self.source.emit(
                self.output,
                "short-name",
                at,
                format!("Short binding '{name}'."),
            );
        }

        if name.starts_with('_') {
            self.source.emit(
                self.output,
                "underscore-binding",
                at,
                format!("Underscore-prefixed binding '{name}'."),
            );
        }

        if clean.split('_').any(|part| {
            [
                "tmp", "temp", "foo", "bar", "baz", "data", "value", "thing", "stuff", "obj",
                "var", "res", "ret",
            ]
            .contains(&part)
        }) {
            self.source.emit(
                self.output,
                "vague-name",
                at,
                format!("Generic binding '{name}'."),
            );
        }

        if clean.chars().any(char::is_uppercase) {
            self.source.emit(
                self.output,
                "binding-naming-style",
                at,
                "Binding contains uppercase letters.",
            );
        }

        let mut words = collections::BTreeSet::new();

        if clean
            .split('_')
            .filter(|part| !part.is_empty())
            .any(|part| !words.insert(part))
        {
            self.source.emit(
                self.output,
                "repeated-name-word",
                at,
                "Binding repeats a word.",
            );
        }

        visit::visit_pat_ident(self, node);
    }

    fn visit_pat_wild(&mut self, node: &'ast syn::PatWild) {
        self.source.emit(
            self.output,
            "wildcard-binding",
            self.source.offset(node.span()),
            "Wildcard pattern discards or matches a value.",
        );
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.function(&node.sig, &node.block);
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.function(&node.sig, &node.block);
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.function(&node.sig, block);
        }

        visit::visit_trait_item_fn(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if ["unwrap", "expect"].contains(&node.method.to_string().as_str()) {
            self.source.emit(
                self.output,
                "review-debug-or-failure",
                self.source.offset(node.span()),
                "Explicit failure call.",
            );
        }

        visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.segments.last().is_some_and(|text| {
            ["dbg", "println", "eprintln", "print", "panic"]
                .contains(&text.ident.to_string().as_str())
        }) {
            self.source.emit(
                self.output,
                "review-debug-or-failure",
                self.source.offset(node.span()),
                "Debug output or panic macro.",
            );
        }

        visit::visit_macro(self, node);
    }
}
