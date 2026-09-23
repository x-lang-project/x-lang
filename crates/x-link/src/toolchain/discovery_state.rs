//! SDK selection also depends on registry values, COM instances and VCToolsVersion.
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use std::{collections::BTreeMap, env, fs, io, ptr};

#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS, FILETIME,
};
#[cfg(windows)]
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    REG_EXPAND_SZ, REG_SZ, RegCloseKey, RegEnumKeyExW, RegEnumValueW, RegOpenKeyExW,
    RegQueryInfoKeyW,
};

#[cfg(windows)]
use super::NativeToolchain;
#[cfg(windows)]
use registry::*;

pub(super) fn fingerprint() -> Option<Vec<u8>> {
    #[cfg(windows)]
    {
        snapshot().ok()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
fn snapshot() -> io::Result<Vec<u8>> {
    let mut state = BTreeMap::<String, Vec<u8>>::new();

    for view in [KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
        for key in [
            r"SOFTWARE\Microsoft\Microsoft SDKs\Windows",
            r"SOFTWARE\Microsoft\Windows Kits\Installed Roots",
            r"SOFTWARE\Microsoft\VisualStudio",
            r"SOFTWARE\Microsoft\VCExpress",
        ] {
            let stamps = registry_state(HKEY_LOCAL_MACHINE, key, view)?;
            state.insert(format!("HKLM:{view}:{key}"), stamps);
        }

        let key = r"CLSID\{177F0C4A-1CD3-4DE7-A32C-71DBBB9FA36D}";
        state.insert(
            format!("HKCR:{view}:{key}"),
            registry_state(HKEY_CLASSES_ROOT, key, view)?,
        );
    }

    let program_data =
        env::var_os("ProgramData").ok_or_else(|| io::Error::other("ProgramData unavailable"))?;
    installation_state(
        &Path::new(&program_data).join("Microsoft/VisualStudio/Packages/_Instances"),
        &mut state,
    )?;
    // LLVM probes both cl.exe and link.exe; capture changes to those PATH files.
    if let Some(path) = env::var_os("PATH") {
        for directory in env::split_paths(&path) {
            for name in ["cl.exe", "link.exe"] {
                optional_file(&directory.join(name), &mut state)?;
            }
        }
    }

    serde_json::to_vec(&state).map_err(io::Error::other)
}

#[cfg(windows)]
struct RegistryKey(HKEY);
#[cfg(windows)]
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

#[cfg(windows)]
fn optional_file(path: &Path, state: &mut BTreeMap<String, Vec<u8>>) -> io::Result<()> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {
            state.insert(
                format!("file:{}", path.display()),
                NativeToolchain::file_identity(path)
                    .ok_or_else(|| io::Error::other("discovery file identity unavailable"))?,
            );
        }
        Ok(_) => {
            return Err(io::Error::other("discovery file is not a regular file"));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            state.insert(format!("absent:{}", path.display()), Vec::new());
        }
        Err(error) => {
            return Err(error);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn installation_state(root: &Path, state: &mut BTreeMap<String, Vec<u8>>) -> io::Result<()> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            state.insert(format!("absent-instances:{}", root.display()), Vec::new());

            return Ok(());
        }
        Err(error) => return Err(error),
    };

    state.insert(format!("instances:{}", root.display()), Vec::new());
    for entry in entries {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            return Err(io::Error::other("unsupported VS instance entry"));
        }

        let instance = entry.path();
        state.insert(format!("instance:{}", instance.display()), Vec::new());
        for file in fs::read_dir(&instance)? {
            let file = file?;

            if !file.file_type()?.is_file() {
                return Err(io::Error::other("unsupported VS instance metadata"));
            }

            optional_file(&file.path(), state)?;
        }

        let path = instance.join("state.json");
        let before = NativeToolchain::file_identity(&path)
            .ok_or_else(|| io::Error::other("VS state identity unavailable"))?;
        let contents: serde_json::Value =
            serde_json::from_slice(&fs::read(&path)?).map_err(io::Error::other)?;

        if NativeToolchain::file_identity(&path).as_ref() != Some(&before) {
            return Err(io::Error::other("VS state changed during discovery"));
        }

        for name in ["installationPath", "resolvedInstallationPath"] {
            if let Some(path) = contents.get(name).and_then(serde_json::Value::as_str) {
                optional_file(
                    &Path::new(path)
                        .join("VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt"),
                    state,
                )?;
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
mod registry;
