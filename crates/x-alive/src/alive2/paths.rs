use super::*;

pub(super) fn wsl_command(distro: &str) -> Command {
    let mut command = Command::new("wsl");
    command.args(["-d", distro, "--"]);
    if let Ok(library_path) = env::var("X_ALIVE_TV_WSL_LIBRARY_PATH")
        && !library_path.trim().is_empty()
    {
        command.args(["env", &format!("LD_LIBRARY_PATH={library_path}")]);
    }

    command
}

pub(super) fn path_to_wsl(distro: &str, path: &Path) -> Result<String, alive_messages::AliveError> {
    let absolute = path
        .canonicalize()
        .map_err(|error| alive_messages::path_resolve(path.display(), error))?;

    #[cfg(windows)]
    {
        if let Some(converted) = windows_drive_path_to_wsl(&absolute) {
            return Ok(converted);
        }
    }

    let output = Command::new("wsl")
        .args(["-d", distro, "--", "wslpath", "-a"])
        .arg(&absolute)
        .output()
        .map_err(|error| alive_messages::wsl_path_launch(absolute.display(), error))?;

    if !output.status.success() {
        return Err(alive_messages::wsl_path_failure(
            absolute.display(),
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(windows)]
pub(super) fn windows_drive_path_to_wsl(path: &Path) -> Option<String> {
    let raw = path.to_string_lossy();
    let raw = raw
        .strip_prefix(r"\\?\")
        .or_else(|| raw.strip_prefix(r"//?/"))
        .unwrap_or(&raw);
    let bytes = raw.as_bytes();
    let [drive, b':', separator, ..] = bytes else {
        return None;
    };

    if !matches!(separator, b'\\' | b'/') {
        return None;
    }

    let drive = (*drive as char).to_ascii_lowercase();

    if !drive.is_ascii_alphabetic() {
        return None;
    }

    let rest = raw[2..].replace('\\', "/");

    Some(format!("/mnt/{drive}{rest}"))
}
