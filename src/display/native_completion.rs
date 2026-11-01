use crate::completion::{Candidate, Completion, Kind};
use crate::coordinates::Glyphs;
use crate::window::Area;

use super::{Band, Error, Frame, Position, Row, Tone, clip};

struct Entry {
    text: String,
    kind: Kind,
    icon: bool,
    detail: Option<std::ops::Range<u16>>,
}

impl Entry {
    fn new(candidate: &Candidate, width: u16) -> Result<Self, Error> {
        let mut entry = Self {
            text: String::new(),
            kind: candidate.kind(),
            icon: width >= 3,
            detail: None,
        };
        let available = width.saturating_sub(if entry.icon { 2 } else { 0 });
        let detail = candidate.detail();
        let detail_width = unicode_width::UnicodeWidthStr::width(detail) as u16;
        let name_width = unicode_width::UnicodeWidthStr::width(candidate.label()).min(8) as u16;
        let show = entry.icon && !detail.is_empty() && available >= detail_width + 2 + name_width;
        let label_width = available.saturating_sub(if show { detail_width + 2 } else { 0 });
        let label = clip::line(Glyphs::for_text(candidate.label())?, 0, label_width)?;
        if entry.icon {
            entry.text.push(entry.kind.indicator());
            entry.text.push(' ');
        }
        entry.text.push_str(&label);
        if show {
            entry.text.push_str("  ");
            entry.text.push_str(detail);
            entry.detail = Some(width - detail_width..width);
        }
        Ok(entry)
    }

    fn faces(&self, column: u16) -> Vec<super::Face> {
        let mut faces = Vec::new();
        if self.icon {
            faces.push(super::Face {
                columns: column..column + 1,
                tone: Tone::Completion(self.kind),
            });
        }
        if let Some(detail) = &self.detail {
            faces.push(super::Face {
                columns: column + detail.start..column + detail.end,
                tone: Tone::Completion(self.kind),
            });
        }
        faces
    }
}

impl Frame {
    pub(crate) fn native_completion(
        &mut self,
        completion: &Completion,
        pane: Area,
    ) -> Result<(), Error> {
        let Some(anchor) = self.cursor else {
            return Ok(());
        };
        let Some(area) = placement(completion, anchor, pane) else {
            return Ok(());
        };
        let rows = usize::from(area.height() - 2);
        let selected = completion.selected().unwrap_or(0);
        let top = selected.saturating_sub(rows - 1);
        for row in 0..area.height() {
            self.clear_projection(
                Area::new(area.column(), area.row() + row, area.width(), 1)
                    .expect("bounded popup row"),
            );
        }
        self.popup_border(area, 0, '┌', '┐')?;
        for row in 0..rows {
            let index = top + row;
            let candidate = completion
                .candidates()
                .get(index)
                .expect("popup rows fit the admitted candidate list");
            let entry = Entry::new(candidate, area.width() - 2)?;
            self.popup_row(area, row as u16 + 1, entry, index == selected)?;
        }
        self.popup_border(area, area.height() - 1, '└', '┘')
    }

    fn popup_border(&mut self, area: Area, row: u16, left: char, right: char) -> Result<(), Error> {
        self.blit(
            area,
            Row {
                row,
                text: format!("{left}{}{right}", "─".repeat(usize::from(area.width() - 2))),
                band: Band::Completion,
                tone: Tone::Border(true),
                faces: Vec::new(),
                syntax: Vec::new(),
                search: None,
                selection: None,
            },
        )
    }

    fn popup_row(
        &mut self,
        area: Area,
        row: u16,
        entry: Entry,
        selected: bool,
    ) -> Result<(), Error> {
        let mut faces = entry.faces(area.column() + 1);
        faces.extend([
            super::Face {
                columns: area.column()..area.column() + 1,
                tone: Tone::Border(true),
            },
            super::Face {
                columns: area.column() + area.width() - 1..area.column() + area.width(),
                tone: Tone::Border(true),
            },
        ]);
        self.blit(
            area,
            Row {
                row,
                text: format!("│{}│", entry.text),
                band: Band::Completion,
                tone: Tone::Document,
                faces,
                syntax: Vec::new(),
                search: None,
                selection: selected.then_some(area.column() + 1..area.column() + area.width() - 1),
            },
        )
    }
}

fn placement(completion: &Completion, anchor: Position, pane: Area) -> Option<Area> {
    pane.local(anchor)?;
    if pane.width() < 3 {
        return None;
    }
    let above = anchor.row - pane.row();
    let below = pane.row() + pane.height() - anchor.row - 1;
    let desired = completion.candidates().len().min(completion.popup_rows) as u16 + 2;
    let down = below >= desired || below >= above;
    let height = desired.min(if down { below } else { above });
    if height < 3 {
        return None;
    }
    let label_width = completion
        .candidates()
        .iter()
        .map(|candidate| unicode_width::UnicodeWidthStr::width(candidate.label()))
        .max()
        .unwrap_or(1);
    let detail_width = completion
        .candidates()
        .iter()
        .map(|candidate| unicode_width::UnicodeWidthStr::width(candidate.detail()))
        .max()
        .unwrap_or(0);
    let width = label_width
        .saturating_add(detail_width)
        .saturating_add(6)
        .min(usize::from(pane.width())) as u16;
    let column = anchor.column.min(pane.column() + pane.width() - width);
    let row = if down {
        anchor.row + 1
    } else {
        anchor.row - height
    };
    Area::new(column, row, width.max(3), height)
}
