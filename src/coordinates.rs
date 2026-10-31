mod checkpoints;
mod cursor;
pub(crate) use checkpoints::Checkpoints;
mod display;
mod hit;
mod motion;
mod work;
pub(crate) use work::{Budget, Scope, Work};

use std::fmt;

use ropey::RopeSlice;

use crate::buffer::{self, Buffer, Stamp};
use cursor::Cursor;
pub(crate) use display::Checkpoint as GlyphCheckpoint;
pub use display::{Glyph, Glyphs};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scalar(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Byte(pub usize);

/// Zero-based logical row and UTF-8 byte column, as used by syntax parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineByte {
    pub row: usize,
    pub column: usize,
}

/// Zero-based logical row and terminal-cell column, independent of a viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub row: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Scalar(Scalar),
    Byte(Byte),
    Line(LineByte),
    Cluster(Scalar),
    Segmentation,
    Overflow,
    Pending,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(point) => write!(formatter, "scalar {} is outside the text", point.0),
            Self::Byte(byte) => write!(formatter, "byte {} is not a valid UTF-8 boundary", byte.0),
            Self::Line(point) => write!(
                formatter,
                "invalid line-byte position {}:{}",
                point.row, point.column
            ),
            Self::Cluster(point) => write!(formatter, "scalar {} is inside a grapheme", point.0),
            Self::Segmentation => formatter.write_str("grapheme context could not be resolved"),
            Self::Overflow => formatter.write_str("display column overflow"),
            Self::Pending => formatter.write_str("source projection pending"),
        }
    }
}

impl std::error::Error for Error {}

/// Borrows one buffer revision and effective display policy; neither can change
/// while these coordinates remain borrowed.
pub struct Coordinates<'a> {
    text: RopeSlice<'a>,
    stamp: Stamp,
    tab_width: usize,
    checkpoints: Option<&'a std::sync::Mutex<Checkpoints>>,
}

impl<'a> Coordinates<'a> {
    pub fn new(buffer: &'a Buffer) -> Result<Self, buffer::Error> {
        let span = buffer.span(0..buffer.len_chars())?;
        Ok(Self {
            text: buffer.slice(&span)?,
            stamp: span.stamp,
            tab_width: buffer.policy().tab_width,
            checkpoints: Some(&buffer.coordinates),
        })
    }

    pub(crate) fn for_text(text: RopeSlice<'a>, stamp: Stamp, tab_width: usize) -> Self {
        Self {
            text,
            stamp,
            tab_width,
            checkpoints: None,
        }
    }

    pub fn stamp(&self) -> Stamp {
        self.stamp
    }

    pub fn to_byte(&self, point: Scalar) -> Result<Byte, Error> {
        self.text
            .try_char_to_byte(point.0)
            .map(Byte)
            .map_err(|_| Error::Scalar(point))
    }

    pub fn from_byte(&self, byte: Byte) -> Result<Scalar, Error> {
        let scalar = self
            .text
            .try_byte_to_char(byte.0)
            .map_err(|_| Error::Byte(byte))?;
        if self.to_byte(Scalar(scalar))? != byte {
            return Err(Error::Byte(byte));
        }
        Ok(Scalar(scalar))
    }

    pub fn to_line(&self, point: Scalar) -> Result<LineByte, Error> {
        let byte = self.to_byte(point)?.0;
        let row = self
            .text
            .try_char_to_line(point.0)
            .map_err(|_| Error::Scalar(point))?;
        let start = self
            .text
            .try_line_to_byte(row)
            .map_err(|_| Error::Line(LineByte { row, column: 0 }))?;
        Ok(LineByte {
            row,
            column: byte - start,
        })
    }

    /// Rejects a previous row's end alias for the next row's byte zero.
    pub fn from_line(&self, point: LineByte) -> Result<Scalar, Error> {
        let line = self.text.get_line(point.row).ok_or(Error::Line(point))?;
        if point.column == line.len_bytes() && point.row + 1 < self.text.len_lines() {
            return Err(Error::Line(point));
        }
        line.get_byte_slice(point.column..point.column)
            .ok_or(Error::Line(point))?;
        let local = line
            .try_byte_to_char(point.column)
            .map_err(|_| Error::Line(point))?;
        let start = self
            .text
            .try_line_to_char(point.row)
            .map_err(|_| Error::Line(point))?;
        Ok(Scalar(start + local))
    }

