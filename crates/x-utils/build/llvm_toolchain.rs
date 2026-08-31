use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::llvm_policy;

pub(super) fn llvm_prefix() -> Option<PathBuf> {
    println!(
        "cargo:rerun-if-env-changed={}",
        llvm_policy::LLVM_PREFIX_ENV
    );

    if let Some(prefix) = env::var_os(llvm_policy::LLVM_PREFIX_ENV) {
        let prefix = PathBuf::from(prefix);
        assert!(
            pinned_llvm_prefix(&prefix),
            "{} must select LLVM {} with its development libraries",
            llvm_policy::LLVM_PREFIX_ENV,
            llvm_policy::LLVM_VERSION
        );

        return Some(prefix);
    }

    for name in ["PATH", "ProgramFiles", "USERPROFILE"] {
        println!("cargo:rerun-if-env-changed={name}");
    }

    let mut candidates = Vec::new();

    if let Some(path) = env::var_os("PATH") {
        candidates.extend(env::split_paths(&path).filter_map(|bin| {
            bin.join("clang.exe")
                .is_file()
                .then(|| bin.parent().map(Path::to_path_buf))
                .flatten()
        }));
    }

    if let Some(program_files) = env::var_os("ProgramFiles") {
        candidates.push(PathBuf::from(program_files).join("LLVM"));
    }

    if let Some(profile) = env::var_os("USERPROFILE") {
        candidates.push(PathBuf::from(profile).join("scoop/apps/llvm/current"));
    }

    candidates
        .into_iter()
        .find(|prefix| pinned_llvm_prefix(prefix))
}

fn pinned_llvm_prefix(prefix: &Path) -> bool {
    prefix.join("lib/LLVM-C.lib").is_file()
        && prefix.join("bin/LLVM-C.dll").is_file()
        && Command::new(prefix.join("bin/clang.exe"))
            .arg("--version")
            .output()
            .is_ok_and(|output| {
                output.status.success()
                    && llvm_policy::clang_is_pinned(&String::from_utf8_lossy(&output.stdout))
            })
}
