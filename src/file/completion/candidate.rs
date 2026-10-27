use std::ffi::OsString;
use std::fs::{DirEntry, FileType};
use std::io;
use std::path::{Path, PathBuf};

/// Lossless observed identity and independent terminal-safe presentation. A
/// symlink is a leaf here, even when its unobserved destination is a directory.
#[derive(Debug)]
pub struct Candidate {
    name: OsString,
    path: PathBuf,
    kind: FileType,
    label: String,
}

impl Candidate {
    pub(super) fn observe(entry: DirEntry, name: OsString) -> io::Result<Self> {
        let kind = entry.file_type()?;
        let mut label = label(&name);
        if kind.is_dir() {
            label.push('/');
        }
        Ok(Self {
            name,
            path: entry.path(),
            kind,
            label,
        })
    }

    pub fn name(&self) -> Option<&str> {
        self.name.to_str()
    }

    /// Return the original path, never a display label or lossy replacement.
    /// Unsupported encodings remain visible but cannot supply an opening path.
    pub fn path(&self) -> io::Result<&Path> {
        if self.name().is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported non-UTF-8 filename",
            ));
        }
        Ok(&self.path)
    }

    /// Safe full label; the common display owner applies whole-grapheme clipping.
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn is_directory(&self) -> bool {
        self.kind.is_dir()
    }

    pub fn is_symlink(&self) -> bool {
        self.kind.is_symlink()
    }

    pub(super) fn matches(&self, prefix: &str) -> bool {
        self.name.as_encoded_bytes().starts_with(prefix.as_bytes())
    }

    pub(super) fn order_key(&self) -> (bool, &[u8]) {
        (!self.is_directory(), self.name.as_encoded_bytes())
    }
}

fn label(name: &std::ffi::OsStr) -> String {
    let Some(name) = name.to_str() else {
        return format!("{name:?} [unsupported non-UTF-8 filename]");
    };
    name.chars()
        .map(|character| {
            if character.is_control() {
                character.escape_default().to_string()
            } else {
                character.to_string()
            }
        })
        .collect()
}
