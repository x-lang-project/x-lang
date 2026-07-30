use super::Builder;
use crate::errors::MirLowerError;
use crate::{LocalDeclaration, LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::Type;

impl Builder<'_> {
    /// Freeze indices and owner slots before later rebinding.
    pub(super) fn freeze(
        &mut self,
        place: Place,
        prefix: &mut Vec<MirStatement>,
    ) -> Result<Place, MirLowerError> {
        Ok(match place {
            Place::Field {
                base,
                field_name,
                value_type,
            } => Place::Field {
                base: Box::new(self.freeze_implicit_base(*base, prefix)?),
                field_name,
                value_type,
            },
            Place::Index {
                base,
                index,
                value_type,
            } => {
                let base = self.freeze_implicit_base(*base, prefix)?;
                let index = if matches!(index, Operand::ConstantInteger(_)) {
                    index
                } else {
                    let value_type = self.body.operand_type(&index);

                    Operand::Copy(Box::new(self.snapshot(index, value_type, prefix)?))
                };

                Place::Index {
                    base: Box::new(base),
                    index,
                    value_type,
                }
            }
            Place::Dereference { base, value_type } => Place::Dereference {
                base: Box::new(self.freeze_base(*base, prefix)?),
                value_type,
            },
            other => other,
        })
    }

    fn freeze_base(
        &mut self,
        base: Place,
        prefix: &mut Vec<MirStatement>,
    ) -> Result<Place, MirLowerError> {
        let base = self.freeze(base, prefix)?;
        let value_type = self.body.place_type(&base);

        if matches!(value_type, Type::Reference { .. } | Type::RawPointer { .. }) {
            self.snapshot(Operand::Copy(Box::new(base)), value_type, prefix)
        } else {
            Ok(base)
        }
    }

    fn freeze_implicit_base(
        &mut self,
        base: Place,
        prefix: &mut Vec<MirStatement>,
    ) -> Result<Place, MirLowerError> {
        let mut base = self.freeze_base(base, prefix)?;

        loop {
            let value_type = self.body.place_type(&base);
            let inner = match value_type {
                Type::Reference { inner, .. } | Type::RawPointer { inner, .. } => *inner,
                _ => return Ok(base),
            };

            if !matches!(inner, Type::Reference { .. } | Type::RawPointer { .. }) {
                return Ok(base);
            }

            let dereferenced = Place::Dereference {
                base: Box::new(base),
                value_type: inner.clone(),
            };
            base = self.snapshot(Operand::Copy(Box::new(dereferenced)), inner, prefix)?;
        }
    }

    fn snapshot(
        &mut self,
        operand: Operand,
        value_type: Type,
        prefix: &mut Vec<MirStatement>,
    ) -> Result<Place, MirLowerError> {
        let maximum = self
            .body
            .locals
            .keys()
            .map(|identifier| identifier.0)
            .max()
            .unwrap_or(0);
        let identifier = LocalIdentifier(
            maximum
                .checked_add(1)
                .ok_or_else(|| super::super::unsupported("local ID overflow in owner context"))?,
        );
        self.body.locals.insert(
            identifier,
            LocalDeclaration {
                identifier,
                name: None,
                value_type,
            },
        );
        prefix.push(MirStatement::Assign(
            Place::Local(identifier),
            Rvalue::Use(operand),
        ));

        Ok(Place::Local(identifier))
    }
}
