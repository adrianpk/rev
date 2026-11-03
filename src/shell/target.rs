//! Checked native source capture. Selected text is data, never shell syntax.
use crate::{
    buffer::{Buffer, Span, edit::Point},
    file::Location,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Command,
    Region,
}
impl Kind {
    pub(crate) fn has_input(self) -> bool {
        self == Self::Region
    }
}

pub(crate) struct Target {
    pub(crate) source: Span,
    pub(crate) file: Option<Location>,
    pub(crate) kind: Kind,
}
impl Target {
    pub(crate) fn capture(buffer: &Buffer, point: Point, kind: Kind) -> Result<Self, String> {
        buffer
            .slice(&Span {
                stamp: point.stamp,
                range: point.position.0..point.position.0,
            })
            .map_err(|e| e.to_string())?;
        let source = if kind.has_input() {
            if buffer.rectangle_active() {
                return Err("Shell region input requires a linear selection".into());
            }
            buffer
                .region(point)
                .map_err(|e| e.to_string())?
                .filter(|s| !s.range.is_empty())
                .ok_or("Shell region input requires a nonempty linear selection")?
        } else {
            buffer
                .span(point.position.0..point.position.0)
                .map_err(|e| e.to_string())?
        };
        Ok(Self {
            source,
            file: buffer.file().cloned(),
            kind,
        })
    }
    pub(crate) fn input(&self, buffer: &Buffer, limit: usize) -> Result<Option<Vec<u8>>, String> {
        if !self.kind.has_input() {
            return Ok(None);
        }
        let slice = buffer.slice(&self.source).map_err(|e| e.to_string())?;
        if slice.len_bytes() > limit {
            return Err("Shell region exceeds max_input_bytes".into());
        }
        Ok(Some(slice.to_string().into_bytes()))
    }
    pub(crate) fn current(&self, buffer: &Buffer) -> Result<(), String> {
        buffer
            .slice(&self.source)
            .map_err(|e| format!("Shell target changed: {e}"))?;
        if buffer.file() != self.file.as_ref() {
            return Err("Shell target file association changed".into());
        }
        Ok(())
    }
}
