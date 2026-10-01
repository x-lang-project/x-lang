use std::path::Path;

use super::{Configuration, PreparedRuntime, cache, prepared, supported_options};
use crate::LinkOptions;
use crate::tinycc::LinkOutcome;
use x_diagnostics::CompilationFailure;

fn self_contained_source(source: &str) -> bool {
    if source.contains('#')
        || source.contains("??=")
        || source.contains("??/")
        || source.contains("_Pragma")
        || source.contains("__pragma")
        || source.contains('\0')
    {
        return false;
    }
    // Reject line splices that could assemble a preprocessing directive.
    !source.match_indices('\\').any(|(index, _)| {
        source
            .as_bytes()
            .get(index.saturating_add(1))
            .is_some_and(u8::is_ascii_whitespace)
    }) && !source.contains("%:")
        && !source.contains(x_runtime::abi::PARALLEL_FOR)
}

pub(in crate::tinycc) fn compile_c(
    source: &str,
    output: &str,
    options: &LinkOptions<'_>,
) -> Result<LinkOutcome, CompilationFailure> {
    if !supported_options(options) || !output.is_ascii() || !self_contained_source(source) {
        return Ok(LinkOutcome::Unsupported(
            "generated C requires O0 native options and self-contained serial input",
        ));
    }

    let Some(config) = Configuration::from_environment()? else {
        return Ok(LinkOutcome::Unsupported("X_TCC_DIR is not configured"));
    };

    if !config.directory.as_os_str().is_ascii() || !config.ucrt.as_os_str().is_ascii() {
        return Ok(LinkOutcome::Unsupported(
            "TinyCC narrow filesystem paths require Clang for Unicode names",
        ));
    }

    let runtime = PreparedRuntime::new(&config)?;
    let mut state = runtime.executable()?;
    state.without_standard_includes()?;
    state.compile(source)?;
    prepared::attach_support(&mut state, &config.directory, &config.ucrt)?;
    state.output_file(Path::new(output))?;
    if options.reproducible && !cache::zero_pe_timestamp(Path::new(output))? {
        return Ok(LinkOutcome::Unsupported(
            "TinyCC output has a nonzero PE timestamp",
        ));
    }

    Ok(LinkOutcome::Linked)
}
