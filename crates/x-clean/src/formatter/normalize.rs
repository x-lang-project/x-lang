//! Normalise trailing separators without changing opaque macro tokens.
use quote::ToTokens;
use syn::punctuated::{Pair, Punctuated};
use syn::visit_mut::{self, VisitMut};

fn trailing<T, P>(values: &mut Punctuated<T, P>) {
    if values.trailing_punct()
        && let Some(Pair::Punctuated(value, _)) = values.pop()
    {
        values.push_value(value);
    }
}

struct Normalize;
macro_rules! normalize {
    ($method:ident, $kind:ty, $field:ident) => {
        fn $method(&mut self, node: &mut $kind) {
            trailing(&mut node.$field);
            visit_mut::$method(self, node);
        }
    };
}
impl VisitMut for Normalize {
    normalize!(visit_fields_named_mut, syn::FieldsNamed, named);
    normalize!(visit_fields_unnamed_mut, syn::FieldsUnnamed, unnamed);
    normalize!(visit_item_enum_mut, syn::ItemEnum, variants);
    normalize!(visit_signature_mut, syn::Signature, inputs);
    normalize!(visit_generics_mut, syn::Generics, params);
    normalize!(visit_where_clause_mut, syn::WhereClause, predicates);
    normalize!(visit_expr_call_mut, syn::ExprCall, args);
    normalize!(visit_expr_method_call_mut, syn::ExprMethodCall, args);
    normalize!(visit_expr_struct_mut, syn::ExprStruct, fields);
    normalize!(visit_expr_array_mut, syn::ExprArray, elems);

    normalize!(visit_pat_tuple_struct_mut, syn::PatTupleStruct, elems);
    normalize!(visit_pat_struct_mut, syn::PatStruct, fields);
    normalize!(
        visit_angle_bracketed_generic_arguments_mut,
        syn::AngleBracketedGenericArguments,
        args
    );
    normalize!(
        visit_parenthesized_generic_arguments_mut,
        syn::ParenthesizedGenericArguments,
        inputs
    );
    normalize!(visit_use_group_mut, syn::UseGroup, items);
    fn visit_expr_match_mut(&mut self, node: &mut syn::ExprMatch) {
        for arm in &mut node.arms {
            arm.comma = None;
        }

        visit_mut::visit_expr_match_mut(self, node);
    }

    fn visit_expr_tuple_mut(&mut self, node: &mut syn::ExprTuple) {
        if node.elems.len() != 1 {
            trailing(&mut node.elems);
        }

        visit_mut::visit_expr_tuple_mut(self, node);
    }

    fn visit_type_tuple_mut(&mut self, node: &mut syn::TypeTuple) {
        if node.elems.len() != 1 {
            trailing(&mut node.elems);
        }

        visit_mut::visit_type_tuple_mut(self, node);
    }

    fn visit_pat_tuple_mut(&mut self, node: &mut syn::PatTuple) {
        if node.elems.len() != 1 {
            trailing(&mut node.elems);
        }

        visit_mut::visit_pat_tuple_mut(self, node);
    }

    fn visit_block_mut(&mut self, node: &mut syn::Block) {
        if let Some(syn::Stmt::Expr(
            syn::Expr::Return(_) | syn::Expr::Break(_) | syn::Expr::Continue(_),
            semi,
        )) = node.stmts.last_mut()
        {
            *semi = None;
        }

        visit_mut::visit_block_mut(self, node);
    }
}

pub(super) fn fingerprint(mut file: syn::File) -> String {
    Normalize.visit_file_mut(&mut file);

    file.to_token_stream().to_string()
}
