use super::*;
use crate::bookmark::{RecordId, Resolution, list::Row as BookmarkRow};
use std::collections::BTreeSet;

pub(crate) struct Display<'a> {
    pub rows: &'a [BookmarkRow],
    pub cursor: usize,
    pub top: usize,
    pub marked: &'a BTreeSet<RecordId>,
    pub stale: bool,
}
impl Frame {
    /// Only visible list rows are formatted; no target source is observed here.
    pub(crate) fn bookmarks(&mut self, display: Display<'_>, panel: Panel) -> Result<(), Error> {
        let area = panel.content();
        let extent = area.height().saturating_sub(1).max(1) as usize;
        let top = display
            .top
            .min(display.cursor)
            .max(display.cursor.saturating_sub(extent - 1));
        for offset in 0..area.height() {
            let line = Area::new(area.column(), area.row() + offset, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            let text = if offset == 0 {
                format!(
                    "*Bookmarks* [read-only] {} entries{}",
                    display.rows.len(),
                    if display.stale {
                        " [changed; refresh on action]"
                    } else {
                        ""
                    }
                )
            } else {
                let index = top + usize::from(offset - 1);
                display.rows.get(index).map_or_else(String::new, |row| {
                    let status = match row.resolution {
                        Resolution::Resolved => "resolved",
                        Resolution::Unresolved => "unresolved",
                        Resolution::Stale => "stale",
                        Resolution::Unavailable => "unavailable",
                    };
                    format!(
                        "{}{} {:?} [{status}] {:?}",
                        if index == display.cursor { ">" } else { " " },
                        if display.marked.contains(&row.id) {
                            "D"
                        } else {
                            " "
                        },
                        row.name,
                        row.path
                    )
                })
            };
            self.projection_text(&text, line)?;
        }
        if let Some(status) = panel.status() {
            self.clear_projection(status);
            let text = clip::line(
                Glyphs::for_text("*Bookmarks*  RET visit  d/u mark/unmark  x delete  q return")?,
                0,
                status.width(),
            )?;
            self.blit(
                status,
                Row {
                    row: 0,
                    text,
                    band: Band::Status,
                    tone: Tone::Status(panel.active()),
                    faces: Vec::new(),
                    syntax: Vec::new(),
                    selection: None,
                    search: None,
                },
            )?;
        }
        if panel.active() {
            self.cursor = None;
        }
        Ok(())
    }
}
