use super::*;

pub(super) fn stamp(path: &Path) -> io::Result<Stamp> {
    let metadata = fs::metadata(path)?;
    let identity = NativeToolchain::file_identity(path);

    if cfg!(windows) && identity.is_none() {
        return Err(io::Error::other(
            "runtime cache requires file journal identity",
        ));
    }

    Ok(Stamp {
        path: path.to_path_buf(),
        modified_nanoseconds: metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos(),
        len: metadata.len(),
        directory: metadata.is_dir(),
        identity,
    })
}
