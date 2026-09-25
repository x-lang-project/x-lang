//! ChangeTime detects edits hidden by a restored mtime.
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::{
    ffi::OsStringExt,
    fs::{MetadataExt, OpenOptionsExt},
    io::AsRawHandle,
};
use std::path::{Path, PathBuf};

use crate::windows_file::FileInformation;

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
