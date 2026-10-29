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
                    display
                        .result
                        .view
                        .as_ref()
                        .map_or(display.result.query.kind.label(), |v| v.title.as_str()),
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
                1 => query_description(display.result),
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
            row_text(result, row),
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

fn query_description(result: &QueryResult) -> String {
    match &result.view {
        Some(view) => format!(
            "Blocks: {}; / filter; | reset; S sort; V columns; g refresh",
            view.blocks
                .iter()
                .enumerate()
                .map(|(i, b)| format!(
                    "{}:{:?}({})",
                    i + 1,
                    b.kind,
                    result.rows.iter().filter(|r| r.block_index == i).count()
                ))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        None => format!("Query: {}", result.query.source),
    }
}
fn row_text(result: &QueryResult, row: &crate::org::agenda::QueryRow) -> String {
    let Some(view) = &result.view else {
        return row.title.to_string();
    };
    let columns = &view.blocks[row.block_index].columns;
    let values = columns
        .iter()
        .map(|key| {
            format!(
                "{key}={}",
                row.fields
                    .get(&key.to_ascii_uppercase())
                    .map_or("", String::as_str)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        "[{}] {}{}{}",
        row.block.as_deref().unwrap_or(""),
        row.title,
        if values.is_empty() { "" } else { " | " },
        values
    )
}
