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

impl CallTarget {
    pub fn new(path: CallPath) -> Self {
        Self {
            path,
            resolution: None,
        }
    }

    pub fn plain(name: impl Into<String>) -> Self {
        let name = name.into();

        Self::new(CallPath {
            segments: name
                .split("::")
                .map(|name| CallSegment {
                    name: name.to_owned(),
                    type_arguments: Vec::new(),
                })
                .collect(),
        })
    }

    pub fn from_path_text(name: &str) -> Self {
        Self::new(text::decode_path(name))
    }

    pub fn source_path(&self) -> String {
        self.path.to_string()
    }

    pub fn symbol_name(&self) -> Cow<'_, str> {
        match self.resolved_name() {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(self.source_path()),
        }
    }

    pub fn leaf_name(&self) -> &str {
        self.path
            .segments
            .last()
            .map(|segment| segment.name.as_str())
            .unwrap_or("")
    }

    pub fn leaf_type_arguments(&self) -> &[Type] {
        self.path
            .segments
            .last()
            .map(|segment| segment.type_arguments.as_slice())
            .unwrap_or_default()
    }

    pub fn owner_type_arguments(&self) -> Option<&[Type]> {
        let (_, owners) = self.path.segments.split_last()?;

        owners
            .iter()
            .find(|segment| !segment.type_arguments.is_empty())
            .map(|segment| segment.type_arguments.as_slice())
    }

    pub fn resolved_name(&self) -> Option<&str> {
        match &self.resolution {
            Some(CallResolution::Source(name) | CallResolution::Builtin { name, .. }) => Some(name),
            None => None,
        }
    }

    pub fn runtime_spelling(&self) -> Cow<'_, str> {
        match &self.resolution {
            Some(CallResolution::Builtin {
                name,
                type_arguments,
            }) if !type_arguments.is_empty() => Cow::Owned(format!(
                "{name}<{}>",
                type_arguments
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            _ => self.symbol_name(),
        }
    }

    pub fn resolve_to(&mut self, name: String) {
        self.resolution = Some(CallResolution::Source(name));
    }

    pub fn resolve_builtin_to(&mut self, name: String, type_arguments: Vec<Type>) {
        self.resolution = Some(CallResolution::Builtin {
            name,
            type_arguments,
        });
    }
}

impl From<String> for CallTarget {
    fn from(name: String) -> Self {
        Self::from_path_text(&name)
    }
}

impl From<&str> for CallTarget {
    fn from(name: &str) -> Self {
        Self::from_path_text(name)
    }
}

impl fmt::Display for CallPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.segments.iter().enumerate() {
            if index != 0 {
                write!(formatter, "::")?;
            }

            write!(formatter, "{}", segment.name)?;

            if !segment.type_arguments.is_empty() {
                write!(formatter, "<")?;
                for (index, argument) in segment.type_arguments.iter().enumerate() {
                    if index != 0 {
                        write!(formatter, ", ")?;
                    }

                    write!(formatter, "{argument}")?;
                }

                write!(formatter, ">")?;
            }
        }

        Ok(())
    }
}
