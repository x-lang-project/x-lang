use super::*;

pub(super) fn validate_build(path: &Path, build: &toml::Value) -> Result<(), CompilationFailure> {
    let build = build
        .as_table()
        .ok_or_else(|| ProjectError::build_table_required(path.display()))?;
    reject_unknown_keys(
        path,
        "build",
        build,
        &["freestanding", "no_borrow_check", "optimization"],
    )?;
    for key in ["freestanding", "no_borrow_check"] {
        if build
            .get(key)
            .is_some_and(|value| value.as_bool().is_none())
        {
            return Err(ProjectError::build_boolean_required(path.display(), key));
        }
    }

    if build.get("optimization").is_some_and(|value| {
        value
            .as_integer()
            .is_none_or(|level| !(0..=i64::from(NativeOptimization::MAX)).contains(&level))
    }) {
        return Err(ProjectError::build_optimization_invalid(path.display()));
    }

    if !build
        .get("freestanding")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
    {
        return Err(ProjectError::freestanding_required(path.display()));
    }

    Ok(())
}

pub(super) fn validate_os(path: &Path, os: &toml::Value) -> Result<(), CompilationFailure> {
    let os = os
        .as_table()
        .ok_or_else(|| ProjectError::os_table_required(path.display()))?;
    reject_unknown_keys(
        path,
        "os",
        os,
        &[
            "arch",
            "target",
            "boot_asm",
            "crt0",
            "support",
            "linker_script",
            "grub_cfg",
            "kernel",
            "iso",
            "iso_dir",
            "nasm",
            "cc",
            "cxx",
            "ld",
            "grub_mkrescue",
        ],
    )?;
    for key in ["boot_asm", "crt0", "linker_script", "kernel"] {
        let value = required_manifest_string(path, "os", os, key)?;

        if !is_portable_package_relative_path(value) {
            return Err(ProjectError::os_path_escapes_package(
                path.display(),
                key,
                value,
            ));
        }
    }

    for key in ["grub_cfg", "iso", "iso_dir"] {
        if let Some(value) = os.get(key) {
            let value = value
                .as_str()
                .ok_or_else(|| ProjectError::os_string_required(path.display(), key))?;

            if !is_portable_package_relative_path(value) {
                return Err(ProjectError::os_path_escapes_package(
                    path.display(),
                    key,
                    value,
                ));
            }
        }
    }

    validate_string_array(path, "os.support", os.get("support"), true)?;
    for support in manifest_string_array(os, "support") {
        if !is_portable_package_relative_path(&support) {
            return Err(ProjectError::os_support_escapes_package(
                path.display(),
                support,
            ));
        }
    }

    for key in ["arch", "target", "nasm", "cc", "cxx", "ld", "grub_mkrescue"] {
        if os.get(key).is_some_and(|value| value.as_str().is_none()) {
            return Err(ProjectError::os_string_required(path.display(), key));
        }
    }

    Ok(())
}

pub(super) fn validate_qemu(path: &Path, qemu: &toml::Value) -> Result<(), CompilationFailure> {
    let qemu = qemu
        .as_table()
        .ok_or_else(|| ProjectError::emulator_table_required(path.display()))?;
    reject_unknown_keys(path, "qemu", qemu, &["command", "memory"])?;
    for key in ["command", "memory"] {
        if qemu.get(key).is_some_and(|value| value.as_str().is_none()) {
            return Err(ProjectError::emulator_string_required(path.display(), key));
        }
    }

    Ok(())
}
