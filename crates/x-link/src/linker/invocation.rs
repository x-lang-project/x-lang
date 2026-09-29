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

pub(super) struct RenderedLink {
    pub(super) arguments: Vec<String>,
    pub(super) inputs: Vec<String>,
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

fn append_target_arguments(arguments: &mut Vec<String>, target_triple: Option<&str>) {
    if let Some(triple) = target_triple {
        if let Some(spec) = x_utils::target::TargetSpec::parse(triple) {
            arguments.extend(spec.compiler_arguments());
        } else {
            arguments.push(format!("--target={triple}"));
        }
    }
}

fn append_codegen_arguments(arguments: &mut Vec<String>, options: &LinkOptions<'_>) {
    arguments.push(
        x_utils::optimization::NativeOptimization::new(options.optimisation_level).argument(),
    );
    if options.gc_sections {
        arguments.push("-ffunction-sections".to_string());
        arguments.push("-fdata-sections".to_string());

        if cfg!(target_os = "macos") {
            arguments.push("-Wl,-dead_strip".to_string());
        } else if !cfg!(target_os = "windows") {
            arguments.push("-Wl,--gc-sections".to_string());
        }
    }

    if options.reproducible && !cfg!(target_os = "windows") {
        arguments.push("-Wl,--build-id=none".to_string());
    }
}
