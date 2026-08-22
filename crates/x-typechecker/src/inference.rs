use crate::types;
use crate::{TypeChecker, errors::TypeError};
use x_ast::Type;

impl TypeChecker {
    pub fn fresh_type(&mut self) -> Type {
        let identifier = self.next_infer_identifier;
        self.next_infer_identifier += 1;

        Type::Infer(identifier)
    }

    pub fn resolve(&self, value_type: &Type) -> Type {
        self.resolve_inference(value_type).clone()
    }

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

    pub fn unify(&mut self, value_type: &Type, current_value_type: &Type) -> Result<(), TypeError> {
        let value_type = self.resolve_type(&self.resolve(value_type))?;
        let current_value_type = self.resolve_type(&self.resolve(current_value_type))?;

        if value_type == current_value_type {
            return Ok(());
        }

        if value_type == Type::Unknown || current_value_type == Type::Unknown {
            return Ok(());
        }

        if matches!(
            (&value_type, &current_value_type),
            (Type::TypeParameter(_), _) | (_, Type::TypeParameter(_))
        ) {
            return Ok(());
        }

        if value_type == Type::Never || current_value_type == Type::Never {
            return Ok(());
        }

        if self.types_compatible(&value_type, &current_value_type) {
            return Ok(());
        }

        if self.is_integer_primitive(&value_type) && self.is_integer_primitive(&current_value_type)
        {
            return Ok(());
        }

        match (value_type, current_value_type) {
            (
                Type::RawPointer {
                    is_mut: m1,
                    inner: i1,
                },
                Type::RawPointer {
                    is_mut: m2,
                    inner: i2,
                },
            ) => {
                if m1 && !m2 {
                    return Err(crate::type_mismatch!(
                        format!("RawPtr (mut: {m1})"),
                        format!("RawPtr (mut: {m2})")
                    ));
                }

                if !self.storage_types_compatible(&i1, &i2) {
                    return Err(crate::type_mismatch!(i1, i2));
                }

                self.unify(&i1, &i2)
            }

            (Type::Infer(identifier), concrete) | (concrete, Type::Infer(identifier)) => {
                if self.occurs_in(identifier, &concrete) {
                    return Err(crate::type_mismatch!("finite type", "recursive type"));
                }

                self.substitutions.insert(identifier, concrete);

                Ok(())
            }
            (
                Type::Reference {
                    lifetime: lifetime1,
                    is_mut: m1,
                    is_unique: u1,
                    inner: i1,
                    ..
                },
                Type::Reference {
                    lifetime: lifetime2,
                    is_mut: m2,
                    is_unique: u2,
                    inner: i2,
                    ..
                },
            ) => {
                if (m1 && !m2)
                    || (u1 && !u2)
                    || !types::lifetimes_compatible(&lifetime1, &lifetime2)
                    || !self.storage_types_compatible(&i1, &i2)
                {
                    let expected = Type::Reference {
                        lifetime: lifetime1,
                        is_mut: m1,
                        is_unique: u1,
                        inner: i1,
                    };
                    let actual = Type::Reference {
                        lifetime: lifetime2,
                        is_mut: m2,
                        is_unique: u2,
                        inner: i2,
                    };

                    return Err(crate::type_mismatch!(expected, actual));
                }

                self.unify(&i1, &i2)
            }

            (Type::Array(i1), Type::Array(i2)) => self.unify(&i1, &i2),
            (Type::FixedSizedArray(i1, len1), Type::FixedSizedArray(i2, len2)) => {
                if len1 != len2 {
                    return Err(crate::type_mismatch!(
                        format!("array length {len1}"),
                        format!("array length {len2}")
                    ));
                }

                self.unify(&i1, &i2)
            }
            (Type::FixedSizedArray(i1, _), Type::Array(i2))
            | (Type::Array(i2), Type::FixedSizedArray(i1, _)) => self.unify(&i1, &i2),
            (
                Type::Simd {
                    element: i1,
                    lanes: len1,
                },
                Type::Simd {
                    element: i2,
                    lanes: len2,
                },
            ) => {
                if len1 != len2 {
                    return Err(crate::type_mismatch!(
                        format!("SIMD lane count {len1}"),
                        format!("SIMD lane count {len2}")
                    ));
                }

                self.unify(&i1, &i2)
            }

            (Type::Tuple(e1), Type::Tuple(e2)) => {
                if e1.len() != e2.len() {
                    return Err(crate::type_mismatch!(
                        format!("{:?}", e1),
                        format!("{:?}", e2)
                    ));
                }

                for (t1, t2) in e1.iter().zip(e2.iter()) {
                    self.unify(t1, t2)?;
                }

                Ok(())
            }

            (expected, got) => Err(crate::type_mismatch!(expected, got)),
        }
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
