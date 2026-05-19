#[derive(Debug, Clone)]
pub struct JsonDiagnosticPayload {
    pub kind: String,
    pub repair_policies: Vec<String>,
    pub caller: Option<String>,
    pub callee: Option<String>,
}
impl JsonDiagnosticPayload {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            repair_policies: vec![],
            caller: None,
            callee: None,
        }
    }

    pub fn repair_policies(
        mut self,
        policies: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.repair_policies = policies.into_iter().map(Into::into).collect();
        self
    }

    pub fn caller(mut self, caller: impl Into<String>) -> Self {
        self.caller = Some(caller.into());
        self
    }

    pub fn callee(mut self, callee: impl Into<String>) -> Self {
        self.callee = Some(callee.into());
        self
    }
}
