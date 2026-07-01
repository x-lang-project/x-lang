#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantBinding {
    pub name: String,
}

impl InvariantBinding {
    pub fn source_receiver() -> Self {
        Self {
            name: "self".into(),
        }
    }
}
