use super::*;
use x_diagnostics::CompilationFailure;
use x_diagnostics::DriverError;

pub fn is_x_source_path(path: &str) -> bool {
    let path = Path::new(path);

    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("x"))
    {
        return true;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| strip_ascii_suffix(name, ".fail").is_some())
}

pub fn x_output_stem(path: &str) -> String {
    [".x.fail", ".fail", ".x"]
        .into_iter()
        .find_map(|suffix| strip_ascii_suffix(path, suffix))
        .unwrap_or(path)
        .to_owned()
}

fn strip_ascii_suffix<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let start = value.len().checked_sub(suffix.len())?;

    if value.get(start..)?.eq_ignore_ascii_case(suffix) {
        value.get(..start)
    } else {
        None
    }
}

pub fn collect_x_files_recursive(
    path: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), CompilationFailure> {
    if !path.exists() {
        return Err(DriverError::project_input_not_found(path.display()));
    }

    if path.is_file() {
        if path.extension().and_then(|text| text.to_str()) == Some("x") {
            output.push(path.to_path_buf());
        }

        return Ok(());
    }

    let mut directories = vec![path.to_path_buf()];

    while let Some(directory) = directories.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| DriverError::project_directory_read(directory.display(), error))?;

        for entry in entries {
            let entry = entry.map_err(|error| {
                DriverError::project_directory_entry_read(directory.display(), error)
            })?;
            let child = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| DriverError::project_input_inspection(child.display(), error))?;

            // Do not follow nested symbolic links: a link may point back to an ancestor.
            if file_type.is_dir() {
                directories.push(child);
            } else if file_type.is_file()
                && child.extension().and_then(|extension| extension.to_str()) == Some("x")
            {
                output.push(child);
            }
        }
    }

    Ok(())
}

pub fn collect_x_files_from_inputs(paths: &[String]) -> Result<Vec<PathBuf>, CompilationFailure> {
    let mut files = Vec::new();

    if paths.is_empty() {
        collect_x_files_recursive(Path::new("."), &mut files)?;
    } else {
        for path in paths {
            collect_x_files_recursive(Path::new(path), &mut files)?;
        }
    }

    files.sort_by(|first_value, second_value| {
        first_value
            .to_string_lossy()
            .cmp(&second_value.to_string_lossy())
    });
    files.dedup_by(|first_value, second_value| first_value == second_value);

    Ok(files)
}
