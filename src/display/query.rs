use super::*;
use crate::org::agenda::QueryResult;
pub struct QueryDisplay<'a> {
    pub result: &'a QueryResult,
    pub top: usize,
    pub selected: usize,
    pub active: bool,
    pub columns: bool,
    pub stale: bool,
    pub marked: &'a [crate::org::agenda::ActionTarget],
}
impl Frame {
    pub fn org_query(&mut self, display: QueryDisplay<'_>, area: Area) -> Result<(), Error> {
        for row in 0..area.height() {
            let line = Area::new(area.column(), area.row() + row, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            let text = match row {
                0 => query_header(&display),
                1 if display.columns => {
                    "Columns from source; Enter visits; g refreshes; q closes".into()
                }
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
    if let Some(crate::org::agenda::QueryLine::Block(block)) = result.lines.get(index) {
        return block_heading(result, *block);
    }
    if let Some(row) = result.row_at(index) {
        let name = result.sources[row.source]
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|p| p.to_string_lossy())
            .unwrap_or(std::borrow::Cow::Borrowed("buffer"));
        return format!(
            "{}{}{} {} ({name}){}",
            if index == display.selected {
                "> "
            } else {
                "  "
            },
            if display.marked.iter().any(|t| t.byte == row.byte
                && t.source.version == result.sources[row.source].version
                && t.source.path == result.sources[row.source].path)
            {
                "* "
            } else {
                ""
            },
            row.task.as_deref().unwrap_or(""),
            row_text(result, row),
            if row.stale { " [stale]" } else { "" }
        );
    }
    if let Some(issue) = result.issues.get(index.saturating_sub(result.line_count())) {
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

fn query_dates(result: &QueryResult) -> String {
    if result.view.is_some() {
        format!(
            "{}..{}",
            result.context.start,
            result
                .context
                .end()
                .map(|d| d.to_string())
                .unwrap_or_else(|e| e)
        )
    } else {
        result.context.today.to_string()
    }
}
fn block_heading(result: &QueryResult, index: usize) -> String {
    let block = &result.view.as_ref().expect("block view").blocks[index];
    format!(
        "Block {} {:?}: {} [{} entries]",
        index + 1,
        block.kind,
        block.title,
        result
            .rows
            .iter()
            .filter(|r| r.block_index == index)
            .count()
    )
}

fn query_header(display: &QueryDisplay<'_>) -> String {
    if display.columns {
        return format!(
            "Org property columns [{}]{} [all recorded clocks] [{}] [read-only]",
            if display.result.complete() {
                "complete"
            } else {
                "incomplete"
            },
            if display.stale {
                " [stale: g refresh]"
            } else {
                ""
            },
            display.result.context.zone
        );
    }
    format!(
        "Org {} agenda [{}]{} {} [{}] [read-only]",
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
        if display.stale {
            " [stale: g refresh]"
        } else {
            ""
        },
        query_dates(display.result),
        display.result.context.zone,
    )
}
