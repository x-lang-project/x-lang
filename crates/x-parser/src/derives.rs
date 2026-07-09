mod methods;
use methods::*;
use x_ast::{
    Expression, ExpressionKind, ImplDefinition, Parameter, Statement, StringLiteral, StringPart,
    StructDefinition, StructInitialisation, Type,
};

#[derive(Clone, Copy)]
enum BuiltinDerive {
    Debug,
    Clone,
    Default,
    Eq,
    PartialEq,
    Hash,
}

fn derived_self_type(definition: &StructDefinition) -> Type {
    if let Some(parameters) = &definition.generic_parameters
        && !parameters.is_empty()
    {
        return Type::GenericInstance {
            name: definition.name.clone(),
            type_arguments: parameters
                .iter()
                .filter_map(x_ast::GenericParameter::non_lifetime_name)
                .map(|name| Type::TypeParameter(name.clone()))
                .collect(),
        };
    }

    Type::Custom(definition.name.clone())
}

struct DerivedMethodSignature<'a> {
    derive: BuiltinDerive,
    method_name: &'a str,
    parameters: Vec<Parameter>,
    return_type: Type,
}
