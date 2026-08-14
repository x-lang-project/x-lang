use crate::TypeChecker;
use crate::types;
use x_ast::Type;

impl TypeChecker {
    // Follow substitution chains iteratively to bound stack use.
    fn resolve_inference<'a>(&'a self, mut value_type: &'a Type) -> &'a Type {
        while let Type::Infer(identifier) = value_type {
            let Some(concrete) = self.substitutions.get(identifier) else {
                break;
            };

            value_type = concrete;
        }

        value_type
    }

    fn occurs_in(&self, identifier: usize, value_type: &Type) -> bool {
        self.resolve_inference(value_type).any_nested_type(|inner| {
            if let Type::Infer(nid) = inner {
                self.substitutions
                    .get(nid)
                    .map_or(identifier == *nid, |resolved| {
                        self.occurs_in(identifier, resolved)
                    })
            } else {
                false
            }
        })
    }
}
