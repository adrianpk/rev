//! GFM source shapes and bounded source plans. Parser admission is mandatory;
//! separator scanning only retains original cells and detects lossy body rows.
mod action;
mod create;
mod layout;
mod observe;
#[cfg(test)]
mod tests;

use std::ops::Range;

use crate::buffer::{Buffer, Span, Stamp, edit::Point};
use crate::coordinates::Scalar;
use crate::process::Cancellation;
use ropey::Rope;

use super::{Edit, Observation};

const ROWS: usize = 10_000;
const COLUMNS: usize = 256;
const CELLS: usize = 100_000;
const WIDTH: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Alignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone)]
pub(crate) struct Table {
    bytes: Range<usize>,
    shape: Result<Shape, String>,
}

#[derive(Clone)]
struct Shape {
    rows: Vec<Row>,
    alignments: Vec<Alignment>,
}

#[derive(Clone)]
struct Row {
    bytes: Range<usize>,
    prefix: Range<usize>,
    ending: Range<usize>,
    cells: Vec<Range<usize>>,
}

pub(crate) struct Work {
    stamp: Stamp,
    text: Rope,
    table: Table,
    point: usize,
    tab: usize,
    newline: &'static str,
    limit: usize,
}

impl Observation {
    pub(crate) fn table_at(&self, buffer: &Buffer, point: Point) -> bool {
        if self.stamp != point.stamp || self.stamp != buffer.stamp() {
            return false;
        }
        let Ok(span) = buffer.span(0..buffer.len_chars()) else {
            return false;
        };
        let Ok(text) = buffer.slice(&span) else {
            return false;
        };
        let byte = text.char_to_byte(point.position.0);
        self.tables.iter().any(|table| table.contains(byte, &text))
    }

    pub(crate) fn table_work(&self, buffer: &Buffer, point: Point) -> Result<Option<Work>, String> {
        if self.stamp != point.stamp || self.stamp != buffer.stamp() {
            return Err("Markdown table structure is not current".into());
        }
        let text = buffer
            .slice(
                &buffer
                    .span(0..buffer.len_chars())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        let byte = text.char_to_byte(point.position.0);
        let Some(table) = self.tables.iter().find(|table| table.contains(byte, &text)) else {
            return Ok(None);
        };
        Ok(Some(Work {
            stamp: self.stamp,
            text: Rope::from(text),
            table: table.clone(),
            point: byte,
            tab: buffer.policy().tab_width,
            newline: buffer.newline(),
            limit: buffer.markdown_byte_limit(),
        }))
    }
}

impl Table {
    fn contains(&self, byte: usize, text: &ropey::RopeSlice<'_>) -> bool {
        self.bytes.contains(&byte)
            || byte == self.bytes.end
                && byte == text.len_bytes()
                && text
                    .get_char(text.len_chars().saturating_sub(1))
                    .is_some_and(|ch| ch != '\n')
    }
}

impl Work {
    fn span(&self, bytes: Range<usize>) -> Span {
        Span {
            stamp: self.stamp,
            range: self.text.byte_to_char(bytes.start)..self.text.byte_to_char(bytes.end),
        }
    }

    pub(crate) fn run(
        self,
        command: crate::command::MarkdownCommand,
        cancel: &Cancellation,
    ) -> Result<Edit, String> {
        let bounded = cancel.bounded(std::time::Instant::now() + std::time::Duration::from_secs(2));
        check(&bounded)?;
        let plan = self.operate(command, &bounded)?;
        check(&bounded)?;
        Ok(plan)
    }
}

fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Markdown table action cancelled or timed out; source unchanged".into())
    } else {
        Ok(())
    }
}
