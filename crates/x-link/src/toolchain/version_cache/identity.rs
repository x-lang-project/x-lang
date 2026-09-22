use super::*;

#[cfg(windows)]
pub(super) fn file_usn(file: &File) -> io::Result<i64> {
    use std::os::windows::io::AsRawHandle;

    let mut output = [0u8; 4096];
    let mut returned = 0u32;
    // Request USN_RECORD_V2 to detect writes hidden by restored or coalesced timestamps.
    let versions = [2u16, 2u16];
    // SAFETY: handle and both buffers remain live for the synchronous call.
    let success = unsafe {
        DeviceIoControl(
            file.as_raw_handle(),
            0x000900eb,
            versions.as_ptr().cast(),
            4,
            output.as_mut_ptr().cast(),
            output.len() as u32,
            &mut returned,
            std::ptr::null_mut(),
        )
    };

    if success == 0 {
        return Err(io::Error::last_os_error());
    }

    if returned < 32 || output.get(4..6) != Some(&[2, 0]) {
        return Err(io::Error::other("unsupported file journal record"));
    }

    let bytes: [u8; 8] = output
        .get(24..32)
        .ok_or_else(|| io::Error::other("short file journal record"))?
        .try_into()
        .map_err(io::Error::other)?;

    Ok(i64::from_le_bytes(bytes))
}

#[cfg(windows)]
pub(super) fn resolved_file_path(file: &File) -> io::Result<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::{ffi::OsStringExt, io::AsRawHandle};

    // Resolve through the open handle so path replacement cannot change the observed file.
    let mut buffer = vec![0u16; 512];

    loop {
        // SAFETY: the handle is owned by file and buffer has the supplied length.
        let length = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle(),
                buffer.as_mut_ptr(),
                u32::try_from(buffer.len()).map_err(io::Error::other)?,
                0,
            )
        };

        if length == 0 {
            return Err(io::Error::last_os_error());
        }

        let length = length as usize;

        if length < buffer.len() {
            let path = buffer
                .get(..length)
                .ok_or_else(|| io::Error::other("invalid canonical file path length"))?;

            return Ok(PathBuf::from(OsString::from_wide(path)));
        }

        buffer.resize(length + 1, 0);
    }
}
