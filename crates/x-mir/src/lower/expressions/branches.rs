use super::*;
use crate::Place;
use x_ast::MatchArm;

impl MirLowerer {
    pub(super) fn lower_if_operand(
        &mut self,
        expression: Expression,
        expected_type: Option<&Type>,
    ) -> Result<Operand, MirLowerError> {
        let value_type = expected_type
            .cloned()
            .unwrap_or_else(|| expression.value_type.clone());
        let result = self.allocate_local(None, value_type.clone());
        self.lower_if_expression_into_place(expression, result, value_type)?;

        Ok(Operand::Move(Box::new(Place::Local(result))))
    }

    pub(super) fn lower_match_operand(
        &mut self,
        scrutinee: &Expression,
        arm: &MatchArm,
        value_type: Type,
    ) -> Result<Operand, MirLowerError> {
        let result = self.allocate_local(None, value_type.clone());
        self.lower_irrefutable_match(scrutinee.clone(), arm, Some((result, value_type)))?;

        Ok(Operand::Move(Box::new(Place::Local(result))))
    }

    pub(super) fn lower_boolean_operand(
        &mut self,
        expression: Expression,
    ) -> Result<Operand, MirLowerError> {
        let result = self.allocate_local(None, Type::Boolean);
        self.lower_boolean_expression_into_place(expression, Place::Local(result))?;

        Ok(Operand::Move(Box::new(Place::Local(result))))
    }
}
