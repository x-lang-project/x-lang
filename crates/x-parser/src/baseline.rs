use x_ast::{Layout, Program, Statement, Type, TypeVisitorMut, walk_statement_types_mut};

pub(super) fn strip_semantic_features_in_program(program: &mut Program) {
    Baseline.visit_statements(&mut program.statements);
}

struct Baseline;
impl TypeVisitorMut for Baseline {
    fn visit_type(&mut self, value_type: &mut Type) {
        strip_semantic_features_in_type(value_type);
    }

    fn visit_statement(&mut self, statement: &mut Statement) {
        if let Statement::Become { call } = statement {
            *statement = Statement::Return {
                value: Some(call.clone()),
            };
        }

        match statement {
            Statement::Function {
                parameters,
                is_pure,
                is_memoised,
                is_static,
                is_multi,
                is_throws,
                is_exported,
                ..
            } => {
                *is_pure = false;
                *is_memoised = false;
                *is_static = false;
                *is_multi = false;
                *is_throws = false;
                *is_exported = false;

                for parameter in parameters {
                    parameter.is_static = false;
                }
            }
            Statement::ForRangeLoop { is_parallel, .. }
            | Statement::ForEachLoop { is_parallel, .. } => *is_parallel = false,
            Statement::StructDeclaration(definition) => {
                definition.layout = Layout::AoS;
                definition.invariant = None;
                for (_, value_type) in &mut definition.fields {
                    self.visit_type(value_type);
                }

                return;
            }
            Statement::TraitDeclaration(definition) => {
                for method in definition.methods.values_mut().flatten() {
                    self.visit_statement(method);
                }

                return;
            }
            Statement::ImplDeclaration(definition) => {
                self.visit_type(&mut definition.type_name);
                self.visit_statements(&mut definition.methods);
                return;
            }
            _ => {}
        }

        walk_statement_types_mut(self, statement);
    }
}

fn strip_semantic_features_in_type(value_type: &mut Type) {
    match value_type {
        Type::Array(inner)
        | Type::FixedSizedArray(inner, _)
        | Type::RawPointer { inner, .. }
        | Type::Simd { element: inner, .. } => strip_semantic_features_in_type(inner),
        Type::GenericInstance {
            type_arguments: items,
            ..
        }
        | Type::Tuple(items) => {
            for item in items {
                strip_semantic_features_in_type(item);
            }
        }
        Type::Reference {
            is_unique, inner, ..
        } => {
            *is_unique = false;
            strip_semantic_features_in_type(inner);
        }
        Type::Function {
            parameters,
            return_value,
        } => {
            for parameter in parameters {
                strip_semantic_features_in_type(parameter);
            }

            strip_semantic_features_in_type(return_value);
        }
        Type::I32
        | Type::F32
        | Type::String
        | Type::Character
        | Type::Boolean
        | Type::Void
        | Type::Never
        | Type::Custom(_)
        | Type::Unknown
        | Type::Infer(_)
        | Type::TypeParameter(_)
        | Type::I8
        | Type::U8
        | Type::I16
        | Type::U16
        | Type::U32
        | Type::I64
        | Type::U64
        | Type::ISize
        | Type::USize
        | Type::F64 => {}
    }
}
