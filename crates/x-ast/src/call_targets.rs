use std::borrow::Cow;
use std::fmt;

use crate::Type;

mod text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSegment {
    pub name: String,
    pub type_arguments: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallPath {
    pub segments: Vec<CallSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallTarget {
    pub path: CallPath,
    pub resolution: Option<CallResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallResolution {
    Source(String),
    Builtin {
        name: String,
        type_arguments: Vec<Type>,
    },
}
