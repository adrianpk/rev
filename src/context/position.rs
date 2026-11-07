use super::Error;
use crate::buffer::Buffer;
use crate::coordinates::{Coordinates, Scalar};

/// One-based logical line and Unicode-scalar column, never a UTF-8 byte offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}
impl SourcePosition {
    pub fn validate(self) -> Result<(), Error> {
        if self.line == 0
            || self.column == 0
            || i64::try_from(self.line).is_err()
            || i64::try_from(self.column).is_err()
        {
            return Err(Error::Invalid("source position"));
        }
        Ok(())
    }
    pub fn capture(buffer: &Buffer, point: Scalar) -> Result<Self, Error> {
        let coordinates =
            Coordinates::new(buffer).map_err(|_| Error::Invalid("source coordinates"))?;
        let line = coordinates
            .to_line(point)
            .map_err(|_| Error::Invalid("source point"))?;
        let start = coordinates
            .from_line(crate::coordinates::LineByte {
                row: line.row,
                column: 0,
            })
            .map_err(|_| Error::Invalid("source line"))?;
        Ok(Self {
            line: line.row + 1,
            column: point.0 - start.0 + 1,
        })
    }
    /// Clamp changed source text at line ends, excluding CR/LF. Scalar-to-byte
    /// conversion remains with the native coordinate owner.
    pub fn restore(self, buffer: &Buffer) -> Result<Scalar, Error> {
        self.validate()?;
        let span = buffer
            .span(0..buffer.len_chars())
            .map_err(|_| Error::Invalid("source span"))?;
        let text = buffer
            .slice(&span)
            .map_err(|_| Error::Invalid("source text"))?;
        let row = (self.line - 1).min(text.len_lines().saturating_sub(1));
        let line = text.line(row);
        let mut end = line.len_chars();
        if end > 0 && line.char(end - 1) == '\n' {
            end -= 1;
            if end > 0 && line.char(end - 1) == '\r' {
                end -= 1;
            }
        }
        let point = Scalar(text.line_to_char(row) + (self.column - 1).min(end));
        Coordinates::new(buffer)
            .map_err(|_| Error::Invalid("source coordinates"))?
            .to_byte(point)
            .map_err(|_| Error::Invalid("restored source point"))?;
        Ok(point)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewport {
    pub anchor: SourcePosition,
    pub horizontal_cells: usize,
}
