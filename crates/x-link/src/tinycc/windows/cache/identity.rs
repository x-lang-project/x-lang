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

fn word(bytes: &[u8], offset: usize) -> io::Result<u32> {
    let bytes = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| io::Error::other("truncated directory information"))?;

    Ok(u32::from_le_bytes(
        bytes.try_into().map_err(io::Error::other)?,
    ))
}
