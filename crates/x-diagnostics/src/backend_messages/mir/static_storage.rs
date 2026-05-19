use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticInitializerError {
    InvalidRepeatLength { name: String, value: i64 },
    NonConstantRepeatLength { name: String },
    MissingArrayInitializer { name: String },
}

impl Display for StaticInitializerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRepeatLength { name, value } => formatter.write_str(&Error::new(format!("static `{name}` array repeat length must be a non-negative usize, got {value}")).render_detail()),
            Self::NonConstantRepeatLength { name } => formatter.write_str(&Error::new(format!("static `{name}` array repeat length must be a constant integer")).render_detail()),
            Self::MissingArrayInitializer { name } => formatter.write_str(&Error::new(format!("static `{name}` has unsized array type but no array initialiser to derive storage length from")).render_detail()),
        }
    }
}

impl error::Error for StaticInitializerError {}
