//! ChangeTime detects edits hidden by a restored mtime.
use std::slice;

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::{
    ffi::OsStringExt,
    fs::{MetadataExt, OpenOptionsExt},
    io::AsRawHandle,
};
use std::path::{Path, PathBuf};

use crate::windows_file::{
    FileInformation, GetFileInformationByHandle, GetFileInformationByHandleEx,
};

fn number(bytes: &[u8], offset: usize) -> io::Result<u64> {
    let bytes = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| io::Error::other("truncated directory information"))?;

    Ok(u64::from_le_bytes(
        bytes.try_into().map_err(io::Error::other)?,
    ))
}

fn word(bytes: &[u8], offset: usize) -> io::Result<u32> {
    let bytes = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| io::Error::other("truncated directory information"))?;

    Ok(u32::from_le_bytes(
        bytes.try_into().map_err(io::Error::other)?,
    ))
}

fn parse_entries(
    bytes: &[u8],
    path: &Path,
    volume: u32,
    entries: &mut Vec<(PathBuf, [u64; 5], u32)>,
) -> io::Result<()> {
    let mut offset = 0;

    loop {
        let part = bytes
            .get(offset..)
            .ok_or_else(|| io::Error::other("invalid directory offset"))?;
        let next = word(part, 0)? as usize;
        let length = word(part, 60)? as usize;

        if length == 0 || !length.is_multiple_of(2) {
            return Err(io::Error::other("invalid directory name"));
        }

        let name = part
            .get(104..104 + length)
            .ok_or_else(|| io::Error::other("truncated directory name"))?;
        let wide: Vec<u16> = name
            .chunks_exact(2)
            .map(|second_value| {
                <[u8; 2]>::try_from(second_value)
                    .map(u16::from_le_bytes)
                    .map_err(io::Error::other)
            })
            .collect::<io::Result<_>>()?;
        let name = OsString::from_wide(&wide);

        if name != "." && name != ".." {
            let attributes = word(part, 56)?;
            let changed = number(part, 32)?;
            let identifier = number(part, 96)?;

            if attributes & 0x400 != 0 || changed == 0 || identifier == 0 {
                return Err(io::Error::other("unstable or reparse package entry"));
            }

            entries.push((
                path.join(name),
                [
                    u64::from(volume),
                    identifier,
                    number(part, 40)?,
                    number(part, 24)?,
                    changed,
                ],
                attributes,
            ));
        }

        if next == 0 {
            break;
        }

        if next < 104 + length
            || !next.is_multiple_of(8)
            || offset
                .checked_add(next)
                .is_none_or(|end| end >= bytes.len())
        {
            return Err(io::Error::other("invalid directory offset"));
        }

        offset += next;
    }

    Ok(())
}

fn directory_entries(path: &Path) -> io::Result<Vec<(PathBuf, [u64; 5], u32)>> {
    let directory = OpenOptions::new()
        .access_mode(1)
        .share_mode(7)
        .custom_flags(0x02000000)
        .open(path)?;
    let mut info = FileInformation::default();
    // SAFETY: the live directory handle and documented C buffer layout are valid.
    if unsafe { GetFileInformationByHandle(directory.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut buffer = vec![0u64; 8192];
    let mut class = 11;
    let mut entries = Vec::new();

    loop {
        // SAFETY: the aligned buffer remains allocated for the full API call.
        if unsafe {
            GetFileInformationByHandleEx(
                directory.as_raw_handle(),
                class,
                buffer.as_mut_ptr().cast(),
                (buffer.len() * 8) as u32,
            )
        } == 0
        {
            let error = io::Error::last_os_error();

            if error.raw_os_error() == Some(18) {
                break;
            }

            return Err(error);
        }

        class = 10;
        // SAFETY: this byte view covers exactly the allocated u64 buffer.
        let bytes =
            unsafe { slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), buffer.len() * 8) };
        parse_entries(bytes, path, info.volume, &mut entries)?;
    }

    Ok(entries)
}

fn package_paths(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut pending = ["include", "lib", "libtcc"]
        .map(|name| root.join(name))
        .to_vec();
    let mut files = vec![root.join("libtcc.dll")];

    while let Some(path) = pending.pop() {
        if path.symlink_metadata()?.file_attributes() & 0x400 != 0 {
            return Err(io::Error::other("reparse package root"));
        }

        for (path, _, attributes) in directory_entries(&path)? {
            if attributes & 0x10 != 0 {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }

    files.sort();

    Ok(files)
}

pub(super) fn guard_package(root: &Path) -> io::Result<(Vec<File>, Vec<u8>)> {
    let files = package_paths(root)?;
    let guards = files
        .iter()
        .map(|file| {
            // READ_DATA makes Windows enforce sharing; deny writes and deletion while linking.
            OpenOptions::new().access_mode(1).share_mode(1).open(file)
        })
        .collect::<io::Result<Vec<_>>>()?;
    let (after, signature) = package_snapshot(root)?;

    if after != files {
        return Err(io::Error::other(
            "TinyCC package entries changed during guard acquisition",
        ));
    }

    Ok((guards, signature))
}

fn package_snapshot(root: &Path) -> io::Result<(Vec<PathBuf>, Vec<u8>)> {
    let mut pending = ["include", "lib", "libtcc"]
        .map(|name| root.join(name))
        .to_vec();
    let mut files = Vec::new();
    let mut paths = vec![root.join("libtcc.dll")];

    while let Some(path) = pending.pop() {
        if path.symlink_metadata()?.file_attributes() & 0x400 != 0 {
            return Err(io::Error::other("reparse package root"));
        }

        for (path, stamp, attributes) in directory_entries(&path)? {
            if attributes & 0x10 != 0 {
                pending.push(path.clone())
            } else {
                paths.push(path.clone());
            }

            files.push((path, stamp));
        }
    }

    files.sort_by(|first_value, second_value| first_value.0.cmp(&second_value.0));
    let mut bytes = Vec::new();

    for (path, stamp) in files {
        let name = path
            .strip_prefix(root)
            .map_err(io::Error::other)?
            .as_os_str()
            .as_encoded_bytes();
        bytes.extend_from_slice(&(name.len() as u64).to_le_bytes());
        bytes.extend_from_slice(name);

        for item in stamp {
            bytes.extend_from_slice(&item.to_le_bytes())
        }
    }

    paths.sort();

    Ok((paths, bytes))
}

pub(super) fn package_signature(root: &Path) -> io::Result<Vec<u8>> {
    package_snapshot(root).map(|(_, signature)| signature)
}
