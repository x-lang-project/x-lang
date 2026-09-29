use super::*;

pub(super) fn compile_flags(options: &LinkOptions<'_>) -> Vec<String> {
    let mut flags = vec![
        "-fopenmp".into(),
        x_utils::optimization::NativeOptimization::new(options.optimisation_level).argument(),
    ];

    if let Some(target) = options.target_triple {
        flags.push(format!("--target={target}"));
    }

    if options.gc_sections {
        flags.extend(["-ffunction-sections".into(), "-fdata-sections".into()]);
    }

    flags
}

pub(super) fn create_entry(build: &RuntimeBuild<'_, '_>, root: &Path) -> io::Result<PathBuf> {
    let key = build.key;
    let temporary = root.join(format!(
        ".building-{}-{}",
        std::process::id(),
        NEXT_ENTRY.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&temporary)?;
    let _temporary_guard = TemporaryEntry(temporary.clone());
    let result = compile_entry(build, &temporary);

    match result {
        Ok(manifest) => {
            let manifest_bytes = serde_json::to_vec(&manifest).map_err(io::Error::other)?;
            fs::write(temporary.join("manifest.json"), &manifest_bytes)?;
            let destination = root.join(digest(&manifest_bytes));

            match fs::rename(&temporary, &destination) {
                Ok(()) => Ok(destination.join("runtime.obj")),
                Err(error) => {
                    cleanup_entry(&temporary);

                    if valid_entry(&destination, key) {
                        Ok(destination.join("runtime.obj"))
                    } else {
                        Err(error)
                    }
                }
            }
        }
        Err(error) => {
            cleanup_entry(&temporary);

            Err(error)
        }
    }
}

pub(super) fn compile_entry(
    build: &RuntimeBuild<'_, '_>,
    directory: &Path,
) -> io::Result<Manifest> {
    let RuntimeBuild {
        toolchain,
        source,
        options,
        key,
    } = build;
    let object = directory.join("runtime.obj");
    let dependencies = directory.join("runtime.d");
    let mut command = toolchain.command();
    command
        .args(["-c", "-x", "c", &source.0])
        .args(compile_flags(options));
    command.arg("-o").arg(&object);
    if options.reproducible {
        command.env("SOURCE_DATE_EPOCH", "0");
    }

    let plan = command.args(["-v", "-###"]).output()?;

    if !plan.status.success() {
        return Err(io::Error::other("runtime compiler plan unavailable"));
    }

    let plan_text = String::from_utf8_lossy(&plan.stderr);

    if plan_text.contains("Configuration file:") {
        return Err(io::Error::other(
            "runtime compiler configuration file unsupported",
        ));
    }

    let mut include_roots = include_directories(&plan_text)
        .ok_or_else(|| io::Error::other("runtime compiler search paths unsupported"))?;
    let compiler = resolved_compiler(toolchain.compiler())?;

    if let Some(directory) = compiler.parent() {
        // Adjacent Clang .cfg files can change without changing any C dependency.
        include_roots.push(directory.to_path_buf());
    }
    // Compare directory state before and after compilation to reject mixed snapshots.
    let before = directory_stamps(&include_roots)?;
    let mut command = toolchain.command();
    command
        .args(["-c", "-x", "c", &source.0])
        .args(compile_flags(options));
    command
        .arg("-MD")
        .arg("-MF")
        .arg(&dependencies)
        .args(["-MT", "x-runtime"]);
    command.arg("-o").arg(&object);
    if options.reproducible {
        command.env("SOURCE_DATE_EPOCH", "0");
    }

    let compilation_start = SystemTime::now();

    if !command.output()?.status.success() {
        return Err(io::Error::other("runtime object compilation failed"));
    }

    let source_path = fs::canonicalize(&source.0)?;
    let headers = dependency_paths(&fs::read_to_string(&dependencies)?)
        .ok_or_else(|| io::Error::other("runtime dependency format unsupported"))?
        .into_iter()
        .map(fs::canonicalize)
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|path| path != &source_path)
        .map(|path| {
            let before = stamp(&path)?;
            let identity = NativeToolchain::file_identity(&path);

            if fs::metadata(&path)?.modified()? > compilation_start {
                return Err(io::Error::other(
                    "runtime header changed during compilation",
                ));
            }

            let bytes = fs::read(&path)?;

            if before != stamp(&path)? || identity != NativeToolchain::file_identity(&path) {
                return Err(io::Error::other(
                    "runtime header changed during fingerprinting",
                ));
            }

            Ok(FileDigest {
                path,
                digest: digest(&bytes),
                identity,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let after = directory_stamps(&include_roots)?;

    if before != after
        || headers.is_empty()
        || cache_key(toolchain, options, &x_runtime::get_runtime_c_blob())? != *key
    {
        return Err(io::Error::other(
            "runtime dependencies changed during compilation",
        ));
    }

    fs::remove_file(dependencies)?;

    Ok(Manifest {
        format: FORMAT,
        key: key.to_string(),
        headers,
        directories: after,
        object_digest: digest(&fs::read(object)?),
    })
}
