use super::*;
use x_diagnostics::CompilationFailure;
use x_diagnostics::link_messages;

pub(super) fn valid_object_inputs(
    object_files: &[String],
) -> Result<Vec<String>, CompilationFailure> {
    object_files
        .iter()
        .map(|object| {
            let path = Path::new(object);
            let metadata = fs::metadata(path)
                .map_err(|error| link_messages::object_read(path.display(), error))?;

            if !metadata.is_file() {
                return Err(link_messages::object_not_file(path.display()));
            }

            if metadata.len() == 0 {
                return Err(link_messages::object_empty(path.display()));
            }

            fs::canonicalize(path)
                .map(|path| path.to_string_lossy().into_owned())
                .map_err(|error| link_messages::object_resolve(path.display(), error))
        })
        .collect()
}
