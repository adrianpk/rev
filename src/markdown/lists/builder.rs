use std::ops::Range;

use crate::buffer::edit::{Replacement, Retained};

use super::{Edit, source::Source};

pub(super) enum Shift {
    In(usize),
    Out(usize),
}
pub(super) struct Layout<'a> {
    pub(super) quotes: usize,
    pub(super) shift: Shift,
    pub(super) first_prefix: Option<&'a str>,
}

pub(super) struct Builder<'a, 'b> {
    source: &'a Source<'b>,
    region: Range<usize>,
    text: String,
    retained: Vec<Retained>,
    end: usize,
}

impl<'a, 'b> Builder<'a, 'b> {
    pub(super) fn new(source: &'a Source<'b>, region: Range<usize>) -> Result<Self, String> {
        let end = source.scalar(region.start)?.0;
        Ok(Self {
            source,
            region,
            text: String::new(),
            retained: Vec::new(),
            end,
        })
    }

    pub(super) fn insert(&mut self, text: &str) -> Result<(), String> {
        self.end = self
            .end
            .checked_add(text.chars().count())
            .ok_or("Markdown output is too large")?;
        self.text.push_str(text);
        Ok(())
    }

    pub(super) fn copy(&mut self, bytes: Range<usize>) -> Result<(), String> {
        if bytes.is_empty() {
            return Ok(());
        }
        if self.retained.len() >= super::super::MAX_EVENTS {
            return Err("Markdown retained source limit reached".into());
        }
        let source = self.source.span(bytes)?;
        let text = self
            .source
            .buffer
            .slice(&source)
            .map_err(|e| e.to_string())?;
        let start = self.end;
        self.end = self
            .end
            .checked_add(text.len_chars())
            .ok_or("Markdown output is too large")?;
        for chunk in text.chunks() {
            self.text.push_str(chunk);
        }
        self.retained.push(Retained {
            source,
            output: start..self.end,
        });
        Ok(())
    }

    pub(super) fn shift(&mut self, bytes: Range<usize>, layout: &Layout<'_>) -> Result<(), String> {
        let mut at = bytes.start;
        while at < bytes.end {
            let line = self.source.line(at, layout.quotes)?;
            let end = line.bytes.end.min(bytes.end);
            if at != line.bytes.start {
                self.insert(
                    layout
                        .first_prefix
                        .ok_or("Markdown item prefix is incomplete")?,
                )?;
                self.copy(at..end)?;
            } else {
                self.shift_line(&line, end, layout)?;
            }
            at = end;
        }
        Ok(())
    }

    fn shift_line(
        &mut self,
        line: &super::source::Line,
        end: usize,
        layout: &Layout<'_>,
    ) -> Result<(), String> {
        self.copy(line.bytes.start..line.quote_end)?;
        let columns = line.after_indent.saturating_sub(line.before_indent);
        let columns = match layout.shift {
            Shift::In(delta) => columns
                .checked_add(delta)
                .ok_or("Markdown indentation is too large")?,
            Shift::Out(delta) => columns.saturating_sub(delta),
        };
        self.insert(&" ".repeat(columns))?;
        self.copy(line.indent_end..end)
    }

    pub(super) fn finish(self) -> Result<Edit, String> {
        Ok(Edit {
            edits: vec![Replacement {
                source: self.source.span(self.region)?,
                text: self.text,
            }],
            retained: self.retained,
            cursor: None,
        })
    }
}
