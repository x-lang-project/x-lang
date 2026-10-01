use std::{collections::BTreeMap, env, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LinkOptions, NativeToolchain, executable_cache};
use x_diagnostics::{CliError, CompilationFailure};

const BRIDGE: &str = "\n#include <stdarg.h>\nint x_jit_printf(const char *format, ...) { va_list args; va_start(args, format); int result = vprintf(format, args); va_end(args); return result; }\nvoid x_jit_flush(void) { fflush(NULL); }\n";
#[derive(Serialize, Deserialize)]
struct Entry {
    path: PathBuf,
    digest: String,
    files: BTreeMap<PathBuf, Option<Vec<u8>>>,
}
fn error(message: impl ToString) -> CompilationFailure {
    CliError::operational(message.to_string()).into_failure()
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn library() -> Result<PathBuf, CompilationFailure> {
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err(error("--jit requires Windows x86-64"));
    }

    let toolchain = NativeToolchain::discover()?;
    let identity = executable_cache::toolchain_identity()
        .ok_or_else(|| error("--jit requires the pinned LLVM toolchain"))?;
    let root = env::var_os("X_RUNTIME_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("x-lang-runtime-cache-v1"))
        .join("jit-v1");
    fs::create_dir_all(&root).map_err(error)?;
    let source = crate::linker::RuntimeSource(x_runtime::write_runtime_file(&root).map_err(error)?);
    let options = LinkOptions {
        optimisation_level: 2,
        reproducible: true,
        ..Default::default()
    };
    let object = crate::runtime_cache::runtime_object(&toolchain, &source, &options)
        .ok_or_else(|| error("--jit requires cacheable hosted runtime inputs; use native x run with custom toolchain flags"))?;
    let key = digest(
        format!(
            "jit-runtime-v2:{identity:?}:{:?}:{BRIDGE}:{:?}",
            Sha256::digest(fs::read(&object).map_err(error)?),
            env::vars_os().collect::<BTreeMap<_, _>>()
        )
        .as_bytes(),
    );
    let manifest = root.join(format!("{key}.json"));

    if let Some(entry) = fs::read(&manifest)
        .ok()
        .and_then(|second_value| serde_json::from_slice::<Entry>(&second_value).ok())
        && executable_cache::valid_files(&entry.files)
        && fs::read(&entry.path)
            .ok()
            .is_some_and(|bytes| digest(&bytes) == entry.digest)
    {
        return Ok(entry.path);
    }

    let mut bridge =
        x_utils::temporary::TemporaryFile::create(&root, "jit-bridge.c").map_err(error)?;
    bridge
        .write_and_sync(format!("#include <stdio.h>\n{BRIDGE}").as_bytes())
        .map_err(error)?;
    let output = root.join(format!("{key}-{}.dll", std::process::id()));
    let mut command = toolchain.command();
    command
        .arg("-shared")
        .arg("-fopenmp")
        .arg("-O2")
        .arg("-x")
        .arg("c")
        .arg(bridge.path())
        .arg("-x")
        .arg("none")
        .arg(&object)
        .arg("-o")
        .arg(&output);
    for name in x_runtime::abi::EXPORTED_FUNCTIONS.into_iter().chain([
        "getpagesize",
        "x_jit_printf",
        "x_jit_flush",
        "malloc",
        "calloc",
        "realloc",
        "free",
        "strlen",
        "memcpy",
        "memmove",
        "memset",
        "memcmp",
        "puts",
        "putchar",
        "fflush",
        "abort",
        "exit",
        "strcmp",
        "strncmp",
        "strcpy",
        "strncpy",
        "strcat",
        "strchr",
        "strstr",
        "qsort",
        "atoi",
        "strtol",
        "strtod",
        "sin",
        "cos",
        "tan",
        "sqrt",
        "pow",
        "fabs",
        "fmod",
        "floor",
        "ceil",
    ]) {
        command.arg(format!("-Wl,/export:{name}"));
    }

    command.arg("-###");
    let plan = command.output().map_err(error)?;
    let text = String::from_utf8_lossy(&plan.stderr);
    let current_arguments = text
        .lines()
        .filter(|line| line.trim_start().starts_with('"'))
        .filter_map(crate::fast_linker::parse_quoted_command)
        .find(|current_arguments| {
            current_arguments
                .first()
                .is_some_and(|argument| argument.contains("lld-link"))
        })
        .ok_or_else(|| error("could not capture JIT runtime link dependencies"))?;
    let files = executable_cache::library_files(&current_arguments, std::iter::empty())
        .ok_or_else(|| error("could not validate JIT runtime libraries"))?;
    let arguments = command
        .get_args()
        .filter(|argument| *argument != "-###")
        .map(|argument| argument.to_owned())
        .collect::<Vec<_>>();
    let status = toolchain
        .command()
        .args(arguments)
        .env("SOURCE_DATE_EPOCH", "0")
        .status()
        .map_err(error)?;

    if !status.success() {
        return Err(error(format!("JIT runtime preparation failed: {status}")));
    }

    if !executable_cache::valid_files(&files) {
        return Err(error(
            "JIT runtime dependencies changed during preparation; retry",
        ));
    }

    let entry = Entry {
        path: output.clone(),
        digest: digest(&fs::read(&output).map_err(error)?),
        files,
    };
    let mut temporary =
        x_utils::temporary::TemporaryFile::create(&root, "jit-runtime").map_err(error)?;
    temporary
        .write_and_sync(&serde_json::to_vec(&entry).map_err(error)?)
        .map_err(error)?;
    if manifest.exists() {
        let _ = fs::remove_file(&manifest);
    }

    let _ = temporary.install(&manifest);

    Ok(output)
}
