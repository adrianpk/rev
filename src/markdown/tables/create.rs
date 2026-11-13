use crate::buffer::edit::Point;

use super::*;

impl Observation {
    pub(crate) fn create_table(
        &self,
        buffer: &Buffer,
        point: Point,
        draft: &str,
    ) -> Result<Work, String> {
        self.blank_table_line(buffer, point)?;
        let (columns, body) = dimensions(draft)?;
        observe::dimensions(columns, body)?;
        let span = buffer
            .span(0..buffer.len_chars())
            .map_err(|e| e.to_string())?;
        let text = Rope::from(buffer.slice(&span).map_err(|e| e.to_string())?);
        let byte = text.char_to_byte(point.position.0);
        let line = text.byte_to_line(byte);
        let start = text.line_to_byte(line);
        let end = start + text.line(line).len_bytes();
        let raw = text.line(line).to_string();
        let ending = start + raw.trim_end_matches(['\r', '\n']).len()..end;
        let mut rows = Vec::with_capacity(body + 2);
        for index in 0..body + 2 {
            rows.push(Row {
                bytes: if index == 0 { start..end } else { end..end },
                prefix: start..ending.start,
                ending: ending.clone(),
                cells: Vec::new(),
            });
        }
        Ok(Work {
            stamp: self.stamp,
            text,
            table: Table {
                bytes: start..end,
                shape: Ok(Shape {
                    rows,
                    alignments: vec![Alignment::None; columns],
                }),
            },
            point: byte,
            tab: buffer.policy().tab_width,
            newline: buffer.newline(),
            limit: buffer.markdown_byte_limit(),
        })
    }

    pub(crate) fn blank_table_line(&self, buffer: &Buffer, point: Point) -> Result<(), String> {
        if self.stamp != point.stamp || self.stamp != buffer.stamp() {
            return Err("Markdown structure is not current".into());
        }
        if buffer.region(point).map_err(|e| e.to_string())?.is_some() {
            return Err("Create a Markdown table without an active region".into());
        }
        let span = buffer
            .span(0..buffer.len_chars())
            .map_err(|e| e.to_string())?;
        let text = buffer.slice(&span).map_err(|e| e.to_string())?;
        let byte = text.char_to_byte(point.position.0);
        let line = text.line(text.byte_to_line(byte)).to_string();
        let body = line.trim_end_matches(['\r', '\n']);
        self.admit_blank(byte, body)
    }

    fn admit_blank(&self, byte: usize, body: &str) -> Result<(), String> {
        let literal = self.nodes.iter().any(|node| {
            matches!(node.kind, super::super::Kind::LiteralBlock) && node.bytes.contains(&byte)
        });
        if literal || !body.bytes().all(|b| b == b' ') || body.len() > 3 {
            return Err("Create a Markdown table on a blank line outside literal code".into());
        }
        Ok(())
    }
}

impl Work {
    pub(super) fn admit_created(
        &self,
        layout: &super::layout::Layout,
        shape: &Shape,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        self.output_limit(layout.text.len())?;
        let mut source = String::new();
        for chunk in self.text.byte_slice(..self.table.bytes.start).chunks() {
            check(cancel)?;
            source.push_str(chunk);
        }
        source.push_str(&layout.text);
        for chunk in self.text.byte_slice(self.table.bytes.end..).chunks() {
            check(cancel)?;
            source.push_str(chunk);
        }
        let parsed = super::super::parse::observe(&source, cancel)
            .ok_or("Markdown table creation analysis is incomplete")?;
        let bytes = self.table.bytes.start..self.table.bytes.start + layout.text.len();
        if parsed
            .tables
            .iter()
            .any(|table| table.created_shape(&bytes, shape))
        {
            Ok(())
        } else {
            Err("Markdown table creation would enter a literal or unsupported context".into())
        }
    }
}

impl Table {
    fn created_shape(&self, bytes: &Range<usize>, expected: &Shape) -> bool {
        self.bytes.start >= bytes.start
            && self.bytes.end <= bytes.end
            && self.shape.as_ref().is_ok_and(|shape| {
                shape.alignments.len() == expected.alignments.len()
                    && shape.rows.len() == expected.rows.len()
            })
    }
}

fn dimensions(draft: &str) -> Result<(usize, usize), String> {
    if draft.len() > 32 {
        return Err("Markdown table dimensions exceed 32 bytes".into());
    }
    let (columns, body) = draft
        .trim()
        .split_once(['x', 'X'])
        .ok_or("Enter explicit columns x body rows, for example 2x1")?;
    let columns = columns
        .trim()
        .parse()
        .map_err(|_| "Invalid Markdown table column count")?;
    let body = body
        .trim()
        .parse()
        .map_err(|_| "Invalid Markdown table body-row count")?;
    Ok((columns, body))
}
