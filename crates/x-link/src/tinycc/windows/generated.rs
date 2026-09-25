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
