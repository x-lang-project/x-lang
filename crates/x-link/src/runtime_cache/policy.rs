use super::*;

// Other quoted includes depend on the source directory and bypass this cache.
pub(super) fn cacheable_source(blob: &str) -> bool {
    let prefix = format!(
        "{}\n#define X_RUNTIME_AMALGAMATED 1\n",
        x_runtime::RUNTIME_ABI_HEADER
    );
    let source = if let Some(source) = blob.strip_prefix(&prefix) {
        source
            .replace(
                "#ifndef X_RUNTIME_AMALGAMATED\r\n#include \"x_runtime.h\"\r\n#endif",
                "",
            )
            .replace(
                "#ifndef X_RUNTIME_AMALGAMATED\n#include \"x_runtime.h\"\n#endif",
                "",
            )
    } else {
        blob.to_owned()
    };

    !source.contains("X_RUNTIME_AMALGAMATED")
        && !source.lines().any(|line| {
            line.trim_start().starts_with('#') && line.contains("include") && line.contains('"')
        })
}
