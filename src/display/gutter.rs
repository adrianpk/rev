use crate::buffer::Buffer;
use crate::gutter::{Geometry, NumberRow, Role, Settings, Side};
use crate::view::View;
use crate::window::Area;

use super::{Band, Error, Face, Frame, Row, Tone};

impl Frame {
    pub(super) fn bookmark_markers(
        &mut self,
        buffer: &Buffer,
        view: &View,
        geometry: Geometry,
        fragments: &[crate::view::Fragment],
    ) -> Result<(), Error> {
        let Some(area) = geometry.marker() else {
            return Ok(());
        };
        let markers = buffer.bookmark_rows()?;
        for row in 0..area.height() {
            let source = source_row(buffer, view, row, fragments);
            self.blit(
                area,
                Row {
                    row,
                    text: if source.is_some_and(|line| markers.contains(&line)) {
                        view.bookmark_marker().into()
                    } else {
                        " ".into()
                    },
                    band: Band::Document,
                    tone: Tone::Gutter(Role::Bookmark),
                    faces: Vec::new(),
                    syntax: Vec::new(),
                    selection: None,
                    search: None,
                },
            )?;
        }
        Ok(())
    }

    pub(super) fn numbers(
        &mut self,
        buffer: &Buffer,
        view: &View,
        geometry: Geometry,
        current: usize,
        fragments: &[crate::view::Fragment],
    ) -> Result<(), Error> {
        let Some(area) = geometry.numbers() else {
            return Ok(());
        };
        let settings = view.gutter.effective();
        for row in 0..area.height() {
            let source = source_row(buffer, view, row, fragments);
            let label =
                source.and_then(|line| NumberRow::new(line, buffer.len_lines(), current, settings));
            let mut projected = field(area, geometry.digits(), settings, label)?;
            projected.row = row;
            self.blit(area, projected)?;
        }
        Ok(())
    }
}

fn source_row(
    buffer: &Buffer,
    view: &View,
    row: u16,
    fragments: &[crate::view::Fragment],
) -> Option<usize> {
    if buffer.soft_wrap() {
        fragments
            .get(usize::from(row))
            .filter(|fragment| fragment.columns.start == 0)
            .map(|fragment| fragment.line)
    } else {
        view.source_row(buffer, usize::from(row))
    }
}

fn field(
    area: Area,
    digits: u8,
    settings: Settings,
    label: Option<NumberRow>,
) -> Result<Row, Error> {
    let format = settings.format;
    let value = label
        .and_then(|row| row.label)
        .map_or_else(String::new, |v| v.to_string());
    let align = usize::from(digits)
        .checked_sub(value.len())
        .ok_or(crate::gutter::Error::Overflow)?;
    let body = format!(
        "{}{}{value}{}",
        " ".repeat(usize::from(format.left_padding())),
        " ".repeat(align),
        " ".repeat(usize::from(format.right_padding()))
    );
    let text = match settings.side {
        Side::Left => format!("{body}│"),
        Side::Right => format!("│{body}"),
    };
    let mut faces = Vec::new();
    let separator = match settings.side {
        Side::Left => area.column() + area.width() - 1,
        Side::Right => area.column(),
    };
    faces.push(Face {
        columns: separator..separator + 1,
        tone: Tone::Gutter(Role::Separator),
    });
    if let Some(label) = label {
        let start = area.column()
            + u16::from(settings.side == Side::Right)
            + u16::from(format.left_padding());
        faces.push(Face {
            columns: start..start + u16::from(digits),
            tone: Tone::Gutter(label.role()),
        });
    }
    Ok(Row {
        row: 0,
        text,
        band: Band::Document,
        tone: Tone::Gutter(Role::Number),
        faces,
        syntax: Vec::new(),
        selection: None,
        search: None,
    })
}
