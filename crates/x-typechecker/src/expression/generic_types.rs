mod inference;
pub(super) use inference::infer_declared_type_substitutions;

use super::*;
use x_ast::GenericParameter;
use x_ast::TypeVisitorMut;

pub(super) fn explicit_type_argument_substitutions(
    type_arguments: &[Type],
    generic_parameters: &[GenericParameter],
) -> HashMap<String, Type> {
    let mut substitutions = HashMap::new();

    for (parameter, argument) in generic_parameters
        .iter()
        .filter(|parameter| !matches!(parameter, GenericParameter::Lifetime { .. }))
        .zip(type_arguments.iter())
    {
        if let GenericParameter::Type { name, .. } = parameter {
            substitutions.insert(name.clone(), argument.clone());
        }
    }

    substitutions
}

pub(super) fn infer_type_substitutions(
    checker: &TypeChecker,
    pattern: &Type,
    actual: &Type,
    substitutions: &mut HashMap<String, Type>,
) {
    infer_declared_type_substitutions(checker, pattern, actual, &[], substitutions);
}

pub(super) fn substitute_type_in_parameter(
    parameter: &mut Parameter,
    checker: &mut TypeChecker,
    substitutions: &HashMap<String, Type>,
) {
    parameter.value_type = checker.substitute_type(&parameter.value_type, substitutions);
}

struct GenericTypeSubstituter<'a> {
    checker: &'a mut TypeChecker,
    substitutions: &'a HashMap<String, Type>,
}

impl TypeVisitorMut for GenericTypeSubstituter<'_> {
    fn visit_type(&mut self, value_type: &mut Type) {
        *value_type = self.checker.substitute_type(value_type, self.substitutions);
    }

    fn visit_statement(&mut self, statement: &mut Statement) {
        // Nested function binders shadow the enclosing substitution map.
        if !matches!(statement, Statement::Function { .. }) {
            x_ast::walk_statement_types_mut(self, statement);
        }
    }
}

pub(super) fn substitute_types_in_statement(
    statement: &mut Statement,
    checker: &mut TypeChecker,
    substitutions: &HashMap<String, Type>,
) {
    TypeVisitorMut::visit_statement(
        &mut GenericTypeSubstituter {
            checker,
            substitutions,
        },
        statement,
    );
}

pub(crate) fn substitute_types_in_expression(
    expression: &mut Expression,
    checker: &mut TypeChecker,
    substitutions: &HashMap<String, Type>,
) {
    TypeVisitorMut::visit_expression(
        &mut GenericTypeSubstituter {
            checker,
            substitutions,
        },
        expression,
    );
}

pub(super) fn is_supported_explicit_function_argument(argument: &Type) -> bool {
    !argument.any_nested_type(|value_type| {
        matches!(
            value_type,
            Type::Reference { .. }
                | Type::RawPointer { .. }
                | Type::Array(_)
                | Type::FixedSizedArray(_, _)
                | Type::Tuple(_)
                | Type::Function { .. }
                | Type::Simd { .. }
        )
    })
}
