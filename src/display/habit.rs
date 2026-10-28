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
                Tone::Habit(cell.state, cell.future),
            );
        }
        text.push_str(&" ".repeat(width - column - count));
        self.habit_clear(area);
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
    fn habit_clear(&mut self, area: Area) {
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
