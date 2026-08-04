use super::*;

impl TypeChecker {
    pub fn substitute_type(&mut self, value_type: &Type, map: &HashMap<String, Type>) -> Type {
        match value_type {
            Type::Custom(name) | Type::TypeParameter(name) => {
                map.get(name).cloned().unwrap_or_else(|| value_type.clone())
            }
            Type::GenericInstance {
                name,
                type_arguments,
            } => {
                let concrete_arguments: Vec<Type> = type_arguments
                    .iter()
                    .map(|type_value| self.substitute_type(type_value, map))
                    .collect();

                Type::GenericInstance {
                    name: name.clone(),
                    type_arguments: concrete_arguments,
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.substitute_type(inner, map))),

            Type::Simd { element, lanes } => Type::Simd {
                element: Box::new(self.substitute_type(element, map)),
                lanes: *lanes,
            },
            Type::Reference {
                lifetime,
                is_mut,
                is_unique,
                inner,
            } => Type::Reference {
                lifetime: lifetime.clone(),
                is_mut: *is_mut,
                is_unique: *is_unique,
                inner: Box::new(self.substitute_type(inner, map)),
            },
            Type::RawPointer { is_mut, inner } => Type::RawPointer {
                is_mut: *is_mut,
                inner: Box::new(self.substitute_type(inner, map)),
            },
            Type::Function {
                parameters,
                return_value,
            } => Type::Function {
                parameters: parameters
                    .iter()
                    .map(|type_value| self.substitute_type(type_value, map))
                    .collect(),
                return_value: Box::new(self.substitute_type(return_value, map)),
            },
            _ => value_type.clone(),
        }
    }
}
