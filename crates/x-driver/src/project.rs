use super::*;
use x_diagnostics::CompilationFailure;
use x_diagnostics::DriverError;
use x_project::manifest::{LANGUAGE_EDITION, PackageKind, RUNTIME_ABI_VERSION};
use x_utils::project::MANIFEST_FILE_NAME;

pub fn scaffold_new_project(project_directory: &Path) -> Result<PathBuf, CompilationFailure> {
    if project_directory.exists() {
        return Err(DriverError::project_destination_exists(
            project_directory.display(),
        ));
    }

    let name = project_directory
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        })
        .ok_or_else(|| DriverError::project_name_invalid(project_directory.display()))?;

    if let Some(parent) = project_directory
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| DriverError::project_parent_directory(parent.display(), error))?;
    }

    // Claim the destination exclusively before writing files or enabling rollback.
    fs::create_dir(project_directory).map_err(|error| {
        DriverError::project_destination_create(project_directory.display(), error)
    })?;

    let result = write_project_files(project_directory, name);

    if let Err(error) = result {
        let cleanup = fs::remove_dir_all(project_directory);

        return match cleanup {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(DriverError::project_rollback(
                project_directory.display(),
                error,
                cleanup_error,
            )),
        };
    }

    Ok(project_directory.to_path_buf())
}

fn write_project_files(project_directory: &Path, name: &str) -> Result<(), CompilationFailure> {
    let source_directory = project_directory.join("src");
    fs::create_dir(&source_directory).map_err(|error| {
        DriverError::project_source_directory(source_directory.display(), error)
    })?;

    let kind = PackageKind::Binary.as_str();
    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"{LANGUAGE_EDITION}\"\nentry = \"src/main.x\"\nkind = \"{kind}\"\n\n[toolchain]\nruntime-abi = {RUNTIME_ABI_VERSION}\n"
    );
    write_file(&project_directory.join(MANIFEST_FILE_NAME), &manifest)?;

    let source = "import std::io;\n\nfn main() -> i32 {\n    io::println(\"Hello from X!\");\n    return 0;\n}\n";
    write_file(&project_directory.join("src/main.x"), source)?;

    let readme = format!(
        "# {name}\n\nCheck, build, and run the project from this directory:\n\n```bash\nx check .\nx build .\nx run .\n```\n"
    );
    write_file(&project_directory.join("README.md"), &readme)?;

    let gitignore = format!("/{name}\n/{name}.exe\n*.ll\n*.o\n");

    write_file(&project_directory.join(".gitignore"), &gitignore)
}

fn write_file(path: &Path, contents: &str) -> Result<(), CompilationFailure> {
    fs::write(path, contents)
        .map_err(|error| DriverError::project_file_write(path.display(), error))
}
