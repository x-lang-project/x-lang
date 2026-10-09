use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;

use x_diagnostics::CliDiagnostic;
use x_diagnostics::CompilationFailure;
use x_project::source;

#[derive(Clone, Copy)]
pub(super) struct CheriBuildOptions<'a> {
    pub clang: &'a Path,
    pub sysroot: &'a Path,
    pub optimisation_level: u8,
    pub save_ir: bool,
}

pub(super) fn build_cheri_object(
    input: &Path,
    output: &Path,
    options: CheriBuildOptions<'_>,
) -> Result<(), CompilationFailure> {
    let CheriBuildOptions {
        clang,
        sysroot,
        save_ir,
        ..
    } = options;

    if !clang.is_file() {
        return Err(CliDiagnostic::cheri_clang_missing(clang.display()));
    }

    if !sysroot.is_dir() {
        return Err(CliDiagnostic::cheri_sysroot_missing(sysroot.display()));
    }

    let source = source::read_utf8(input)?;
    let input_name = input
        .to_str()
        .ok_or_else(CliDiagnostic::cheri_input_path_encoding)?;
    let ir = x_compile::compile_source_to_cheri_riscv64_ir(&source, input_name)?;

    audit_purecap_ir(&ir)?;

    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| CliDiagnostic::cheri_output_directory(parent.display(), error))?;
    }

    let ir_path = ir_path(output, save_ir);
    fs::write(&ir_path, ir.as_bytes())
        .map_err(|error| CliDiagnostic::cheri_ir_write(ir_path.display(), error))?;

    let candidate = candidate_object_path(output);
    let result = compile_with_cheri_clang(options, &ir_path, &candidate)
        .and_then(|()| certify_cheri_elf(clang, &candidate))
        .and_then(|()| {
            fs::rename(&candidate, output)
                .map_err(|error| CliDiagnostic::cheri_object_install(output.display(), error))
        });

    if !save_ir {
        let _ = fs::remove_file(&ir_path);
    }

    if result.is_err() {
        let _ = fs::remove_file(&candidate);
    }

    result?;
    println!("CHERI-RISC-V pure-capability object: {}", output.display());
    if save_ir {
        println!("CHERI LLVM IR: {}", ir_path.display());
    }

    Ok(())
}

fn audit_purecap_ir(ir: &str) -> Result<(), CompilationFailure> {
    use x_utils::target::CheriPurecap;

    for required in [
        format!("target triple = \"{}\"", CheriPurecap::TRIPLE),
        format!("target datalayout = \"{}\"", CheriPurecap::DATA_LAYOUT),
        CheriPurecap::POINTER_IR_TYPE.to_string(),
    ] {
        if !ir.contains(&required) {
            return Err(CliDiagnostic::cheri_ir_marker_missing(&required));
        }
    }

    Ok(())
}

fn compile_with_cheri_clang(
    options: CheriBuildOptions<'_>,
    ir: &Path,
    output: &Path,
) -> Result<(), CompilationFailure> {
    let CheriBuildOptions {
        clang,
        sysroot,
        optimisation_level,
        ..
    } = options;

    let result = Command::new(clang)
        .args(x_utils::target::CheriPurecap::compiler_arguments())
        .arg(format!("--sysroot={}", sysroot.display()))
        .arg(x_utils::optimization::NativeOptimization::new(optimisation_level).argument())
        .arg("-Werror")
        .arg("-x")
        .arg("ir")
        .arg("-c")
        .arg(ir)
        .arg("-o")
        .arg(output)
        .output()
        .map_err(CliDiagnostic::cheri_clang_launch)?;

    if result.status.success() {
        return Ok(());
    }

    Err(CliDiagnostic::cheri_clang_rejected_ir(
        result.status,
        String::from_utf8_lossy(&result.stderr),
    ))
}

fn certify_cheri_elf(clang: &Path, output: &Path) -> Result<(), CompilationFailure> {
    let program = x_utils::tools::executable_name("llvm-readelf");
    let readelf = clang
        .parent()
        .map(|parent| parent.join(&program))
        .unwrap_or_else(|| PathBuf::from(program));

    if !readelf.is_file() {
        return Err(CliDiagnostic::cheri_readelf_missing(readelf.display()));
    }

    let result = Command::new(&readelf)
        .arg("-h")
        .arg(output)
        .output()
        .map_err(CliDiagnostic::cheri_object_inspection)?;
    let report = String::from_utf8_lossy(&result.stdout).to_ascii_lowercase();
    let has_cheri_abi = report.contains("cheriabi") || report.contains("cheri abi");
    let has_capability_mode = report.contains("capability mode") || report.contains("capmode");

    if !result.status.success() || !has_cheri_abi || !has_capability_mode {
        let _ = fs::remove_file(output);

        return Err(CliDiagnostic::cheri_object_certification(
            String::from_utf8_lossy(&result.stdout),
        ));
    }

    Ok(())
}

fn ir_path(output: &Path, save_ir: bool) -> PathBuf {
    if save_ir {
        return output.with_extension("cheri.ll");
    }

    env::temp_dir().join(format!(
        "x-cheri-{}-{}.ll",
        process::id(),
        output.file_name().unwrap_or_default().to_string_lossy()
    ))
}

fn candidate_object_path(output: &Path) -> PathBuf {
    let name = output.file_name().unwrap_or_default().to_string_lossy();

    output.with_file_name(format!(".{name}.x-cheri-candidate-{}", process::id()))
}
