use std::ops::Range;

use crate::buffer::{Buffer, Span};
use crate::coordinates::{Byte, Coordinates, LineByte, Scalar};

pub(super) struct Source<'a> {
    pub(super) buffer: &'a Buffer,
    pub(super) coordinates: Coordinates<'a>,
}

pub(super) struct Line {
    pub(super) bytes: Range<usize>,
    pub(super) body_end: usize,
    pub(super) quote_end: usize,
    pub(super) indent_end: usize,
    pub(super) before_indent: usize,
    pub(super) after_indent: usize,
}

impl<'a> Source<'a> {
    pub(super) fn new(buffer: &'a Buffer) -> Result<Self, String> {
        Ok(Self {
            buffer,
            coordinates: Coordinates::new(buffer).map_err(|e| e.to_string())?,
        })
    }

    pub(super) fn scalar(&self, byte: usize) -> Result<Scalar, String> {
        self.coordinates
            .from_byte(Byte(byte))
            .map_err(|e| e.to_string())
    }

    pub(super) fn span(&self, bytes: Range<usize>) -> Result<Span, String> {
        self.buffer
            .span(self.scalar(bytes.start)?.0..self.scalar(bytes.end)?.0)
            .map_err(|e| e.to_string())
    }

    pub(super) fn text(&self, bytes: Range<usize>) -> Result<String, String> {
        self.buffer
            .slice(&self.span(bytes)?)
            .map(|s| s.to_string())
            .map_err(|e| e.to_string())
    }

    pub(super) fn line(&self, byte: usize, quotes: usize) -> Result<Line, String> {
        let row = self
            .coordinates
            .to_line(self.scalar(byte)?)
            .map_err(|e| e.to_string())?
            .row;
        let start = self.line_start(row)?;
        let end = if row + 1 < self.buffer.len_lines() {
            self.line_start(row + 1)?
        } else {
            self.buffer.len_bytes()
        };
        let raw = self.text(start..end)?;
        let body_end = end - separator(&raw);
        let quote_end = super::super::item::quote_end(&raw, quotes);
        let indentation = raw[quote_end..]
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .count();
        let indent_end = quote_end + indentation;
        Ok(Line {
            bytes: start..end,
            body_end,
            quote_end: start + quote_end,
            indent_end: start + indent_end,
            before_indent: super::super::item::columns(&raw[..quote_end]),
            after_indent: super::super::item::columns(&raw[..indent_end]),
        })
    }

    fn line_start(&self, row: usize) -> Result<usize, String> {
        let scalar = self
            .coordinates
            .from_line(LineByte { row, column: 0 })
            .map_err(|e| e.to_string())?;
        self.coordinates
            .to_byte(scalar)
            .map(|b| b.0)
            .map_err(|e| e.to_string())
    }

    pub(super) fn item_end(&self, item: &super::super::Item) -> Result<usize, String> {
        let raw = self.text(item.bytes.clone())?;
        let last = raw.trim_end_matches([' ', '\t', '\r', '\n']).len();
        Ok(self.line(item.bytes.start + last, item.quotes)?.bytes.end)
    }
}

pub(super) fn separator(source: &str) -> usize {
    if source.ends_with("\r\n") {
        2
    } else {
        usize::from(source.ends_with('\n'))
    }
}
