use std::ops::Range;

use ropey::RopeSlice;

use crate::buffer::{Buffer, Error, Span, Stamp};
use crate::coordinates::Byte;

/// An immutable observation of one canonical buffer revision, without copied text.
pub struct Source<'a> {
    buffer: &'a Buffer,
}

/// A source location qualified by buffer identity and publication revision.
/// Endpoints are UTF-8 scalar boundaries, not display cells or grapheme claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRange {
    span: Span,
}

impl<'a> Source<'a> {
    pub fn new(buffer: &'a Buffer) -> Self {
        Self { buffer }
    }

    pub fn stamp(&self) -> Stamp {
        self.buffer.stamp()
    }

    /// Admit half-open byte offsets, including empty ranges at BOF and EOF.
    /// Reversed, out-of-bounds and interior UTF-8 endpoints return buffer errors.
    pub fn range(&self, bytes: Range<Byte>) -> Result<SourceRange, Error> {
        let span = self.buffer.byte_span(bytes.start.0..bytes.end.0)?;
        Ok(SourceRange { span })
    }

    /// Recheck identity, revision and bounds before borrowing original rope chunks.
    /// Retaining a range after an edit never makes it authority over newer text.
    pub fn text(&self, source: &SourceRange) -> Result<RopeSlice<'a>, Error> {
        self.buffer.slice(&source.span)
    }
}

impl SourceRange {
    pub fn stamp(&self) -> Stamp {
        self.span.stamp
    }

    /// Reuse the canonical scalar range for editor transactions. The edit owner
    /// still checks freshness, read-only state, view-point boundaries and history.
    pub fn span(&self) -> &Span {
        &self.span
    }
}
