use super::*;

pub(super) fn plan_path(
    toolchain: &NativeToolchain,
    compiler: &Path,
    linker: &Path,
    arguments: &[String],
) -> Option<PathBuf> {
    let environment = env::vars_os().collect::<BTreeMap<_, _>>();
    let identity = (
        "host-link-plan-v1",
        compiler,
        toolchain.version(),
        toolchain.cache_identity()?,
        NativeToolchain::tool_identity(linker)?,
        NativeToolchain::discovery_fingerprint()?,
        arguments,
        environment,
        env::current_dir().ok()?,
    );
    let digest = Sha256::digest(format!("{identity:?}").as_bytes());
    let key = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let directory = env::var_os("X_RUNTIME_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);

    Some(
        directory
            .join("x-lang-native-link-v1")
            .join(format!("{key}.json")),
    )
}
