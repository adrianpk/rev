use crate::completion::Completion;
use crate::coordinates::Glyphs;
use crate::window::Area;

use super::{Band, Error, Frame, Position, Row, Tone, clip};

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
            let label = completion
                .candidates()
                .get(index)
                .map_or("", |candidate| candidate.label());
            let text = clip::line(Glyphs::for_text(label)?, 0, area.width() - 2)?;
            self.popup_row(area, row as u16 + 1, text, index == selected)?;
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
        text: String,
        selected: bool,
    ) -> Result<(), Error> {
        self.blit(
            area,
            Row {
                row,
                text: format!("│{text}│"),
                band: Band::Completion,
                tone: Tone::Document,
                faces: vec![
                    super::Face {
                        columns: area.column()..area.column() + 1,
                        tone: Tone::Border(true),
                    },
                    super::Face {
                        columns: area.column() + area.width() - 1..area.column() + area.width(),
                        tone: Tone::Border(true),
                    },
                ],
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
    let width = completion
        .candidates()
        .iter()
        .map(|candidate| unicode_width::UnicodeWidthStr::width(candidate.label()))
        .max()
        .unwrap_or(1)
        .saturating_add(2)
        .min(usize::from(pane.width())) as u16;
    let column = anchor.column.min(pane.column() + pane.width() - width);
    let row = if down {
        anchor.row + 1
    } else {
        anchor.row - height
    };
    Area::new(column, row, width.max(3), height)
}
