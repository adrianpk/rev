mod clip;
mod prompt;

use std::io::{self, Write};

use crossterm::{
    QueueableCommand,
    cursor::{Hide, MoveTo, Show},
};

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::{Cell, Coordinates, Glyphs};
use crate::view::View;
pub use crate::view::{Error, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Document,
    Status,
    Echo,
}

/// A surface position, distinct from a logical document cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub column: u16,
    pub row: u16,
}

/// Immutable derived row; its text is padded to exactly the terminal width.
pub struct Row {
    row: u16,
    text: String,
    band: Band,
}

impl Row {
    pub fn row(&self) -> u16 {
        self.row
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn band(&self) -> Band {
        self.band
    }
}

/// One derived frame. The session builds a fresh projection at each redraw;
/// no retained row cache can survive an edit, metadata change, or resize.
pub struct Frame {
    size: Size,
    stamp: Stamp,
    rows: Vec<Row>,
    cursor: Option<Position>,
}

impl Frame {
    pub fn project(buffer: &Buffer, view: &View, size: Size, echo: &str) -> Result<Self, Error> {
        let point = view.cell(buffer)?;
        let mut frame = Self {
            size,
            stamp: buffer.stamp(),
            rows: Vec::new(),
            cursor: None,
        };
        if size.width == 0 || size.height == 0 {
            return Ok(frame);
        }
        if size.usable() {
            frame.document(buffer, view)?;
            frame.status(buffer, point)?;
            frame.cursor = frame.position(point, view);
        } else {
            frame.blank_rows();
        }
        frame.echo(echo)?;
        Ok(frame)
    }

    pub fn stamp(&self) -> Stamp {
        self.stamp
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn cursor(&self) -> Option<Position> {
        self.cursor
    }

    /// Writes only projected rows and an admitted cursor, then flushes once.
    /// Explicit row moves prevent a full-width row from wrapping into its peer.
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.queue(Hide)?;
        for row in &self.rows {
            writer.queue(MoveTo(0, row.row))?;
            writer.write_all(row.text.as_bytes())?;
        }
        if let Some(cursor) = self.cursor {
            writer
                .queue(MoveTo(cursor.column, cursor.row))?
                .queue(Show)?;
        }
        writer.flush()
    }

    fn document(&mut self, buffer: &Buffer, view: &View) -> Result<(), Error> {
        let coordinates = Coordinates::new(buffer)?;
        for row in 0..self.size.text_rows() {
            let source = view.top.saturating_add(usize::from(row));
            let text = if source < buffer.len_lines() {
                clip::line(coordinates.glyphs(source)?, view.left, self.size.width)?
            } else {
                " ".repeat(usize::from(self.size.width))
            };
            self.rows.push(Row {
                row,
                text,
                band: Band::Document,
            });
        }
        Ok(())
    }

    fn status(&mut self, buffer: &Buffer, point: Cell) -> Result<(), Error> {
        let dirty = if buffer.is_modified() { "**" } else { "--" };
        let access = if buffer.is_read_only() { "RO" } else { "RW" };
        let status = format!(
            "> {} {dirty} {access} L{} C{} Text",
            buffer.name(),
            point.row + 1,
            point.column + 1
        );
        self.rows.push(Row {
            row: self.size.height - 2,
            text: clip::line(Glyphs::for_text(&status)?, 0, self.size.width)?,
            band: Band::Status,
        });
        Ok(())
    }

    fn echo(&mut self, message: &str) -> Result<(), Error> {
        self.rows.push(Row {
            row: self.size.height - 1,
            text: clip::line(Glyphs::for_text(message)?, 0, self.size.width)?,
            band: Band::Echo,
        });
        Ok(())
    }

    fn blank_rows(&mut self) {
        for row in 0..self.size.height - 1 {
            self.rows.push(Row {
                row,
                text: " ".repeat(usize::from(self.size.width)),
                band: Band::Document,
            });
        }
    }

    fn position(&self, point: Cell, view: &View) -> Option<Position> {
        let column = point.column.checked_sub(view.left)?;
        let row = point.row.checked_sub(view.top)?;
        if column >= usize::from(self.size.width) || row >= usize::from(self.size.text_rows()) {
            return None;
        }
        Some(Position {
            column: u16::try_from(column).ok()?,
            row: u16::try_from(row).ok()?,
        })
    }
}
