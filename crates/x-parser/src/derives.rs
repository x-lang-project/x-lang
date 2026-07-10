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

impl BuiltinDerive {
    fn name(self) -> &'static str {
        match self {
            Self::Debug => "Debug",
            Self::Clone => "Clone",
            Self::Default => "Default",
            Self::Eq => "Eq",
            Self::PartialEq => "PartialEq",
            Self::Hash => "Hash",
        }
    }
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

fn derive_marker_impl(definition: &StructDefinition, derive: BuiltinDerive) -> Statement {
    Statement::ImplDeclaration(ImplDefinition {
        generic_parameters: Vec::new(),
        trait_name: derive.name().to_string(),
        type_name: derived_self_type(definition),
        associated_types: Default::default(),
        methods: Vec::new(),
    })
}

struct DerivedMethodSignature<'a> {
    derive: BuiltinDerive,
    method_name: &'a str,
    parameters: Vec<Parameter>,
    return_type: Type,
}

fn derive_method_impl(
    definition: &StructDefinition,
    signature: DerivedMethodSignature<'_>,
    value: Expression,
) -> Statement {
    let DerivedMethodSignature {
        derive,
        method_name,
        parameters,
        return_type,
    } = signature;

    Statement::ImplDeclaration(ImplDefinition {
        generic_parameters: Vec::new(),
        trait_name: derive.name().to_string(),
        type_name: derived_self_type(definition),
        associated_types: Default::default(),
        methods: vec![Statement::Function {
            name: method_name.to_string(),
            generic_parameters: Vec::new(),
            parameters,
            return_type,
            body: Some(Box::new(vec![Statement::Return { value: Some(value) }])),
            is_pure: false,
            is_memoised: false,
            is_static: false,
            is_multi: false,
            is_throws: false,
            is_exported: false,
        }],
    })
}
