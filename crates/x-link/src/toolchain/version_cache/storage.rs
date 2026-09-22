use super::*;

pub(super) fn save_entry(root: &Path, path: &Path, entry: &Entry) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let temporary = root.join(format!(
        ".{}-{}.tmp",
        std::process::id(),
        NEXT_ENTRY.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        fs::write(
            &temporary,
            serde_json::to_vec(entry).map_err(io::Error::other)?,
        )?;

        fs::rename(&temporary, path)
    })();
    let _ = fs::remove_file(temporary);

    result
}
