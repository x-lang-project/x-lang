use std::env;
use std::process;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use inkwell::support;

use x_diagnostics::CliDiagnostic;
use x_diagnostics::CompilationFailure;
use x_project::manifest::{LANGUAGE_EDITION, RUNTIME_ABI_VERSION};
use x_utils::tools::LLVM_VERSION;

pub(super) fn print_version() {
    println!("x {}", env!("CARGO_PKG_VERSION"));
    println!("language edition: {LANGUAGE_EDITION}");
    let (major, minor, patch) = support::get_llvm_version();
    println!("LLVM: {major}.{minor}.{patch}");
    println!("host target: {}", env!("X_HOST_TARGET"));
    println!("runtime ABI: {RUNTIME_ABI_VERSION}");
}

pub(super) fn doctor() -> Result<(), CompilationFailure> {
    print_version();
    let mut missing = Vec::new();
    let clang = check_clang(&mut missing)?;

    if let Some(clang) = clang {
        match verify_openmp(&clang) {
            Ok(()) => println!("ok: OpenMP (compile, link, and runtime probe)"),
            Err(error) => {
                println!("missing: OpenMP (compile, link, and runtime probe; {error})");
                missing.push("OpenMP");
            }
        }
    }

    check_llvm_config(&mut missing);

    if missing.is_empty() {
        println!("doctor: toolchain is ready");

        Ok(())
    } else {
        Err(CliDiagnostic::doctor_tools_missing(missing.join(", ")))
    }
}

fn check_clang(missing: &mut Vec<&'static str>) -> Result<Option<PathBuf>, CompilationFailure> {
    let clang = match x_link::NativeToolchain::discover() {
        Ok(toolchain) => {
            let clang = toolchain.compiler();
            let first_line = toolchain
                .version()
                .lines()
                .next()
                .unwrap_or("unknown version");
            println!(
                "ok: clang (native linking; {}; {first_line})",
                clang.display()
            );

            Some(clang.to_path_buf())
        }
        Err(error) => {
            println!("missing: clang (native linking; {error})");
            missing.push("clang");

            None
        }
    };

    Ok(clang)
}

fn check_llvm_config(missing: &mut Vec<&'static str>) {
    let llvm_config = x_utils::tools::llvm_program("llvm-config");

    match Command::new(&llvm_config).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let version = version.trim();

            if version == LLVM_VERSION {
                println!(
                    "ok: llvm-config (LLVM installation inspection; {}; {version})",
                    llvm_config.display()
                );
            } else if cfg!(windows) {
                println!("ignored: llvm-config {version}; using LLVM-C {LLVM_VERSION}");
            } else {
                println!("mismatch: llvm-config must be LLVM {LLVM_VERSION} (found {version})");
                missing.push("llvm-config");
            }
        }
        _ if cfg!(windows) => {
            println!("ok: LLVM-C dynamic ABI (official Windows LLVM does not ship llvm-config)")
        }
        _ => {
            println!(
                "missing: llvm-config (LLVM installation inspection; {})",
                llvm_config.display()
            );
            missing.push("llvm-config");
        }
    }
}

fn verify_openmp(clang: &Path) -> Result<(), CompilationFailure> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(CliDiagnostic::clock_before_epoch)?
        .as_nanos();
    let probe_directory =
        env::temp_dir().join(format!("x-doctor-openmp-{}-{unique}", process::id()));
    fs::create_dir(&probe_directory)
        .map_err(|error| CliDiagnostic::probe_directory_create(probe_directory.display(), error))?;
    let executable =
        x_utils::artifacts::ArtifactPaths::new(probe_directory.join("openmp-probe")).executable();

    let probe_result = run_openmp_probe(clang, &executable);

    let cleanup_result = fs::remove_dir_all(&probe_directory)
        .map_err(|error| CliDiagnostic::probe_directory_remove(probe_directory.display(), error));

    probe_result.and(cleanup_result)
}

fn run_openmp_probe(clang: &Path, executable: &Path) -> Result<(), CompilationFailure> {
    let mut child = Command::new(clang)
        .args(["-x", "c", "-", "-fopenmp", "-o"])
        .arg(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| CliDiagnostic::openmp_compiler_start(clang.display(), error))?;
    let source = b"#include <omp.h>\nint main(void) { return omp_get_max_threads() < 1; }\n";
    // Take ownership so stdin closes before waiting, including write failures.
    let input_result = child
        .stdin
        .take()
        .ok_or_else(CliDiagnostic::openmp_input_missing)
        .and_then(|mut input| {
            input
                .write_all(source)
                .map_err(CliDiagnostic::openmp_input_write)
        });

    if let Err(error) = input_result {
        // Dropping Child does not reap it; preserve the input error while collecting it.
        let _ = child.kill();
        let _ = child.wait();

        return Err(error);
    }

    let output = child
        .wait_with_output()
        .map_err(CliDiagnostic::openmp_compiler_wait)?;

    if !output.status.success() {
        return Err(CliDiagnostic::openmp_compile(
            String::from_utf8_lossy(&output.stderr).trim(),
        ));
    }

    let status = Command::new(executable)
        .status()
        .map_err(CliDiagnostic::openmp_probe_run)?;

    if !status.success() {
        return Err(CliDiagnostic::openmp_probe_exit(status));
    }

    Ok(())
}
