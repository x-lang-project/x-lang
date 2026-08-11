use x_ast::{Expression, ExpressionKind, StringPart, UnaryOperator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum StaticValue {
    Integer(i64),
    Float(u64),
    Boolean(bool),
    Character(char),
    String(String),
    Negated(Box<StaticValue>),
}

impl StaticValue {
    pub(super) fn from_expression(expression: &Expression) -> Option<Self> {
        Some(match &expression.kind {
            ExpressionKind::Integer(value) => Self::Integer(*value),
            ExpressionKind::Float(value) if value.is_finite() => Self::Float(value.to_bits()),
            ExpressionKind::Boolean(value) => Self::Boolean(*value),
            ExpressionKind::Character(value) => Self::Character(*value),
            ExpressionKind::String(literal) => {
                let mut value = String::new();

                for part in &literal.parts {
                    let StringPart::Text(text) = part else {
                        return None;
                    };

                    value.push_str(text);
                }

                Self::String(value)
            }
            ExpressionKind::UnaryOperator {
                operator: UnaryOperator::Negate,
                expression,
            } => Self::Negated(Box::new(Self::from_expression(expression)?)),
            _ => return None,
        })
    }

    fn encode(&self) -> String {
        match self {
            Self::Integer(value) => format!("i{:016x}", *value as u64),
            Self::Float(bits) => format!("f{bits:016x}"),
            Self::Boolean(value) => format!("b{}", u8::from(*value)),
            Self::Character(value) => format!("c{:08x}", *value as u32),
            Self::String(value) => format!("s{}", hex(value.as_bytes())),
            Self::Negated(value) => format!("n{}", value.encode()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct SpecializationKey {
    pub function: String,
    pub arguments: Vec<(String, StaticValue)>,
}

impl SpecializationKey {
    // Hex payloads keep symbol delimiters unambiguous.
    pub fn symbol(&self) -> String {
        let suffix = self
            .arguments
            .iter()
            .map(|(name, value)| format!("{}_{}", hex(name.as_bytes()), value.encode()))
            .collect::<Vec<_>>()
            .join("__");

        format!(
            "{}__static_{}",
            self.function,
            if suffix.is_empty() { "unit" } else { &suffix }
        )
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
