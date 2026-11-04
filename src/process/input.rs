//! Bounded anonymous stdin staging for the shared file-input process owner.
use rustix::fs::{MemfdFlags, SealFlags, fcntl_add_seals, memfd_create};
use std::{
    fs::File,
    io::{Seek, Write},
};

pub(crate) fn stage(input: Option<&[u8]>) -> Result<Option<File>, String> {
    let Some(bytes) = input else {
        return Ok(None);
    };
    let descriptor = memfd_create(
        "tool-input",
        MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING,
    )
    .map_err(|e| format!("Create anonymous tool input: {e}"))?;
    let mut file = File::from(descriptor);
    file.write_all(bytes)
        .map_err(|e| format!("Stage tool input: {e}"))?;
    file.rewind()
        .map_err(|e| format!("Rewind tool input: {e}"))?;
    fcntl_add_seals(
        &file,
        SealFlags::WRITE | SealFlags::GROW | SealFlags::SHRINK | SealFlags::SEAL,
    )
    .map_err(|e| format!("Seal tool input: {e}"))?;
    Ok(Some(file))
}
