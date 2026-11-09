mod artwork;

use super::{Error, Face, Frame, Position, Tone, clip};
use crate::coordinates::Glyphs;
use crate::view::Size;

const LABELS: [&str; 4] = ["New Buffer", "Open…", "Open Project", "Open Workspace"];
const ACTION_WIDTH: u16 = 16;
const MENU_GAP: u16 = 3;

/// One disposable placement shared by painting and pointer admission. No banner
/// row, offscreen action or stale resize geometry can become a source text hit.
pub(crate) struct Layout {
    full: bool,
    title_row: Option<u16>,
    banner_column: u16,
    list_row: u16,
    list_column: u16,
    width: u16,
    rows: u16,
    first: usize,
}
impl Layout {
    pub(crate) fn new(size: Size, selected: usize, color: bool) -> Self {
        let height = if size.height > 1 {
            size.height - 1
        } else {
            size.height
        };
        let actions = u16::try_from(LABELS.len()).expect("four startup actions fit terminal rows");
        let full = color && size.width >= artwork::WIDTH && height > artwork::HEIGHT + actions;
        let (title, gap, rows) = if full {
            (
                artwork::HEIGHT,
                (height - artwork::HEIGHT - actions).min(MENU_GAP),
                actions,
            )
        } else if height >= actions + 2 {
            (1, (height - actions - 1).min(MENU_GAP), actions)
        } else if height >= 2 {
            (1, 0, (height - 1).min(actions))
        } else {
            (0, 0, height)
        };
        let group = title + gap + rows;
        let top = height.saturating_sub(group) / 2;
        let width = ACTION_WIDTH.min(size.width);
        Self {
            full,
            title_row: (title > 0).then_some(top),
            banner_column: size.width.saturating_sub(artwork::WIDTH) / 2,
            list_row: top + title + gap,
            list_column: size.width.saturating_sub(width) / 2,
            width,
            rows,
            first: selected.saturating_sub(usize::from(rows.saturating_sub(1))),
        }
    }
    pub(crate) fn hit(&self, position: Position) -> Option<usize> {
        if position.column < self.list_column
            || position.column >= self.list_column + self.width
            || position.row < self.list_row
            || position.row >= self.list_row + self.rows
        {
            return None;
        }
        let index = self.first + usize::from(position.row - self.list_row);
        (index < LABELS.len()).then_some(index)
    }
}
impl Frame {
    pub(crate) fn startup(
        &mut self,
        selected: usize,
        color: bool,
        notice: &str,
    ) -> Result<(), Error> {
        let layout = Layout::new(self.size, selected, color);
        self.cursor = None;
        self.cursor_style = None;
        self.previews.clear();
        self.source_rows.clear();
        self.terminal_rows.clear();
        self.rows.clear();
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }
        self.blank_rows();
        self.echo(notice)?;
        for row in self
            .rows
            .iter_mut()
            .take(usize::from(self.size.height.saturating_sub(1)))
        {
            row.tone = Tone::Startup;
        }
        self.startup_title(&layout)?;
        self.startup_actions(&layout, selected)
    }

    fn startup_actions(&mut self, layout: &Layout, selected: usize) -> Result<(), Error> {
        for offset in 0..layout.rows {
            let index = layout.first + usize::from(offset);
            let Some(label) = LABELS.get(index) else {
                break;
            };
            let row = &mut self.rows[usize::from(layout.list_row + offset)];
            // At one row the visible action replaces the echo entirely.
            row.band = super::Band::Document;
            row.tone = Tone::Startup;
            let marker = if selected == index { "> " } else { "  " };
            let text = clip::line(
                Glyphs::for_text(&format!("{marker}{label}"))?,
                0,
                layout.width,
            )?;
            row.text = format!(
                "{}{}{}",
                " ".repeat(usize::from(layout.list_column)),
                text,
                " ".repeat(usize::from(
                    self.size.width - layout.list_column - layout.width
                ))
            );
            row.selection = (index == selected)
                .then_some(layout.list_column..layout.list_column + layout.width);
        }
        Ok(())
    }
    fn startup_title(&mut self, layout: &Layout) -> Result<(), Error> {
        if let Some(top) = layout.title_row {
            if layout.full {
                for (offset, source) in artwork::get().iter().enumerate() {
                    let row = &mut self.rows[usize::from(top) + offset];
                    row.text = format!(
                        "{}{}{}",
                        " ".repeat(usize::from(layout.banner_column)),
                        source.text,
                        " ".repeat(usize::from(
                            self.size.width - layout.banner_column - artwork::WIDTH
                        ))
                    );
                    row.faces = source
                        .runs
                        .iter()
                        .map(|r| Face {
                            columns: layout.banner_column + r.columns.start
                                ..layout.banner_column + r.columns.end,
                            tone: Tone::Artwork(r.foreground, r.background),
                        })
                        .collect();
                }
            } else {
                let column = self.size.width.saturating_sub(3) / 2;
                let row = &mut self.rows[usize::from(top)];
                row.text = clip::line(
                    Glyphs::for_text(&format!("{}rev", " ".repeat(usize::from(column))))?,
                    0,
                    self.size.width,
                )?;
            }
        }
        Ok(())
    }
}
