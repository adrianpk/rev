use crate::buffer::edit::{Replacement, Retained};
use crate::coordinates::Glyphs;

use super::*;

pub(super) struct Layout {
    pub(super) text: String,
    pub(super) positions: Vec<Vec<Range<usize>>>,
    anchors: Vec<Anchor>,
}

struct Anchor {
    source: Range<usize>,
    output: Range<usize>,
}

struct CellLayout {
    index: usize,
    rule: bool,
    width: usize,
    alignment: Alignment,
    column: usize,
}

impl Work {
    pub(super) fn layout(&self, shape: &Shape, cancel: &Cancellation) -> Result<Layout, String> {
        let widths = self.widths(shape, cancel)?;
        let mut layout = Layout {
            text: String::new(),
            positions: Vec::new(),
            anchors: Vec::new(),
        };
        for (index, row) in shape.rows.iter().enumerate() {
            check(cancel)?;
            let prefix = self.text.byte_slice(row.prefix.clone()).to_string();
            layout.copy(self, row.prefix.clone(), !row.bytes.is_empty());
            layout.text.push('|');
            let mut column = measure(&prefix, 0, self.tab, cancel)? + 2;
            let mut positions = Vec::with_capacity(widths.len());
            for (cell, width) in widths.iter().copied().enumerate() {
                let spec = CellLayout {
                    index: cell,
                    rule: index == 1,
                    width,
                    alignment: shape.alignments[cell],
                    column,
                };
                let range = self.render_cell(&mut layout, row, &spec, cancel)?;
                positions.push(range);
                column += width + 3;
            }
            layout.positions.push(positions);
            layout.copy(self, row.ending.clone(), !row.bytes.is_empty());
            if index + 1 < shape.rows.len() && row.ending.is_empty() {
                layout.text.push_str(self.newline);
            }
            if layout.text.len() > super::super::MAX_BYTES {
                return Err("Markdown table exceeds 8 MiB output limit".into());
            }
        }
        Ok(layout)
    }

    fn widths(&self, shape: &Shape, cancel: &Cancellation) -> Result<Vec<usize>, String> {
        let mut widths = Vec::with_capacity(shape.alignments.len());
        let mut prior = 0;
        for alignment in &shape.alignments {
            let mut width = minimum(*alignment);
            for row in shape
                .rows
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != 1)
                .map(|(_, row)| row)
            {
                check(cancel)?;
                let prefix = self.text.byte_slice(row.prefix.clone()).to_string();
                let column = measure(&prefix, 0, self.tab, cancel)? + prior + 2;
                let value = row
                    .cells
                    .get(widths.len())
                    .map_or_else(String::new, |bytes| {
                        self.text.byte_slice(bytes.clone()).to_string()
                    });
                width = width.max(measure_cell(&value, column, self.tab, cancel)?);
            }
            widths.push(width);
            prior += width + 3;
        }
        Ok(widths)
    }

    fn render_cell(
        &self,
        layout: &mut Layout,
        row: &Row,
        spec: &CellLayout,
        cancel: &Cancellation,
    ) -> Result<Range<usize>, String> {
        let CellLayout {
            index: cell,
            rule,
            width,
            alignment,
            column,
        } = *spec;
        layout.text.push(' ');
        if rule {
            let start = layout.text.len();
            delimiter(&mut layout.text, width, alignment);
            let range = start..layout.text.len();
            layout.text.push_str(" |");
            return Ok(range);
        }
        let bytes = row
            .cells
            .get(cell)
            .cloned()
            .unwrap_or(row.ending.start..row.ending.start);
        let value = self.text.byte_slice(bytes.clone()).to_string();
        let (left, right) = padding(&value, (column, width, alignment, self.tab), cancel)?;
        layout.text.push_str(&" ".repeat(left));
        let start = layout.text.len();
        layout.copy(self, bytes, true);
        let range = start..layout.text.len();
        layout.text.push_str(&" ".repeat(right));
        layout.text.push_str(" |");
        Ok(range)
    }

    pub(super) fn output_limit(&self, bytes: usize) -> Result<(), String> {
        if self.text.len_bytes() - self.table.bytes.len() + bytes > self.limit {
            Err("Markdown table exceeds source/output byte limit".into())
        } else {
            Ok(())
        }
    }

    pub(super) fn plan(
        &self,
        layout: Layout,
        cursor: (usize, usize, usize),
    ) -> Result<Edit, String> {
        self.output_limit(layout.text.len())?;
        let cell = layout
            .positions
            .get(cursor.0)
            .and_then(|row| row.get(cursor.1))
            .ok_or("Markdown table destination has no cell")?;
        let offset = layout.text[cell.clone()]
            .char_indices()
            .nth(cursor.2)
            .map_or(cell.end, |(byte, _)| cell.start + byte);
        let local = boundary(&layout.text, offset)?;
        let start = self.text.byte_to_char(self.table.bytes.start);
        let retained = layout.retained(self);
        let edits = if self.text.byte_slice(self.table.bytes.clone()) == layout.text {
            Vec::new()
        } else {
            vec![Replacement {
                source: self.span(self.table.bytes.clone()),
                text: layout.text,
            }]
        };
        Ok(Edit {
            edits,
            retained,
            cursor: Some(Scalar(start + local)),
        })
    }
}

