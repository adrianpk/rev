use std::fmt;

use crate::buffer::{self, Buffer, Span, edit::Point};
use crate::coordinates::{self, Cell, Coordinates, Scalar};
use crate::display::Size;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Buffer(buffer::Error),
    Coordinates(coordinates::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buffer(error) => error.fmt(formatter),
            Self::Coordinates(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for Error {}

impl From<buffer::Error> for Error {
    fn from(error: buffer::Error) -> Self {
        Self::Buffer(error)
    }
}

impl From<coordinates::Error> for Error {
    fn from(error: coordinates::Error) -> Self {
        Self::Coordinates(error)
    }
}

/// A view owns point and scroll, never document text. Top is a logical row;
/// left is a terminal-cell offset. Its point identifies the displayed buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub point: Point,
    pub top: usize,
    pub left: usize,
}

impl View {
    pub fn new(buffer: &Buffer) -> Self {
        Self {
            point: Point {
                stamp: buffer.stamp(),
                position: Scalar(0),
            },
            top: 0,
            left: 0,
        }
    }

    pub fn cell(&self, buffer: &Buffer) -> Result<Cell, Error> {
        buffer.slice(&Span {
            stamp: self.point.stamp,
            range: self.point.position.0..self.point.position.0,
        })?;
        Ok(Coordinates::new(buffer)?.to_cell(self.point.position)?)
    }

    /// Reveal point in usable geometry. Suspended presentation preserves scroll.
    pub fn reveal(&mut self, buffer: &Buffer, size: Size) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        if size.usable() {
            reveal(&mut self.top, point.row, usize::from(size.text_rows()));
            reveal(&mut self.left, point.column, usize::from(size.width));
        }
        Ok(())
    }
}

fn reveal(origin: &mut usize, point: usize, extent: usize) {
    if point < *origin {
        *origin = point;
    } else if point - *origin >= extent {
        *origin = point - (extent - 1);
    }
}
