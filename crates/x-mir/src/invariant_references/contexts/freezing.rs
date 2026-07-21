use super::Builder;
use crate::errors::MirLowerError;
use crate::{LocalDeclaration, LocalIdentifier, MirStatement, Operand, Place, Rvalue};
use x_ast::Type;

impl Builder<'_> {
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