impl Layout {
    pub(super) fn isolate(&mut self, work: &Work) {
        let row = work.text.byte_to_line(work.table.bytes.start);
        let before = row > 0 && !work.text.line(row - 1).to_string().trim().is_empty();
        let after = work.table.bytes.end < work.text.len_bytes()
            && !work
                .text
                .line(work.text.byte_to_line(work.table.bytes.end))
                .to_string()
                .trim()
                .is_empty();
        if before {
            self.text.insert_str(0, work.newline);
            for range in self.positions.iter_mut().flatten() {
                range.start += work.newline.len();
                range.end += work.newline.len();
            }
            for anchor in &mut self.anchors {
                anchor.output.start += work.newline.len();
                anchor.output.end += work.newline.len();
            }
        }
        if after {
            self.text.push_str(work.newline);
        }
    }

    fn copy(&mut self, work: &Work, bytes: Range<usize>, retain: bool) {
        let start = self.text.len();
        let value = work.text.byte_slice(bytes.clone());
        for chunk in value.chunks() {
            self.text.push_str(chunk);
        }
        if retain && !bytes.is_empty() {
            self.anchors.push(Anchor {
                source: bytes,
                output: start..self.text.len(),
            });
        }
    }

    fn retained(&self, work: &Work) -> Vec<Retained> {
        let mut byte = 0;
        let mut scalar = work.text.byte_to_char(work.table.bytes.start);
        self.anchors
            .iter()
            .map(|anchor| {
                scalar += self.text[byte..anchor.output.start].chars().count();
                let start = scalar;
                scalar += self.text[anchor.output.clone()].chars().count();
                byte = anchor.output.end;
                Retained {
                    source: work.span(anchor.source.clone()),
                    output: start..scalar,
                }
            })
            .collect()
    }
}

fn minimum(alignment: Alignment) -> usize {
    3 + usize::from(matches!(alignment, Alignment::Left | Alignment::Center))
        + usize::from(matches!(alignment, Alignment::Right | Alignment::Center))
}

fn delimiter(text: &mut String, width: usize, alignment: Alignment) {
    let left = matches!(alignment, Alignment::Left | Alignment::Center);
    let right = matches!(alignment, Alignment::Right | Alignment::Center);
    if left {
        text.push(':');
    }
    text.push_str(&"-".repeat(width - usize::from(left) - usize::from(right)));
    if right {
        text.push(':');
    }
}

fn padding(
    value: &str,
    spec: (usize, usize, Alignment, usize),
    cancel: &Cancellation,
) -> Result<(usize, usize), String> {
    let (column, width, alignment, tab) = spec;
    let measured = measure_cell(value, column, tab, cancel)?;
    let mut left = match alignment {
        Alignment::None | Alignment::Left => 0,
        Alignment::Center => (width - measured) / 2,
        Alignment::Right => width - measured,
    };
    while left + measure_cell(value, column + left, tab, cancel)? > width {
        left -= 1;
    }
    let right = width - left - measure_cell(value, column + left, tab, cancel)?;
    Ok((left, right))
}

fn measure_cell(
    value: &str,
    column: usize,
    tab: usize,
    cancel: &Cancellation,
) -> Result<usize, String> {
    let contextual = format!(" {value}");
    measure(&contextual, column - 1, tab, cancel).map(|width| width - 1)
}

fn measure(value: &str, column: usize, tab: usize, cancel: &Cancellation) -> Result<usize, String> {
    let glyphs = Glyphs::for_columns(value, column, tab).map_err(|e| e.to_string())?;
    let mut end = column;
    for glyph in glyphs {
        check(cancel)?;
        end = glyph.map_err(|e| e.to_string())?.columns.end;
        if end - column > WIDTH + 1 {
            return Err("Markdown cell exceeds 4096 displayed columns".into());
        }
    }
    Ok(end - column)
}

fn boundary(text: &str, byte: usize) -> Result<usize, String> {
    let start = text[..byte].rfind('\n').map_or(0, |at| at + 1);
    let end = text[byte..].find('\n').map_or(text.len(), |at| byte + at);
    let offset = text[start..byte].chars().count();
    let prefix = text[..start].chars().count();
    for glyph in Glyphs::for_text(&text[start..end]).map_err(|e| e.to_string())? {
        let glyph = glyph.map_err(|e| e.to_string())?;
        if offset < glyph.source.end.0 {
            return Ok(prefix + glyph.source.start.0);
        }
    }
    Ok(prefix + offset)
}
