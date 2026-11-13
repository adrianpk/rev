use super::*;

impl Table {
    pub(crate) fn observe(
        source: &str,
        bytes: Range<usize>,
        rows: &[Range<usize>],
        alignments: Vec<Alignment>,
    ) -> Self {
        let start = line_start(source, bytes.start);
        let end = line_end(source, bytes.end.saturating_sub(1));
        let shape = shape(source, rows, alignments);
        Self {
            bytes: start..end,
            shape,
        }
    }
}

fn shape(source: &str, rows: &[Range<usize>], alignments: Vec<Alignment>) -> Result<Shape, String> {
    dimensions(alignments.len(), rows.len().saturating_sub(1))?;
    let header = rows.first().ok_or("Markdown table has no header")?;
    let delimiter = line_end(source, header.start);
    let end = line_end(source, delimiter);
    let prefix = source[delimiter..end]
        .bytes()
        .take_while(|b| matches!(b, b' ' | b'\t' | b'>'))
        .count();
    let mut result = Vec::with_capacity(rows.len() + 1);
    result.push(Row::observe(source, header.start)?);
    result.push(Row::observe(source, delimiter + prefix)?);
    for row in rows.iter().skip(1) {
        result.push(Row::observe(source, row.start)?);
    }
    let columns = alignments.len();
    if result.iter().any(|row| row.cells.len() > columns) {
        return Err("Markdown table has extra cells; alignment would discard source".into());
    }
    if result[0].cells.len() != columns || result[1].cells.len() != columns {
        return Err("Markdown table header and delimiter do not match".into());
    }
    Ok(Shape {
        rows: result,
        alignments,
    })
}

pub(super) fn dimensions(columns: usize, body: usize) -> Result<(), String> {
    let rows = body
        .checked_add(2)
        .ok_or("Markdown table dimensions overflow")?;
    if columns == 0 || columns > COLUMNS || rows > ROWS {
        return Err("Markdown tables require 1..256 columns and at most 10000 total rows".into());
    }
    if rows.checked_mul(columns).is_none_or(|count| count > CELLS) {
        return Err("Markdown table exceeds 100000 cells".into());
    }
    Ok(())
}

impl Row {
    fn observe(source: &str, content: usize) -> Result<Self, String> {
        let start = line_start(source, content);
        let end = line_end(source, content);
        let body = source[start..end].trim_end_matches(['\r', '\n']).len() + start;
        let indent = source[content..body]
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .count();
        let content = content + indent;
        Ok(Self {
            bytes: start..end,
            prefix: start..content,
            ending: body..end,
            cells: cells(source, content..body)?,
        })
    }
}

fn cells(source: &str, bytes: Range<usize>) -> Result<Vec<Range<usize>>, String> {
    let mut cells = Vec::new();
    let mut start = bytes.start;
    for at in bytes.clone() {
        let byte = source.as_bytes()[at];
        if byte == b'|' && (at == bytes.start || source.as_bytes()[at - 1] != b'\\') {
            cells.push(trim(source, start..at));
            start = at + 1;
        }
        if cells.len() > COLUMNS + 1 {
            return Err("Markdown table exceeds 256 columns".into());
        }
    }
    cells.push(trim(source, start..bytes.end));
    if source[bytes.clone()].starts_with('|') {
        cells.remove(0);
    }
    if start == bytes.end
        || source[start..bytes.end]
            .bytes()
            .all(|b| matches!(b, b' ' | b'\t'))
            && start > bytes.start
    {
        cells.pop();
    }
    Ok(cells)
}

fn trim(source: &str, bytes: Range<usize>) -> Range<usize> {
    let value = &source[bytes.clone()];
    let left = value.len() - value.trim_start_matches([' ', '\t']).len();
    let len = value.trim_matches([' ', '\t']).len();
    bytes.start + left..bytes.start + left + len
}

fn line_start(source: &str, byte: usize) -> usize {
    source[..byte].rfind('\n').map_or(0, |at| at + 1)
}

fn line_end(source: &str, byte: usize) -> usize {
    source[byte..]
        .find('\n')
        .map_or(source.len(), |at| byte + at + 1)
}
