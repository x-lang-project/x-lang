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

impl LinkInvocation {
    pub(super) fn prepare(
        object_files: &[String],
        options: &LinkOptions<'_>,
    ) -> Result<Self, CompilationFailure> {
        let inputs = valid_object_inputs(object_files)?;

        if inputs.is_empty() && options.freestanding {
            return Err(link_messages::object_inputs_missing());
        }

        let runtime = if options.freestanding {
            RuntimeInput::None
        } else {
            let path = write_runtime_file(Path::new("."))
                .map_err(|error| LinkDiagnostic::RuntimeSourceWrite.into_failure(error))?;

            RuntimeInput::Source(RuntimeSource(path))
        };

        Ok(Self { inputs, runtime })
    }
}
