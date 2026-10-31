use super::Encoding;
use ropey::Rope;
use serde::{Deserialize, Serialize};

/// Protocol columns are encoding units, distinct from scalar/display columns.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

pub fn encode(text: &Rope, scalar: usize, encoding: &Encoding) -> Result<Position, String> {
    if scalar > text.len_chars() {
        return Err("LSP scalar is outside the document".into());
    }
    boundary(text, scalar)?;
    let row = text.char_to_line(scalar);
    let start = text.line_to_char(row);
    let units = offset(text, scalar, encoding) - offset(text, start, encoding);
    Ok(Position {
        line: u32::try_from(row).map_err(|_| "LSP line overflow")?,
        character: u32::try_from(units).map_err(|_| "LSP column overflow")?,
    })
}

pub fn decode(text: &Rope, position: Position, encoding: &Encoding) -> Result<usize, String> {
    let row = position.line as usize;
    if row >= text.len_lines() {
        return Err("LSP line is outside the document".into());
    }
    let start = text.line_to_char(row);
    let end = line_end(text, row);
    let target = offset(text, start, encoding)
        .checked_add(position.character as usize)
        .filter(|target| *target <= offset(text, end, encoding))
        .ok_or("LSP column exceeds its line")?;
    let scalar = match encoding {
        Encoding::Utf8 => text.byte_to_char(target),
        Encoding::Utf16 => text.utf16_cu_to_char(target),
    };
    if offset(text, scalar, encoding) != target {
        return Err("LSP column splits an encoding unit".into());
    }
    Ok(scalar)
}

// The rope index avoids rescanning a long line for every edit range endpoint.
fn offset(text: &Rope, scalar: usize, encoding: &Encoding) -> usize {
    match encoding {
        Encoding::Utf8 => text.char_to_byte(scalar),
        Encoding::Utf16 => text.char_to_utf16_cu(scalar),
    }
}

fn line_end(text: &Rope, row: usize) -> usize {
    let start = text.line_to_char(row);
    let mut end = start + text.line(row).len_chars();
    if end > start && text.char(end - 1) == '\n' {
        end -= 1;
    }
    if end > start && text.char(end - 1) == '\r' {
        end -= 1;
    }
    end
}

fn boundary(text: &Rope, scalar: usize) -> Result<(), String> {
    let between = scalar > 0 && scalar < text.len_chars();
    if between && text.char(scalar - 1) == '\r' && text.char(scalar) == '\n' {
        return Err("LSP position splits a CRLF ending".into());
    }
    Ok(())
}
