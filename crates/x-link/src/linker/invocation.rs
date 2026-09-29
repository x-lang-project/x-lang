use super::*;

#[derive(Debug)]
pub(crate) struct RuntimeSource(pub(crate) String);

impl Drop for RuntimeSource {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[derive(Debug)]
enum RuntimeInput {
    None,
    Source(RuntimeSource),
    Object(String),
}

#[derive(Debug)]
pub(super) struct LinkInvocation {
    inputs: Vec<String>,
    runtime: RuntimeInput,
}
