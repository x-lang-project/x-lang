use std::collections::BTreeMap;

use crate::Layout;
use crate::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct ExternParameter {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterRole {
    Argument,
    Receiver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub role: ParameterRole,
    pub name: String,
    pub value_type: Type,
    pub is_static: bool,
    pub is_mut: bool,
}

impl Parameter {
    pub fn is_receiver(&self) -> bool {
        self.role == ParameterRole::Receiver
    }
}
