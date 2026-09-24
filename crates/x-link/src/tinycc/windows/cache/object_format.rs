use super::*;

pub(crate) fn elf_object(bytes: &[u8]) -> bool {
    bytes.get(..6) == Some(b"\x7fELF\x02\x01") && bytes.get(16..20) == Some(&[1, 0, 62, 0])
}

pub(crate) fn zero_pe_timestamp(path: &Path) -> Result<bool, CompilationFailure> {
    let bytes = fs::read(path).map_err(failure)?;
    let offset = bytes
        .get(60..64)
        .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
        .map(u32::from_le_bytes)
        .and_then(|value| usize::try_from(value).ok());

    Ok(offset.is_some_and(|offset| {
        offset.checked_add(4).and_then(|end| bytes.get(offset..end)) == Some(b"PE\0\0")
            && offset
                .checked_add(8)
                .and_then(|start| start.checked_add(4).and_then(|end| bytes.get(start..end)))
                == Some(&[0, 0, 0, 0])
    }))
}
