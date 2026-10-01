use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LinkOptions, NativeToolchain};
use x_diagnostics::{CompilationFailure, link_messages};

const OUTPUT_TOKEN: &str = "__X_LINK_OUTPUT__";
const INPUT_PREFIX: &str = "__X_LINK_INPUT_";

#[derive(Serialize, Deserialize)]
struct LinkPlan {
    arguments: Vec<String>,
    stamps: BTreeMap<PathBuf, (u64, u128)>,
}

pub(crate) struct LinkRequest<'a> {
    pub(crate) arguments: &'a [String],
    pub(crate) inputs: &'a [String],
    pub(crate) output: &'a str,
    pub(crate) options: &'a LinkOptions<'a>,
}

pub(crate) fn try_link(
    toolchain: &NativeToolchain,
    request: LinkRequest<'_>,
) -> Option<Result<(), CompilationFailure>> {
    let LinkRequest {
        arguments: current_arguments,
        inputs,
        output,
        options,
    } = request;

    if !eligible(options) || inputs.is_empty() {
        return None;
    }

    let compiler = toolchain.compiler().canonicalize().ok()?;
    let linker = compiler.parent()?.join("lld-link.exe");

    if !linker.is_file() {
        return None;
    }

    let normalized = normalize(current_arguments, inputs, output);
    let cache_path = plan_path(toolchain, &compiler, &linker, &normalized)?;
    let plan = read_plan(&cache_path, &linker, inputs.len()).or_else(|| {
        let plan = derive_plan(toolchain, &request, &linker)?;

        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(bytes) = serde_json::to_vec(&plan) {
            let _ = fs::write(&cache_path, bytes);
        }

        Some(plan)
    })?;
    let captured = if crate::executable_cache::capturing() {
        crate::executable_cache::library_files(&plan.arguments, plan.stamps.keys().cloned())
    } else {
        None
    };

    let arguments = expand(&plan.arguments, inputs, output)?;
    let mut command = Command::new(linker);
    command.args(arguments);
    if options.reproducible {
        command.env("SOURCE_DATE_EPOCH", "0");
    }

    let status = match command.status() {
        Ok(status) => status,
        Err(_) => return None,
    };

    Some(if !status.success() {
        Err(link_messages::clang_failure(status))
    } else if !Path::new(output).is_file() {
        Err(link_messages::output_missing(output))
    } else {
        crate::executable_cache::record_files(
            captured.filter(crate::executable_cache::valid_files),
        );

        Ok(())
    })
}

fn eligible(options: &LinkOptions<'_>) -> bool {
    cfg!(all(windows, target_arch = "x86_64"))
        && env::var_os("X_CLANG").is_none()
        && !options.freestanding
        && !options.gc_sections
        && options.libs.is_empty()
        && options.lib_paths.is_empty()
        && options.include_paths.is_empty()
        && options.passthrough_arguments.is_empty()
        && options.target_triple.is_none_or(|triple| {
            x_utils::target::ProductTarget::parse(triple)
                == Some(x_utils::target::ProductTarget::WindowsMsvc)
        })
}

fn normalize(arguments: &[String], inputs: &[String], output: &str) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| {
            if argument == output {
                OUTPUT_TOKEN.to_owned()
            } else if let Some(index) = inputs.iter().position(|input| input == argument) {
                format!("{INPUT_PREFIX}{index}__")
            } else if argument == &format!("-out:{output}") {
                format!("-out:{OUTPUT_TOKEN}")
            } else {
                argument.clone()
            }
        })
        .collect()
}

fn expand(arguments: &[String], inputs: &[String], output: &str) -> Option<Vec<String>> {
    arguments
        .iter()
        .map(|argument| {
            if argument == &format!("-out:{OUTPUT_TOKEN}") {
                Some(format!("-out:{output}"))
            } else if let Some(index) = argument
                .strip_prefix(INPUT_PREFIX)
                .and_then(|index| index.strip_suffix("__"))
            {
                inputs.get(index.parse::<usize>().ok()?).cloned()
            } else {
                Some(argument.clone())
            }
        })
        .collect()
}

fn stamp(path: &Path) -> Option<(u64, u128)> {
    let metadata = path.metadata().ok()?;

    Some((
        metadata.len(),
        metadata
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos(),
    ))
}

// Parse Clang's quoted -### arguments without invoking a shell.

mod plans;
pub(crate) use plans::*;
