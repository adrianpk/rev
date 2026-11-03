//! Bounded anonymous stdin staging for the shared file-input process owner.
use rustix::fs::{MemfdFlags, SealFlags, fcntl_add_seals, memfd_create};
use std::{
    fs::File,
    io::{Seek, Write},
};

pub(super) fn stage(input: Option<Vec<u8>>) -> Result<Option<File>, String> {
    let Some(bytes) = input else {
        return Ok(None);
    };
    let descriptor = memfd_create(
        "shell-input",
        MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING,
    )
    .map_err(|e| format!("Create anonymous shell input: {e}"))?;
    let mut file = File::from(descriptor);
    file.write_all(&bytes)
        .map_err(|e| format!("Stage shell input: {e}"))?;
    file.rewind()
        .map_err(|e| format!("Rewind shell input: {e}"))?;
    fcntl_add_seals(
        &file,
        SealFlags::WRITE | SealFlags::GROW | SealFlags::SHRINK | SealFlags::SEAL,
    )
    .map_err(|e| format!("Seal shell input: {e}"))?;
    Ok(Some(file))
}