    pub fn is_boundary(&self, point: Scalar) -> Result<bool, Error> {
        let row = self.to_line(point)?.row;
        if self
            .checkpoint(point, row)
            .is_some_and(|(at, _)| at == point)
        {
            return Ok(true);
        }
        Cursor::new(self.text, self.to_byte(point)?.0)?.boundary()
    }

    /// Moves to the next complete extended grapheme, clamping at EOF.
    pub fn next(&self, point: Scalar) -> Result<Scalar, Error> {
        let byte = self.to_byte(point)?.0;
        let next = Cursor::new(self.text, byte)?.next()?.unwrap_or(byte);
        self.from_byte(Byte(next))
    }

    /// Moves to the preceding complete extended grapheme, clamping at BOF.
    pub fn previous(&self, point: Scalar) -> Result<Scalar, Error> {
        let byte = self.to_byte(point)?.0;
        let previous = Cursor::new(self.text, byte)?.previous()?.unwrap_or(byte);
        self.from_byte(Byte(previous))
    }

    pub fn glyphs(&self, row: usize) -> Result<Glyphs<'a>, Error> {
        let (line, start) = self.body(row)?;
        Glyphs::new(line, start, self.tab_width)
    }

    /// Resume only from an observed grapheme boundary at its logical column.
    /// The source observation supplies the boundary; tabs keep absolute columns.
    pub(crate) fn glyphs_at(
        &self,
        row: usize,
        point: Scalar,
        column: usize,
    ) -> Result<Glyphs<'a>, Error> {
        let (line, start) = self.body(row)?;
        Glyphs::at(line, start, self.tab_width, point, column)
    }

    pub(crate) fn resume_glyphs(
        &self,
        row: usize,
        at: GlyphCheckpoint,
    ) -> Result<Glyphs<'a>, Error> {
        let (line, start) = self.body(row)?;
        Glyphs::resume(line, start, self.tab_width, at)
    }

    pub(crate) fn remember_cell(&self, point: Scalar, cell: Cell) {
        self.remember(point, cell);
    }

    fn checkpoint(&self, point: Scalar, row: usize) -> Option<(Scalar, usize)> {
        let mut cache = self.checkpoints?.lock().expect("coordinate cache poisoned");
        cache.observe(self.stamp, self.tab_width);
        cache.before(point, row)
    }

    fn column_checkpoint(&self, cell: Cell) -> Option<(Scalar, usize)> {
        let (line, start) = self.body(cell.row).ok()?;
        let end = Scalar(start.0 + line.len_chars());
        let mut cache = self.checkpoints?.lock().expect("coordinate cache poisoned");
        cache.observe(self.stamp, self.tab_width);
        cache.column(cell, start..=end)
    }

    fn remember(&self, point: Scalar, cell: Cell) {
        if let Some(cache) = self.checkpoints {
            let mut cache = cache.lock().expect("coordinate cache poisoned");
            cache.observe(self.stamp, self.tab_width);
            cache.remember(point, cell);
        }
    }

    fn pause(&self, point: Scalar, cell: Cell) {
        if let Some(cache) = self.checkpoints {
            let mut cache = cache.lock().expect("coordinate cache poisoned");
            cache.observe(self.stamp, self.tab_width);
            cache.pause(point, cell);
        }
    }

    pub(crate) fn work(&self) -> Option<Work> {
        self.checkpoints?
            .lock()
            .expect("coordinate cache poisoned")
            .work
            .clone()
    }

    fn charge(work: &Option<Work>) -> bool {
        work.as_ref()
            .is_none_or(|work| work.lock().expect("layout budget poisoned").charge())
    }

    pub(crate) fn glyphs_from_column(&self, cell: Cell) -> Result<Glyphs<'a>, Error> {
        let (line, start) = self.body(cell.row)?;
        if cell.column == 0 {
            return Glyphs::new(line, start, self.tab_width);
        }
        self.from_cell(cell)?;
        let (point, column) = self.column_checkpoint(cell).unwrap_or((start, 0));
        self.glyphs_at(cell.row, point, column)
    }

    /// Converts only grapheme boundaries; an interior cluster has no unique cell.
    pub fn to_cell(&self, point: Scalar) -> Result<Cell, Error> {
        let row = self.to_line(point)?.row;
        let (line, start) = self.body(row)?;
        if point == start {
            return Ok(Cell { row, column: 0 });
        }
        let (from, mut column) = self.checkpoint(point, row).unwrap_or((start, 0));
        if from == point {
            return Ok(Cell { row, column });
        }
        let work = self.work();
        let mut last = from;
        let mut glyphs = self.glyphs_at(row, from, column)?;
        let mut index = 0;
        loop {
            if !Self::charge(&work) {
                self.pause(last, Cell { row, column });
                return Err(Error::Pending);
            }
            let Some(glyph) = glyphs.next() else { break };
            let glyph = glyph?;
            if point < glyph.source.end {
                let cell = cluster_cell(point, &glyph, row)?;
                self.remember(point, cell);
                return Ok(cell);
            }
            column = glyph.columns.end;
            last = glyph.source.end;
            index += 1;
            if index % checkpoints::INTERVAL == 0 {
                self.remember(glyph.source.end, Cell { row, column });
            }
        }
        if point.0 != start.0 + line.len_chars() {
            return Err(Error::Cluster(point));
        }
        let cell = Cell { row, column };
        self.remember(point, cell);
        Ok(cell)
    }

    /// Interior wide/tab cells map to their source start; columns past EOL clamp.
    pub fn from_cell(&self, cell: Cell) -> Result<Scalar, Error> {
        let (line, start) = self.body(cell.row)?;
        if cell.column == 0 {
            return Ok(start);
        }
        let (mut last, mut column) = self.column_checkpoint(cell).unwrap_or((start, 0));
        if column == cell.column || last.0 == start.0 + line.len_chars() {
            return Ok(last);
        }
        let work = self.work();
        let mut glyphs = self.glyphs_at(cell.row, last, column)?;
        let mut index = 0;
        loop {
            if !Self::charge(&work) {
                self.pause(
                    last,
                    Cell {
                        row: cell.row,
                        column,
                    },
                );
                return Err(Error::Pending);
            }
            let Some(glyph) = glyphs.next() else { break };
            let glyph = glyph?;
            column = glyph.columns.end;
            if cell.column < column {
                self.remember(
                    glyph.source.start,
                    Cell {
                        row: cell.row,
                        column: glyph.columns.start,
                    },
                );
                return Ok(glyph.source.start);
            }
            last = glyph.source.end;
            index += 1;
            if index % checkpoints::INTERVAL == 0 {
                self.remember(
                    glyph.source.end,
                    Cell {
                        row: cell.row,
                        column,
                    },
                );
            }
        }
        // The checkpoint may itself be EOL and have no remaining glyph.
        column = column.max(self.column_checkpoint(cell).map_or(0, |(_, column)| column));
        let end = Scalar(start.0 + line.len_chars());
        self.remember(
            end,
            Cell {
                row: cell.row,
                column,
            },
        );
        Ok(end)
    }

    fn body(&self, row: usize) -> Result<(RopeSlice<'a>, Scalar), Error> {
        let position = LineByte { row, column: 0 };
        let line = self.text.get_line(row).ok_or(Error::Line(position))?;
        let start = self
            .text
            .try_line_to_char(row)
            .map_err(|_| Error::Line(position))?;
        let mut end = line.len_chars();
        if end > 0 && line.get_char(end - 1) == Some('\n') {
            end -= 1;
            if end > 0 && line.get_char(end - 1) == Some('\r') {
                end -= 1;
            }
        }
        let body = line.get_slice(0..end).ok_or(Error::Line(position))?;
        Ok((body, Scalar(start)))
    }
}

fn cluster_cell(point: Scalar, glyph: &Glyph<'_>, row: usize) -> Result<Cell, Error> {
    if point != glyph.source.start {
        return Err(Error::Cluster(point));
    }
    Ok(Cell {
        row,
        column: glyph.columns.start,
    })
}
