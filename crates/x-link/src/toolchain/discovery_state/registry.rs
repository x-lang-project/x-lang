use super::*;

#[cfg(windows)]
pub(super) fn registry_state(hive: HKEY, name: &str, view: u32) -> io::Result<Vec<u8>> {
    let mut stamps = BTreeMap::new();
    let mut pending = vec![(name.to_owned(), 0usize)];

    while let Some((name, depth)) = pending.pop() {
        if name.ends_with("\\TreatAs") || name.ends_with("\\AutoTreatAs") {
            return Err(io::Error::other(
                "redirected Setup COM discovery unsupported",
            ));
        }

        if stamps.len() > 4096 {
            return Err(io::Error::other("discovery registry tree too large"));
        }

        let wide = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let mut raw = ptr::null_mut();
        // SAFETY: the UTF-16 name is terminated and the output key is valid.
        let status = unsafe { RegOpenKeyExW(hive, wide.as_ptr(), 0, KEY_READ | view, &mut raw) };

        if matches!(status, ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND) {
            stamps.insert(name, None);
            continue;
        }

        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status as i32));
        }

        let key = RegistryKey(raw);
        let mut time = FILETIME::default();
        let mut subkeys = 0;
        // SAFETY: null optional outputs are permitted; other outputs and the owned key stay live.
        let status = unsafe {
            RegQueryInfoKeyW(
                key.0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut subkeys,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut time,
            )
        };

        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status as i32));
        }

        stamps.insert(
            name.clone(),
            Some((
                (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime),
                registry_values(key.0)?,
            )),
        );
        // SDK/VS keys and Setup COM registration require two registry levels.
        if depth >= 2 {
            continue;
        }

        for index in 0..subkeys {
            let mut buffer = [0u16; 256];
            let mut length = buffer.len() as u32;
            // SAFETY: buffer/length point to writable storage of stated size.
            let status = unsafe {
                RegEnumKeyExW(
                    key.0,
                    index,
                    buffer.as_mut_ptr(),
                    &mut length,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };

            if status != ERROR_SUCCESS {
                return Err(io::Error::from_raw_os_error(status as i32));
            }

            let child = String::from_utf16(
                buffer
                    .get(..length as usize)
                    .ok_or_else(|| io::Error::other("invalid registry name size"))?,
            )
            .map_err(io::Error::other)?;
            pending.push((format!("{name}\\{child}"), depth + 1));
        }
    }

    serde_json::to_vec(&stamps).map_err(io::Error::other)
}

#[cfg(windows)]
pub(super) fn registry_values(key: HKEY) -> io::Result<BTreeMap<String, Vec<u8>>> {
    use sha2::{Digest, Sha256};
    use windows_sys::Win32::Foundation::ERROR_NO_MORE_ITEMS;

    let mut values = BTreeMap::new();

    for index in 0..4096 {
        let mut name = vec![0u16; 16384];
        let mut name_length = name.len() as u32;
        let mut data = vec![0u8; 65536];
        let mut length = data.len() as u32;
        let mut kind = 0;
        // SAFETY: each pointer has writable storage matching its supplied size.
        let status = unsafe {
            RegEnumValueW(
                key,
                index,
                name.as_mut_ptr(),
                &mut name_length,
                ptr::null_mut(),
                &mut kind,
                data.as_mut_ptr(),
                &mut length,
            )
        };

        if status == ERROR_NO_MORE_ITEMS {
            return Ok(values);
        }

        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status as i32));
        }

        let name = String::from_utf16(
            name.get(..name_length as usize)
                .ok_or_else(|| io::Error::other("registry value name too long"))?,
        )
        .map_err(io::Error::other)?;
        let data = data
            .get(..length as usize)
            .ok_or_else(|| io::Error::other("registry value too long"))?;
        let mut digest = kind.to_le_bytes().to_vec();
        digest.extend_from_slice(&Sha256::digest(data));
        values.insert(name.clone(), digest);
        // COM server implementations can change without changing registration.
        if name.is_empty() && matches!(kind, REG_SZ | REG_EXPAND_SZ) {
            let wide = data
                .as_chunks::<2>()
                .0
                .iter()
                .copied()
                .map(u16::from_le_bytes)
                .take_while(|word| *word != 0)
                .collect::<Vec<_>>();
            let text = String::from_utf16(&wide).map_err(io::Error::other)?;
            let path = Path::new(&text);

            if text.to_ascii_lowercase().ends_with(".dll")
                || text.to_ascii_lowercase().ends_with(".exe")
            {
                if !path.is_absolute() || text.contains('%') {
                    return Err(io::Error::other(
                        "COM discovery server path cannot be captured",
                    ));
                }

                let mut files = BTreeMap::new();
                optional_file(path, &mut files)?;
                values.insert(
                    "COM_SERVER_FILE_IDENTITY".into(),
                    serde_json::to_vec(&files).map_err(io::Error::other)?,
                );
            }
        }
    }

    Err(io::Error::other("too many discovery registry values"))
}
