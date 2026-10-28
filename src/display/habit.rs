use super::*;
use crate::org::agenda::HabitGraph;

impl Frame {
    /// Draw one read-only habit graph inside an existing pane row. Whole title
    /// graphemes and graph cells remain bounded; narrow panes keep today visible.
    pub fn habit_row(&mut self, graph: &HabitGraph, title: &str, area: Area) -> Result<(), Error> {
        let width = usize::from(area.width());
        let count = graph.cells.len().min(width);
        let today = graph.cells.iter().position(|c| c.today).unwrap_or(0);
        let first = today
            .saturating_sub(count / 2)
            .min(graph.cells.len() - count);
        let column = graph.column.min(width - count);
        let mut text = clip::line(Glyphs::for_text(title)?, 0, column as u16)?;
        let mut faces = Vec::new();
        for (i, cell) in graph.cells[first..first + count].iter().enumerate() {
            text.push(cell.glyph());
            style::append(
                &mut faces,
                area.column() + (column + i) as u16,
                Tone::Habit(cell.state, cell.subdued()),
            );
        }
        text.push_str(&" ".repeat(width - column - count));
        self.clear_projection(area);
        self.blit(
            area,
            Row {
                row: 0,
                text,
                band: Band::Document,
                tone: Tone::Document,
                faces,
                syntax: Vec::new(),
                selection: None,
                search: None,
            },
        )?;
        self.cursor = None;
        Ok(())
    }
}

impl Frame {
    pub(super) fn clear_projection(&mut self, area: Area) {
        let Some(row) = self.rows.get_mut(usize::from(area.row())) else {
            return;
        };
        let overlaps = |r: &std::ops::Range<u16>| {
            r.start < area.column() + area.width() && r.end > area.column()
        };
        row.syntax.retain(|p| !overlaps(&p.columns));
        row.selection = row.selection.take().filter(|r| !overlaps(r));
        row.search = row.search.take().filter(|r| !overlaps(r));
    }
}

pub struct HabitDisplay<'a> {
    pub agenda: &'a crate::org::agenda::HabitAgenda,
    pub policy: &'a crate::config::Habit,
    pub date: crate::org::time::Date,
    pub shown: bool,
    pub top: usize,
    pub stale: bool,
}
impl Frame {
    pub fn habit_agenda(&mut self, display: HabitDisplay<'_>, area: Area) -> Result<(), Error> {
        let label = format!(
            "Habit agenda {} [read-only]{}",
            display.date,
            if display.stale {
                " [stale: g refresh]"
            } else {
                ""
            }
        );
        for row in 0..area.height() {
            let line = Area::new(area.column(), area.row() + row, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            self.projection_text(if row == 0 { &label } else { "" }, line)?;
        }
        for (i, row) in display
            .agenda
            .visible(display.date, display.policy, display.shown)
            .skip(display.top)
            .take(usize::from(area.height().saturating_sub(1)))
            .enumerate()
        {
            let line = Area::new(area.column(), area.row() + 1 + i as u16, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            let title = if row.graph.unresolved == 0 {
                row.title.clone()
            } else {
                format!("{} [unresolved {}]", row.title, row.graph.unresolved)
            };
            self.habit_row(&row.graph, &title, line)?;
        }
        self.cursor = None;
        Ok(())
    }
    pub(super) fn projection_text(&mut self, text: &str, area: Area) -> Result<(), Error> {
        self.clear_projection(area);
        let text = clip::line(Glyphs::for_text(text)?, 0, area.width())?;
        self.blit(
            area,
            Row {
                row: 0,
                text,
                band: Band::Document,
                tone: Tone::Document,
                faces: Vec::new(),
                syntax: Vec::new(),
                selection: None,
                search: None,
            },
        )
    }
}
