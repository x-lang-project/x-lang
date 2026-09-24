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
    pub(in crate::tinycc) fn link_elf(
        &mut self,
        objects: &[String],
        output: &str,
        options: &LinkOptions<'_>,
    ) -> Result<LinkOutcome, CompilationFailure> {
        if !supported_options(options) || !output.is_ascii() {
            return Ok(LinkOutcome::Unsupported("request requires Clang"));
        }

        if objects.is_empty() {
            return Ok(LinkOutcome::Unsupported("no object inputs"));
        }

        for object in objects {
            if !object.is_ascii() {
                return Ok(LinkOutcome::Unsupported(
                    "Unicode object paths require Clang",
                ));
            }

            let bytes = fs::read(object).map_err(failure)?;

            if !cache::elf_object(&bytes)
                || bytes
                    .windows(x_runtime::abi::PARALLEL_FOR.len())
                    .any(|part| part == x_runtime::abi::PARALLEL_FOR.as_bytes())
                || bytes
                    .windows(b"__chkstk\0".len())
                    .any(|part| part == b"__chkstk\0")
            {
                return Ok(LinkOutcome::Unsupported(
                    "object requires the Clang runtime",
                ));
            }
        }

        let _guard = TINYCC
            .lock()
            .map_err(|_| failure("TinyCC serialisation lock poisoned"))?;
        let mut state = self.fresh_state()?;

        for object in objects {
            state.add_file(Path::new(object))?;
        }

        if self.finish(&mut state, output, options).is_err() {
            return Ok(LinkOutcome::Unsupported(
                "TinyCC cannot link these ELF inputs",
            ));
        }

        Ok(LinkOutcome::Linked)
    }

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
