use crate::Type;

struct SelfSubstituter<'a>(&'a Type);

pub fn substitute_self_in_type(value_type: &mut Type, self_type: &Type) {
    match value_type {
        Type::TypeParameter(name) | Type::Custom(name) if name == "Self" => {
            *value_type = self_type.clone();
        }
        _ => {
            for child in value_type.children_mut() {
                substitute_self_in_type(child, self_type);
            }
        }
    }
}
