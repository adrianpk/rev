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
    let units = match encoding {
        Encoding::Utf8 => text.char_to_byte(scalar) - text.line_to_byte(row),
        Encoding::Utf16 => text.slice(start..scalar).chars().map(char::len_utf16).sum(),
    };
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
    let mut units = 0;
    for scalar in start..end {
        if units == position.character as usize {
            return Ok(scalar);
        }
        units += match encoding {
            Encoding::Utf8 => text.char(scalar).len_utf8(),
            Encoding::Utf16 => text.char(scalar).len_utf16(),
        };
    }
    if units == position.character as usize {
        Ok(end)
    } else {
        Err("LSP column splits an encoding unit or exceeds its line".into())
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
