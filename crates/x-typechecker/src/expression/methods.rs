mod lowering;

use super::*;
use lowering::CheckedMethod;
use x_ast::SourceError;
use x_ast::intrinsics::PointerOperation;
use x_diagnostics::frontend_messages::FrontendMessage;

mod builtin;
mod pointer;

struct DeclaredMethodCall {
    return_type: Type,
    callee: String,
    receiver_parameter: Type,
}

struct MethodReceiver {
    inferred: Type,
    base: Type,
}

impl TypeChecker {
    fn method_receiver_type_arguments(&self, receiver_base: &Type, struct_name: &str) -> Vec<Type> {
        if let Type::GenericInstance { type_arguments, .. } = receiver_base {
            type_arguments.clone()
        } else {
            self.symbols
                .records
                .arguments(struct_name)
                .cloned()
                .unwrap_or_default()
        }
    }

    fn specialise_method_signature(
        &mut self,
        signature: &mut FunctionSignature,
        method_owner: &str,
        receiver_type_arguments: &[Type],
    ) {
        if !receiver_type_arguments.is_empty()
            && let Some(sdef) = self.symbols.records.templates().get(method_owner)
        {
            let mut substitutions = HashMap::new();

            if let Some(generic_parameters) = &sdef.generic_parameters {
                for (parameter_name, argument_type) in generic_parameters
                    .iter()
                    .filter_map(x_ast::GenericParameter::non_lifetime_name)
                    .zip(receiver_type_arguments.iter())
                {
                    substitutions.insert(parameter_name.clone(), argument_type.clone());
                }
            }
            signature.parameter_types = signature
                .parameter_types
                .iter()
                .map(|value_type| self.substitute_type(value_type, &substitutions))
                .collect();
            signature.return_type = self.substitute_type(&signature.return_type, &substitutions);
        }
    }

    fn matching_method_owner(&self, struct_name: &str, method_base: &str) -> Option<String> {
        let requested_origin = struct_name.split('<').next().unwrap_or(struct_name);
        let mut matches = self.symbols.struct_methods.keys().filter(|candidate| {
            candidate.split('<').next() == Some(requested_origin)
                && self
                    .symbols
                    .get_struct_method(candidate, method_base)
                    .is_some()
        });
        let only = matches.next()?;

        matches.next().is_none().then(|| only.clone())
    }
}
