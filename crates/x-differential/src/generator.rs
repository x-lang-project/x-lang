//! Draw order determines the reproducible corpus.
use std::array;

use crate::{
    BooleanExpression, BooleanOperator, ComparisonOperator, Expression, Operator, StatefulCase,
};

const VALUE_CHOICES: u32 = 19;
const VALUE_OFFSET: i32 = 9;
const DIVISOR_CHOICES: u32 = 9;
const SHIFT_CHOICES: u32 = 4;
const COMPARISON_DEPTH: u8 = 2;

/// Seed zero remains an absorbing state for existing corpora.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Generator(u64);

impl Generator {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;

        self.0 as u32
    }

    fn value(&mut self) -> i32 {
        (self.next() % VALUE_CHOICES) as i32 - VALUE_OFFSET
    }

    fn arithmetic_operator(&mut self) -> Operator {
        match self.next() % 10 {
            0 => Operator::Add,
            1 => Operator::Sub,
            2 => Operator::Mul,
            3 => Operator::Div,
            4 => Operator::Rem,
            5 => Operator::BitAnd,
            6 => Operator::BitOr,
            7 => Operator::BitXor,
            8 => Operator::ShiftLeft,
            _ => Operator::ShiftRight,
        }
    }

    fn right_operand(&mut self, operator: Operator, depth: u8) -> Expression {
        match operator {
            Operator::Div | Operator::Rem => {
                Expression::Value((self.next() % DIVISOR_CHOICES + 1) as i32)
            }
            Operator::ShiftLeft | Operator::ShiftRight => {
                Expression::Value((self.next() % SHIFT_CHOICES) as i32)
            }
            _ => self.expression(depth),
        }
    }

    /// Generate at most `depth` binary levels; depths above three may overflow i32.
    pub fn expression(&mut self, depth: u8) -> Expression {
        if depth == 0 {
            return Expression::Value(self.value());
        }

        let operator = self.arithmetic_operator();
        let left = self.expression(depth - 1);
        let right = self.right_operand(operator, depth - 1);

        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn boolean_value(&mut self) -> BooleanExpression {
        BooleanExpression::Value(self.next() & 1 == 0)
    }

    fn comparison(&mut self) -> BooleanExpression {
        let operator = match self.next() % 6 {
            0 => ComparisonOperator::Equal,
            1 => ComparisonOperator::NotEqual,
            2 => ComparisonOperator::Less,
            3 => ComparisonOperator::LessEqual,
            4 => ComparisonOperator::Greater,
            _ => ComparisonOperator::GreaterEqual,
        };

        BooleanExpression::Comparison {
            operator,
            left: Box::new(self.expression(COMPARISON_DEPTH)),
            right: Box::new(self.expression(COMPARISON_DEPTH)),
        }
    }

    /// Comparisons contain depth-two integer trees in addition to the Boolean depth.
    pub fn boolean_expression(&mut self, depth: u8) -> BooleanExpression {
        if depth == 0 {
            return self.boolean_value();
        }

        match self.next() % 4 {
            0 => self.boolean_value(),
            1 => self.comparison(),
            2 => BooleanExpression::Not(Box::new(self.boolean_expression(depth - 1))),
            _ => BooleanExpression::Binary {
                operator: if self.next() & 1 == 0 {
                    BooleanOperator::And
                } else {
                    BooleanOperator::Or
                },
                left: Box::new(self.boolean_expression(depth - 1)),
                right: Box::new(self.boolean_expression(depth - 1)),
            },
        }
    }

    pub fn stateful_case(&mut self) -> StatefulCase {
        let values = array::from_fn(|_| self.value());
        let continue_after = (self.next() % (values.len() - 1) as u32 + 1) as usize;
        let break_after =
            continue_after + (self.next() % (values.len() - continue_after) as u32 + 1) as usize;

        StatefulCase {
            values,
            continue_after,
            break_after,
        }
    }
}
