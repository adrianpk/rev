mod agenda;
mod agenda_actions;
pub(crate) mod bookmarks;
mod query;
pub use query::QueryDisplay;
mod calendar;
mod clip;
mod completion;
mod compose;
mod gutter;
mod habit;
mod native_completion;
pub use agenda::AgendaDisplay;
pub use habit::HabitDisplay;
mod inventory;
pub mod preview;
mod prompt;
mod search;
mod selection;
mod status;
mod style;
mod syntax;
pub use style::Message;
use style::{Face, Tone};
mod timer;
mod tree;
pub use syntax::Paint;

use std::io::{self, Write};

use crossterm::{
    QueueableCommand,
    cursor::{Hide, MoveTo, Show},
};

use crate::buffer::{Buffer, Stamp};
use crate::coordinates::{Cell, Coordinates, Glyphs};
use crate::panel::Panel;
use crate::view::View;
pub use crate::view::{Error, Size};
use crate::window::Area;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Document,
    Status,
    Echo,
    Completion,
    Divider,
    Mixed,
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
    tone: Tone,
    faces: Vec<Face>,
    syntax: Vec<Paint>,
    selection: Option<std::ops::Range<u16>>,
    search: Option<std::ops::Range<u16>>,
}

impl Row {
    pub fn row(&self) -> u16 {
        self.row
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> Option<std::ops::Range<u16>> {
        self.selection.clone()
    }

    pub fn search(&self) -> Option<std::ops::Range<u16>> {
        self.search.clone()
    }

    pub fn syntax(&self) -> &[Paint] {
        &self.syntax
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
    palette: crate::config::Palette,
    rows: Vec<Row>,
    cursor: Option<Position>,
    previews: Vec<preview::Placement>,
    source_rows: Vec<SourceRows>,
}

/// Metadata for this immutable frame only, separate from retained view caches.
struct SourceRows {
    area: Area,
    stamp: Stamp,
    rows: Vec<(usize, usize, u16)>,
}

impl Frame {
    /// No text hit or cursor is exposed while a cooperative source projection
    /// is incomplete. A later frame uses only the current admitted mapping.
    pub(crate) fn pending(
        buffer: &Buffer,
        size: Size,
        palette: &crate::config::Palette,
    ) -> Result<Self, Error> {
        let mut frame = Self {
            size,
            stamp: buffer.stamp(),
            palette: palette.clone(),
            rows: Vec::new(),
            cursor: None,
            previews: Vec::new(),
            source_rows: Vec::new(),
        };
        if size.width > 0 && size.height > 0 {
            frame.blank_rows();
            frame.echo("Reflowing text")?;
        }
        Ok(frame)
    }

    pub fn project(buffer: &Buffer, view: &View, size: Size, echo: &str) -> Result<Self, Error> {
        view.cell(buffer)?;
        let mut frame = Self {
            size,
            stamp: buffer.stamp(),
            palette: crate::config::Palette::default(),
            rows: Vec::new(),
            cursor: None,
            previews: Vec::new(),
            source_rows: Vec::new(),
        };
        if size.width == 0 || size.height == 0 {
            return Ok(frame);
        }
        frame.blank_rows();
        frame.echo(echo)?;
        if size.usable() {
            let area = Area::new(0, 0, size.width, size.height - 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            frame.panel(buffer, view, Panel::new(area, true, true))?;
        }
        Ok(frame)
    }

    /// Supply the global palette for a standalone projection. The frame owns
    /// only a derived copy; captures, canonical text and buffer policy stay put.
    /// Registry composition obtains this value from its admitted snapshot.
    pub fn with_palette(mut self, palette: &crate::config::Palette) -> Self {
        self.palette = palette.clone();
        self
    }

    fn panel(&mut self, buffer: &Buffer, view: &View, panel: Panel) -> Result<(), Error> {
        let point = view.cell(buffer)?;
        let geometry = view.gutter_geometry(buffer, panel.content())?;
        let text = Panel::new(geometry.text(), false, panel.active());
        let fragments = panel_fragments(buffer, view, text)?;
        self.document(buffer, view, text, &fragments)?;
        self.numbers(
            buffer,
            view,
            geometry,
            gutter::NumberRows {
                current: point.row,
                fragments: &fragments,
            },
        )?;
        self.bookmark_markers(buffer, view, geometry, &fragments)?;
        self.status(buffer, point, panel)?;
        self.preview(buffer, view, text.content(), &fragments)?;
        if panel.active() {
            self.cursor = Self::position(point, view, text, buffer, &fragments);
        }
        self.record_source_rows(buffer, text, &fragments);
        Ok(())
    }

    fn record_source_rows(
        &mut self,
        buffer: &Buffer,
        panel: Panel,
        fragments: &[crate::view::Fragment],
    ) {
        if buffer.soft_wrap() {
            self.source_rows.push(SourceRows {
                area: panel.content(),
                stamp: buffer.stamp(),
                rows: fragments
                    .iter()
                    .map(|f| {
                        (
                            f.line,
                            f.columns.start,
                            f.highlight_width(panel.content().width()),
                        )
                    })
                    .collect(),
            });
        }
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
        self.write_with_separator(writer, crate::gutter::Separator::default())
    }

    /// Select the printable one-cell fallback when the frontend cannot display
    /// the primary separator. Frame cells and document coordinates stay fixed.
    pub fn write_with_separator(
        &self,
        writer: &mut impl Write,
        separator: crate::gutter::Separator,
    ) -> io::Result<()> {
        self.write_surface(writer, separator)?;
        self.finish(writer)
    }
    pub(crate) fn write_surface(
        &self,
        writer: &mut impl Write,
        separator: crate::gutter::Separator,
    ) -> io::Result<()> {
        writer.queue(Hide)?;
        for row in &self.rows {
            writer.queue(MoveTo(0, row.row))?;
            row.write(writer, &self.palette, separator)?;
        }
        Ok(())
    }
    pub(crate) fn finish(&self, writer: &mut impl Write) -> io::Result<()> {
        if let Some(cursor) = self.cursor {
            writer
                .queue(MoveTo(cursor.column, cursor.row))?
                .queue(Show)?;
        }
        writer.flush()
    }

    fn document(
        &mut self,
        buffer: &Buffer,
        view: &View,
        panel: Panel,
        fragments: &[crate::view::Fragment],
    ) -> Result<(), Error> {
        let highlight = selection::Highlight::new(buffer, view, panel.active())?;
        let coordinates = Coordinates::new(buffer)?;
        let content = panel.content();
        for row in 0..content.height() {
            let fragment = fragments.get(usize::from(row));
            remember_columns(buffer, fragment)?;
            let source = if buffer.soft_wrap() {
                fragment.map_or(buffer.len_lines(), |f| f.line)
            } else {
                view.source_row(buffer, usize::from(row))
                    .unwrap_or(buffer.len_lines())
            };
            let left = fragment.map_or(view.left, |f| f.columns.start);
            let painted = fragment.map_or(content.width(), |f| f.painted_width(content.width()));
            let text = document_text(
                buffer,
                &coordinates,
                fragment,
                Cell {
                    row: source,
                    column: left,
                },
                content.width(),
            )?;
            let projected = Row {
                row,
                text,
                syntax: syntax::row(buffer, left, source, painted)?,
                band: Band::Document,
                tone: Tone::Document,
                faces: Vec::new(),
                search: None,
                selection: highlight
                    .as_ref()
                    .map(|h| {
                        h.selected_row(
                            buffer,
                            source,
                            left,
                            fragment.map_or(painted, |f| f.highlight_width(content.width())),
                        )
                    })
                    .transpose()?
                    .flatten(),
            }
            .projected(panel);
            self.blit(content, projected)?;
        }
        Ok(())
    }

    fn status(&mut self, buffer: &Buffer, point: Cell, panel: Panel) -> Result<(), Error> {
        let Some(area) = panel.status() else {
            return Ok(());
        };
        let status = status::line(buffer, point, area.width(), panel.active())?;
        self.blit(
            area,
            Row {
                row: 0,
                text: status,
                syntax: Vec::new(),
                band: Band::Status,
                tone: Tone::Status(panel.active()),
                faces: Vec::new(),
                search: None,
                selection: None,
            },
        )
    }

    fn echo(&mut self, message: &str) -> Result<(), Error> {
        self.rows.push(Row {
            row: self.size.height - 1,
            text: clip::line(Glyphs::for_text(message)?, 0, self.size.width)?,
            syntax: Vec::new(),
            band: Band::Echo,
            tone: Tone::Message(Message::Normal),
            faces: Vec::new(),
            search: None,
            selection: None,
        });
        Ok(())
    }

    fn blank_rows(&mut self) {
        for row in 0..self.size.height - 1 {
            self.rows.push(Row {
                row,
                text: " ".repeat(usize::from(self.size.width)),
                syntax: Vec::new(),
                band: Band::Document,
                tone: Tone::Document,
                faces: Vec::new(),
                search: None,
                selection: None,
            });
        }
    }

    fn position(
        point: Cell,
        view: &View,
        panel: Panel,
        buffer: &Buffer,
        fragments: &[crate::view::Fragment],
    ) -> Option<Position> {
        if buffer.soft_wrap() {
            return wrapped_position(point, view, panel, fragments);
        }
        panel.position(Position {
            column: u16::try_from(point.column.checked_sub(view.left)?).ok()?,
            row: u16::try_from(view.screen_row(buffer, point.row)?).ok()?,
        })
    }
}

fn wrapped_position(
    point: Cell,
    view: &View,
    panel: Panel,
    fragments: &[crate::view::Fragment],
) -> Option<Position> {
    let content = panel.content();
    let (row, fragment) = fragments
        .iter()
        .enumerate()
        .find(|(_, f)| f.line == point.row && f.contains(view.point.position, content.width()))?;
    panel.position(Position {
        column: u16::try_from(point.column.checked_sub(fragment.columns.start)?).ok()?,
        row: u16::try_from(row).ok()?,
    })
}
fn document_text(
    buffer: &Buffer,
    coordinates: &Coordinates<'_>,
    fragment: Option<&crate::view::Fragment>,
    from: Cell,
    width: u16,
) -> Result<String, Error> {
    if let Some(fragment) = fragment {
        return Ok(fragment.text.clone());
    }
    if from.row < buffer.len_lines() {
        return Ok(clip::line(
            coordinates.glyphs_from_column(from)?,
            from.column,
            width,
        )?);
    }
    Ok(" ".repeat(usize::from(width)))
}

fn panel_fragments(
    buffer: &Buffer,
    view: &View,
    text: Panel,
) -> Result<Vec<crate::view::Fragment>, Error> {
    Ok(if buffer.soft_wrap() {
        view.wrapped_rows(buffer, text.content().width(), text.content().height())?
    } else {
        Vec::new()
    })
}

fn remember_columns(
    buffer: &Buffer,
    fragment: Option<&crate::view::Fragment>,
) -> Result<(), Error> {
    if let Some(fragment) = fragment {
        fragment.remember_columns(buffer)?;
    }
    Ok(())
}
