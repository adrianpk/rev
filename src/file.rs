use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use ropey::{Rope, RopeSlice};

use crate::buffer::Buffer;

const BOM: &[u8] = b"\xef\xbb\xbf";

/// Validated private ingress, consumed when installed as a canonical buffer.
pub struct Loaded {
    text: Rope,
    bom: bool,
}

impl Loaded {
    /// Previously decoded native source snapshot, retained through bounded IO.
    pub(crate) fn from_snapshot(text: Rope, bom: bool) -> Self {
        Self { text, bom }
    }
    pub(crate) fn observed_text(&self) -> RopeSlice<'_> {
        self.text.slice(..)
    }
    /// Decode strict UTF-8 incrementally. No candidate survives a read, encoding
    /// or NUL-policy failure; exactly one leading BOM becomes format metadata.
    pub fn read(reader: impl Read) -> io::Result<Self> {
        let mut text = Rope::from_reader(reader)?;
        if text.chunks().any(|chunk| chunk.as_bytes().contains(&0)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NUL bytes indicate unsupported binary input",
            ));
        }
        let bom = text.get_char(0) == Some('\u{feff}');
        if bom {
            text.try_remove(0..1).map_err(io::Error::other)?;
        }
        Ok(Self { text, bom })
    }

    pub(crate) fn into_parts(self) -> (Rope, bool) {
        (self.text, self.bom)
    }
}

/// Reads existing regular files; missing paths and read errors retain their kind.
pub fn load(path: &Path) -> io::Result<Loaded> {
    read_file(path).map_err(|error| io::Error::new(error.kind(), format!("load {path:?}: {error}")))
}

fn read_file(path: &Path) -> io::Result<Loaded> {
    regular(fs::metadata(path)?.is_file())?;
    let file = File::open(path)?;
    regular(file.metadata()?.is_file())?;
    Loaded::read(file)
}

fn regular(is_file: bool) -> io::Result<()> {
    if !is_file {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file",
        ));
    }
    Ok(())
}

/// Serialize exact format/text bytes. The caller owns flush, sync and publication;
/// writing bytes alone never acknowledges a save or changes buffer metadata.
pub fn write(buffer: &Buffer, writer: &mut impl Write) -> io::Result<()> {
    let span = buffer
        .span(0..buffer.len_chars())
        .map_err(io::Error::other)?;
    let text = buffer.slice(&span).map_err(io::Error::other)?;
    write_text(text, buffer.has_bom(), writer)
}

pub(crate) fn write_text(
    text: RopeSlice<'_>,
    bom: bool,
    writer: &mut impl Write,
) -> io::Result<()> {
    if bom {
        writer.write_all(BOM)?;
    }
    for chunk in text.chunks() {
        writer.write_all(chunk.as_bytes())?;
    }
    Ok(())
}
pub mod completion;
mod delete;
mod location;
pub(crate) mod name;
mod observation;
mod output;
pub(crate) mod rename;
mod save;
pub(crate) mod temporary;

pub use delete::{DeleteDecision, Deleted, Deletion};
pub(crate) use location::directory;
pub use location::{Location, resolve};
pub use observation::{Observation, observe};
pub use output::{Files, Output};
pub(crate) use save::publish;
pub use save::{Outcome, Publication};

pub use rename::{Rename, Renamed};
