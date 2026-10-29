use super::*;
use crate::org::agenda::QueryResult;
pub struct QueryDisplay<'a> {
    pub result: &'a QueryResult,
    pub top: usize,
    pub selected: usize,
    pub active: bool,
    pub stale: bool,
}
impl Frame {
    pub fn org_query(&mut self, display: QueryDisplay<'_>, area: Area) -> Result<(), Error> {
        for row in 0..area.height() {
            let line = Area::new(area.column(), area.row() + row, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            let text = match row {
                0 => format!(
                    "Org {} agenda [{}] {} [{}] [read-only]{}",
                    display.result.query.kind.label(),
                    if display.result.complete() {
                        "complete"
                    } else {
                        "incomplete"
                    },
                    display.result.context.today,
                    display.result.context.zone,
                    if display.stale {
                        " [stale: g refresh]"
                    } else {
                        ""
                    }
                ),
                1 => format!("Query: {}", display.result.query.source),
                _ => query_line(&display, display.top + usize::from(row - 2)),
            };
            self.projection_text(&text, line)?;
        }
        if display.active {
            self.cursor = None;
        }
        Ok(())
    }
}
fn query_line(display: &QueryDisplay<'_>, index: usize) -> String {
    let result = display.result;
    if let Some(row) = result.rows.get(index) {
        let name = result.sources[row.source]
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|p| p.to_string_lossy())
            .unwrap_or(std::borrow::Cow::Borrowed("buffer"));
        return format!(
            "{}{} {} ({name}){}",
            if index == display.selected {
                "> "
            } else {
                "  "
            },
            row.task.as_deref().unwrap_or(""),
            row.title,
            if row.stale { " [stale]" } else { "" }
        );
    }
    if let Some(issue) = result.issues.get(index.saturating_sub(result.rows.len())) {
        return format!("Incomplete {:?}: {}", issue.path, issue.message);
    }
    if result.rows.is_empty() && index == 0 {
        return "No matching entries".into();
    }
    String::new()
}
