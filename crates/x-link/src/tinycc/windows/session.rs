use super::*;
use resources::{Resources, read_guard};

mod resources;
pub(in crate::tinycc) struct Session {
    // Release file guards before RuntimeObject's possible temporary cleanup.
    _runtime_guard: fs::File,
    _ucrt_guard: fs::File,
    runtime: cache::RuntimeObject,
    resources: Resources,
    library: Library,
    config: Configuration,
}

impl Session {
    pub(in crate::tinycc) fn compile_c(
        &mut self,
        source: &str,
        output: &str,
        options: &LinkOptions<'_>,
    ) -> Result<LinkOutcome, CompilationFailure> {
        if !supported_options(options)
            || !output.is_ascii()
            || source.contains(x_runtime::abi::PARALLEL_FOR)
        {
            return Ok(LinkOutcome::Unsupported("request requires Clang"));
        }

        let _guard = TINYCC
            .lock()
            .map_err(|_| failure("TinyCC serialisation lock poisoned"))?;
        let mut state = self.fresh_state()?;
        state.compile(source)?;
        self.finish(&mut state, output, options)?;

        Ok(LinkOutcome::Linked)
    }

    fn fresh_state(&self) -> Result<library::State<'_>, CompilationFailure> {
        prepared::executable_state(&self.library, &self.runtime.path)
    }

    fn finish(
        &self,
        state: &mut library::State<'_>,
        output: &str,
        options: &LinkOptions<'_>,
    ) -> Result<(), CompilationFailure> {
        prepared::attach_support(state, &self.resources.root, &self.config.ucrt)?;
        state.output_file(Path::new(output))?;
        if options.reproducible && !cache::zero_pe_timestamp(Path::new(output))? {
            return Err(failure("TinyCC output has a nonzero PE timestamp"));
        }

        Ok(())
    }
}
