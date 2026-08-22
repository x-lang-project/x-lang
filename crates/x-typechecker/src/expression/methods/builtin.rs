use super::*;
use x_ast::intrinsics::SliceOperation;

fn array_method(name: &str) -> Option<SliceOperation> {
    if matches!(
        name,
        "len"
            | "is_empty"
            | "has_duplicate_by"
            | "unique_by"
            | "first"
            | "last"
            | "get"
            | "contains"
            | "sum"
    ) {
        SliceOperation::resolve(name)
    } else {
        None
    }
}

impl TypeChecker {
    pub(super) fn check_array_method(
        &mut self,
        receiver_base: Type,
        method_base: String,
        arguments: &mut [Expression],
    ) -> Result<Type, TypeError> {
        let element = match &receiver_base {
            Type::Array(inner) | Type::FixedSizedArray(inner, _) => (**inner).clone(),
            _ => {
                return Err(TypeError::UnresolvedProperty {
                    field: method_base,
                    value_type: receiver_base,
                });
            }
        };

        let pointer = Type::RawPointer {
            is_mut: false,
            inner: Box::new(element.clone()),
        };
        let operation =
            array_method(&method_base).ok_or_else(|| TypeError::UnresolvedProperty {
                field: method_base.clone(),
                value_type: receiver_base.clone(),
            })?;
        let (parameters, result) = match operation {
            SliceOperation::Len => (vec![], Type::USize),
            SliceOperation::IsEmpty | SliceOperation::HasDuplicateBy | SliceOperation::UniqueBy => {
                (vec![], Type::Boolean)
            }
            SliceOperation::First | SliceOperation::Last => (vec![], pointer),
            SliceOperation::Get => (vec![Type::USize], pointer),
            SliceOperation::Contains => (vec![element.clone()], Type::Boolean),
            SliceOperation::Sum if arguments.len() <= 1 => {
                if !self.is_integer_primitive(&element) && !matches!(element, Type::F32 | Type::F64)
                {
                    return Err(crate::type_mismatch!(
                        "numeric array element for sum",
                        element.to_string()
                    ));
                }

                (vec![element.clone(); arguments.len()], element.clone())
            }
            _ => {
                return Err(TypeError::UnresolvedProperty {
                    field: method_base,
                    value_type: receiver_base,
                });
            }
        };

        if arguments.len() != parameters.len() {
            return Err(TypeError::ArityMismatch {
                expected: parameters.len(),
                got: arguments.len(),
                symbol_name: format!("array::{method_base}"),
            });
        }

        for (argument, expected) in arguments.iter_mut().zip(parameters.iter()) {
            let actual = self.check_expression(argument)?;
            let actual = if operation == SliceOperation::Contains {
                match actual {
                    Type::Reference { inner, .. } => *inner,
                    other => other,
                }
            } else {
                actual
            };

            self.unify(expected, &actual)?;
        }

        Ok(result)
    }

    pub(super) fn check_string_method(
        receiver_base: Type,
        method_base: String,
        arguments: &mut [Expression],
    ) -> Result<Type, TypeError> {
        if !arguments.is_empty() {
            return Err(TypeError::ArityMismatch {
                expected: 0,
                got: arguments.len(),
                symbol_name: format!("str::{method_base}"),
            });
        }

        match method_base.as_str() {
            "len" => Ok(Type::USize),
            "as_str" => Ok(Type::String),
            _ => Err(TypeError::UnresolvedProperty {
                field: method_base,
                value_type: receiver_base,
            }),
        }
    }
}
