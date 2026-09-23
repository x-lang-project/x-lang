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
struct RegistryKey(HKEY);
#[cfg(windows)]
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}
