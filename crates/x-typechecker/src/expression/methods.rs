mod lowering;
mod receiver;

use super::*;
use lowering::CheckedMethod;
use x_ast::SourceError;
use x_ast::intrinsics::PointerOperation;
use x_diagnostics::frontend_messages::FrontendMessage;

mod registered;

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
    pub(super) fn check_method_call(
        &mut self,
        object: &mut Box<Expression>,
        target: &x_ast::CallTarget,
        arguments: &mut Vec<Expression>,
    ) -> Result<CheckedMethod, TypeError> {
        let receiver_type = self.check_expression(object)?;

        if target.path.segments.len() != 1 {
            return Err(TypeError::UnresolvedProperty {
                field: target.source_path(),
                value_type: receiver_type,
            });
        }

        let method = target.leaf_name();
        let resolved_receiver = self.resolve(&receiver_type);

        if matches!(resolved_receiver, Type::RawPointer { .. })
            && pointer::resolve_method(method) == Some(PointerOperation::Offset)
        {
            let [index] = arguments.as_mut_slice() else {
                return Err(TypeError::ArityMismatch {
                    expected: 1,
                    got: arguments.len(),
                    symbol_name: "raw pointer offset".into(),
                });
            };

            let index_type = self.check_expression(index)?;
            let resolved_index = self.resolve(&index_type);

            if !self.is_integer_primitive(&resolved_index) {
                return Err(crate::type_mismatch!(
                    "integer pointer offset",
                    format!("{:?}", resolved_index)
                ));
            }

            object.value_type = resolved_receiver.clone();
            index.value_type = resolved_index;

            return Ok(CheckedMethod::Builtin(resolved_receiver));
        }

        let mut primitive_receiver = self.resolve(&receiver_type);

        while let Type::Reference { inner, .. } = primitive_receiver {
            primitive_receiver = *inner;
        }

        if matches!(primitive_receiver, Type::RawPointer { .. }) {
            let method_base = method;
            let (parameters, result) = pointer::resolve_method(method_base)
                .and_then(|operation| pointer::signature(operation, &primitive_receiver))
                .ok_or_else(|| TypeError::UnresolvedProperty {
                    field: method.to_owned(),
                    value_type: primitive_receiver.clone(),
                })?;

            if arguments.len() != parameters.len() {
                return Err(TypeError::ArityMismatch {
                    expected: parameters.len(),
                    got: arguments.len(),
                    symbol_name: format!("ptr::{method_base}"),
                });
            }

            for (argument, expected) in arguments.iter_mut().zip(parameters.iter()) {
                let actual = self.check_expression(argument)?;
                self.unify(expected, &actual)?;
                argument.value_type = self.resolve(&actual);
            }

            return Ok(CheckedMethod::Builtin(result));
        }

        let receiver_base = Self::unwrap_type_references(self.resolve(&receiver_type));
        let method_base = method.to_string();

        if matches!(receiver_base, Type::Array(_) | Type::FixedSizedArray(_, _)) {
            return self
                .check_array_method(receiver_base, method_base, arguments)
                .map(CheckedMethod::Builtin);
        }

        if matches!(receiver_base, Type::String) {
            return Self::check_string_method(receiver_base, method_base, arguments)
                .map(CheckedMethod::Builtin);
        }

        let call = self.check_registered_method(
            object,
            MethodReceiver {
                inferred: receiver_type,
                base: receiver_base,
            },
            method,
            arguments,
        )?;

        self.lower_declared_method_call(object, target, arguments, call)
    }

    fn resolve_method_owner(
        &self,
        struct_name: &str,
        method_base: &str,
    ) -> Result<String, TypeError> {
        Ok(
            if self
                .symbols
                .get_struct_method(struct_name, method_base)
                .is_some()
            {
                struct_name.to_owned()
            } else {
                self.symbols
                    .records
                    .instantiations()
                    .iter()
                    .find_map(|((origin, arguments), mangled)| {
                        if mangled != struct_name {
                            return None;
                        }

                        let arguments = arguments
                            .iter()
                            .map(|argument| argument.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let typed_origin = format!("{origin}<{arguments}>");

                        self.symbols
                            .get_struct_method(&typed_origin, method_base)
                            .is_some()
                            .then_some(typed_origin)
                            .or_else(|| {
                                self.symbols
                                    .get_struct_method(origin, method_base)
                                    .is_some()
                                    .then(|| origin.clone())
                            })
                    })
                    .or_else(|| self.matching_method_owner(struct_name, method_base))
                    .ok_or_else(|| TypeError::UnresolvedProperty {
                        field: method_base.to_owned(),
                        value_type: Type::Custom(struct_name.to_owned()),
                    })?
            },
        )
    }

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
