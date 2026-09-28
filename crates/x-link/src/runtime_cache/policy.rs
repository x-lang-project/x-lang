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

pub(super) fn supported(options: &LinkOptions<'_>) -> bool {
    cfg!(all(windows, target_arch = "x86_64"))
        && !options.freestanding
        && env::var_os("X_CLANG").is_none()
        && options.target_triple.is_none_or(|triple| {
            x_utils::target::ProductTarget::parse(triple)
                == Some(x_utils::target::ProductTarget::WindowsMsvc)
        })
        && options.include_paths.is_empty()
        && options.passthrough_arguments.is_empty()
        && [
            "CPATH",
            "C_INCLUDE_PATH",
            "CPLUS_INCLUDE_PATH",
            "OBJC_INCLUDE_PATH",
            "CL",
            "_CL_",
            "CCC_OVERRIDE_OPTIONS",
            "CCC_ADD_ARGS",
            "DEPENDENCIES_OUTPUT",
            "SUNPRO_DEPENDENCIES",
            "CC_PRINT_OPTIONS",
            "CC_PRINT_OPTIONS_FILE",
            "CC_PRINT_HEADERS",
            "CC_PRINT_HEADERS_FILE",
            "CC_LOG_DIAGNOSTICS",
            "CC_LOG_DIAGNOSTICS_FILE",
            "CC_PRINT_PROC_STAT",
            "CC_PRINT_PROC_STAT_FILE",
            "CC_PRINT_INTERNAL_STAT",
        ]
        .iter()
        .all(|name| env::var_os(name).is_none())
}
