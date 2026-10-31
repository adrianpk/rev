mod fold;
mod hit;
pub use fold::FoldError;
mod scroll;
mod visibility;
mod visual;
mod wrap;
pub(crate) use wrap::Budget as LayoutBudget;
pub use wrap::Fragment;
pub(crate) use wrap::LayoutScope;

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
    Fold(FoldError),
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
            Self::Fold(error) => error.fmt(formatter),
            Self::WorkingDirectory(kind) => write!(formatter, "read working directory: {kind}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn pending(self) -> bool {
        matches!(
            self,
            Self::Coordinates(coordinates::Error::Pending)
                | Self::Edit(buffer::edit::Error::Coordinates(
                    coordinates::Error::Pending
                ))
        )
    }
}

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
    /// Independent view presentation, inherited by splits and retained on rebind.
    pub bookmark_markers: bool,
    bookmark_marker: String,
    pub preview: crate::org::preview::State,
    /// Intended terminal-cell column, retained across vertical/page/recenter input.
    pub column: Option<usize>,
    pub top: usize,
    pub left: usize,
    /// Source anchor within the top logical line; reflow resolves it afresh.
    pub(crate) wrap_start: Scalar,
    wrap_cache: wrap::Cache,
    wrap_width: Option<u16>,
    wrap_active: bool,
    folds: Option<fold::Folds>,
    visibility: Option<std::sync::Arc<crate::org::Visibility>>,
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
        Ok(crate::gutter::Geometry::with_marker(
            content,
            buffer.len_lines(),
            self.gutter.effective(),
            self.bookmark_markers,
        )?)
    }

    pub fn new(buffer: &Buffer) -> Self {
        Self {
            point: Point {
                stamp: buffer.stamp(),
                position: Scalar(0),
            },
            gutter: crate::gutter::Policy::default(),
            bookmark_markers: false,
            bookmark_marker: "●".into(),
            preview: crate::org::preview::State::default(),
            column: None,
            top: 0,
            left: 0,
            wrap_start: Scalar(0),
            wrap_cache: wrap::Cache::default(),
            wrap_width: None,
            wrap_active: buffer.soft_wrap(),
            folds: None,
            visibility: None,
            alignment: scroll::Alignment::Center,
            tab_width: buffer.policy().tab_width,
        }
    }

    /// Construct an initial view from the same admitted session snapshot used by
    /// buffers and input. Later rebinds retain this policy instead of reloading it.
    pub fn with_settings(buffer: &Buffer, settings: &crate::config::Snapshot) -> Self {
        let mut view = Self::new(buffer);
        view.gutter = crate::gutter::Policy::new(settings.settings().gutter);
        view.preview.enabled = settings.settings().org.preview.enabled;
        view
    }

    /// Switch canonical text using the ordinary new-buffer navigation state,
    /// retaining only this view's independent presentation policy and provenance.
    pub fn rebind(&mut self, buffer: &Buffer) {
        let gutter = self.gutter;
        let markers = self.bookmark_markers;
        let marker = self.bookmark_marker.clone();
        let enabled = self.preview.enabled;
        *self = Self::new(buffer);
        self.gutter = gutter;
        self.bookmark_markers = markers;
        self.bookmark_marker = marker;
        self.preview.enabled = enabled;
    }

    pub(crate) fn bookmark_marker(&self) -> &str {
        &self.bookmark_marker
    }

    pub(crate) fn clone_for_split(&self) -> Self {
        let mut view = self.clone();
        view.wrap_cache = wrap::Cache::default();
        view
    }

    pub fn cell(&self, buffer: &Buffer) -> Result<Cell, Error> {
        buffer.slice(&Span {
            stamp: self.point.stamp,
            range: self.point.position.0..self.point.position.0,
        })?;
        let coordinates = Coordinates::new(buffer)?;
        self.wrap_cache.remember_point_columns(
            buffer,
            coordinates.to_line(self.point.position)?.row,
            self.point.position,
        )?;
        Ok(coordinates.to_cell(self.point.position)?)
    }

    pub(crate) fn check_point(&self, buffer: &Buffer) -> Result<(), Error> {
        self.check_source(buffer)?;
        if !Coordinates::new(buffer)?.is_boundary(self.point.position)? {
            return Err(coordinates::Error::Cluster(self.point.position).into());
        }
        Ok(())
    }

    pub(crate) fn check_source(&self, buffer: &Buffer) -> Result<(), Error> {
        buffer.slice(&Span {
            stamp: self.point.stamp,
            range: self.point.position.0..self.point.position.0,
        })?;
        Ok(())
    }

    /// Reveal point in usable geometry. Suspended presentation preserves scroll.
    pub fn reveal(&mut self, buffer: &Buffer, size: Size) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        self.reveal_fold(buffer, point.row)?;
        self.sync_policy(buffer);
        if size.usable() {
            let content = crate::window::Area::new(0, 0, size.width, size.text_rows())
                .ok_or(coordinates::Error::Overflow)?;
            let width = self.gutter_geometry(buffer, content)?.text().width();
            if buffer.soft_wrap() && width > 0 {
                self.left = 0;
                return self.reveal_wrapped(buffer, width, size.text_rows());
            }
            let mut top = self.projected_top(buffer);
            let row = self
                .project_row(buffer, point.row)
                .ok_or(coordinates::Error::Overflow)?;
            reveal(&mut top, row, usize::from(size.text_rows()));
            self.top = self
                .source_at(buffer, top)
                .ok_or(coordinates::Error::Overflow)?;
            reveal(&mut self.left, point.column, usize::from(width));
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

    fn sync_wrap_width(&mut self, width: Option<u16>) {
        if self.wrap_width.is_some() && self.wrap_width != width
            || self.wrap_active != width.is_some()
        {
            self.reset_navigation();
        }
        self.wrap_width = width;
        self.wrap_active = width.is_some();
    }
}

fn reveal(origin: &mut usize, point: usize, extent: usize) {
    if point < *origin {
        *origin = point;
    } else if point - *origin >= extent {
        *origin = point - (extent - 1);
    }
}
