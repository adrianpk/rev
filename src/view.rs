mod scroll;

use std::fmt;

use crate::buffer::{self, Buffer, Span, edit::Point};
use crate::coordinates::{self, Cell, Coordinates, Scalar};

/// Native terminal dimensions; zero is a valid suspended-presentation input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn usable(self) -> bool {
        self.width >= 3 && self.height >= 3
    }

    pub fn text_rows(self) -> u16 {
        if self.usable() { self.height - 2 } else { 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Buffer(buffer::Error),
    Edit(buffer::edit::Error),
    Coordinates(coordinates::Error),
    Window(crate::window::Error),
    Gutter(crate::gutter::Error),
    WorkingDirectory(std::io::ErrorKind),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buffer(error) => error.fmt(formatter),
            Self::Edit(error) => error.fmt(formatter),
            Self::Coordinates(error) => error.fmt(formatter),
            Self::Window(error) => error.fmt(formatter),
            Self::Gutter(error) => error.fmt(formatter),
            Self::WorkingDirectory(kind) => write!(formatter, "read working directory: {kind}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::gutter::Error> for Error {
    fn from(error: crate::gutter::Error) -> Self {
        Self::Gutter(error)
    }
}

impl From<buffer::Error> for Error {
    fn from(error: buffer::Error) -> Self {
        Self::Buffer(error)
    }
}

impl From<buffer::edit::Error> for Error {
    fn from(error: buffer::edit::Error) -> Self {
        Self::Edit(error)
    }
}

impl From<coordinates::Error> for Error {
    fn from(error: coordinates::Error) -> Self {
        Self::Coordinates(error)
    }
}

impl From<crate::window::Error> for Error {
    fn from(error: crate::window::Error) -> Self {
        Self::Window(error)
    }
}

/// A view owns point and scroll, never document text. Top is a logical row;
/// left is a terminal-cell offset. Its point identifies the displayed buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub point: Point,
    /// Session-only presentation values, copied by splits and never text data.
    pub gutter: crate::gutter::Policy,
    /// Intended terminal-cell column, retained across vertical/page/recenter input.
    pub column: Option<usize>,
    pub top: usize,
    pub left: usize,
    alignment: scroll::Alignment,
    tab_width: usize,
}

impl View {
    /// One number/text partition shared by rendering, overlays and reveal.
    pub fn gutter_geometry(
        &self,
        buffer: &Buffer,
        content: crate::window::Area,
    ) -> Result<crate::gutter::Geometry, Error> {
        Ok(crate::gutter::Geometry::new(
            content,
            buffer.len_lines(),
            self.gutter.effective(),
        )?)
    }

    pub fn new(buffer: &Buffer) -> Self {
        Self {
            point: Point {
                stamp: buffer.stamp(),
                position: Scalar(0),
            },
            gutter: crate::gutter::Policy::default(),
            column: None,
            top: 0,
            left: 0,
            alignment: scroll::Alignment::Center,
            tab_width: buffer.policy().tab_width,
        }
    }

    /// Switch canonical text using the ordinary new-buffer navigation state,
    /// retaining only this view's independent presentation policy and provenance.
    pub fn rebind(&mut self, buffer: &Buffer) {
        let gutter = self.gutter;
        *self = Self::new(buffer);
        self.gutter = gutter;
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
        self.sync_policy(buffer);
        if size.usable() {
            reveal(&mut self.top, point.row, usize::from(size.text_rows()));
            reveal(&mut self.left, point.column, usize::from(size.width));
        }
        Ok(())
    }

    // Goals are display columns, not canonical positions. Width comparison
    // invalidates them before use without advancing the text publication stamp.
    fn sync_policy(&mut self, buffer: &Buffer) {
        let width = buffer.policy().tab_width;
        if self.tab_width != width {
            self.reset_navigation();
            self.tab_width = width;
        }
    }
}

fn reveal(origin: &mut usize, point: usize, extent: usize) {
    if point < *origin {
        *origin = point;
    } else if point - *origin >= extent {
        *origin = point - (extent - 1);
    }
}
