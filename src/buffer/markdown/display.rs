use std::ops::Range;

use super::super::{Buffer, presentation::Mapping};
use crate::markdown::{DisplayItem, Observation};
use crate::syntax::{Capture, MarkdownRole, Style};

/// Borrowed complete owner data with a current physical-row mapping. It never
/// yields an action observation or editable list/table subtree.
struct Display<'a> {
    observation: &'a Observation,
    base: Range<usize>,
    current: Range<usize>,
}

impl Display<'_> {
    fn current_range(&self, base: Range<usize>) -> Option<Range<usize>> {
        Some(
            self.current
                .start
                .checked_add(base.start.checked_sub(self.base.start)?)?
                ..self
                    .current
                    .start
                    .checked_add(base.end.checked_sub(self.base.start)?)?,
        )
    }

    fn heading(&self) -> Option<(u8, Option<Range<usize>>)> {
        self.observation.presentation_heading(self.base.clone())
    }

    fn items(&self) -> Vec<DisplayItem> {
        self.observation
            .presentation_items(self.base.clone())
            .filter_map(|mut item| {
                item.marker = self.current_range(item.marker)?;
                Some(item)
            })
            .collect()
    }

    fn captures(&self, visible: Range<usize>) -> Vec<Capture> {
        let mut captures = Vec::new();
        if let Some(capture) = self.heading_capture() {
            captures.push(capture);
        }
        for item in self.items() {
            captures.push(Capture {
                bytes: item.marker,
                style: Style::Markdown(MarkdownRole::List),
            });
        }
        captures.retain(|capture| {
            capture.bytes.start < visible.end && capture.bytes.end > visible.start
        });
        captures
    }

    fn heading_capture(&self) -> Option<Capture> {
        let (level, prefix) = self.heading()?;
        let start = prefix.map_or(Some(self.current.start), |prefix| {
            self.current_range(prefix).map(|range| range.start)
        })?;
        Some(Capture {
            bytes: start..self.current.end,
            style: Style::Markdown(MarkdownRole::Heading(level)),
        })
    }
}

impl Buffer {
    fn markdown_mapping(&self) -> Option<Option<&Mapping>> {
        if !self.markdown_admitted() {
            return None;
        }
        let observation = self.markdown.observation.as_ref()?;
        if observation.stamp == self.stamp() {
            return Some(None);
        }
        self.presentation
            .as_ref()
            .filter(|map| map.base == observation.stamp && map.current == self.stamp())
            .map(Some)
    }

    fn markdown_display(&self, row: usize) -> Option<Display<'_>> {
        let mapping = self.markdown_mapping()?;
        let line = self.text.get_line(row)?;
        let start = self.text.try_line_to_byte(row).ok()?;
        let current = start..start.checked_add(line.len_bytes())?;
        let base = mapping.map_or_else(
            || Some(current.clone()),
            |map| map.base_line(row, current.clone()),
        )?;
        Some(Display {
            observation: self.markdown.observation.as_ref()?,
            base,
            current,
        })
    }

    pub(super) fn has_markdown_display(&self) -> bool {
        self.markdown_mapping().is_some()
    }

    pub(super) fn markdown_structural_captures(&self, bytes: Range<usize>) -> Vec<Capture> {
        self.text
            .try_byte_to_line(bytes.start)
            .ok()
            .and_then(|row| self.markdown_display(row))
            .map_or_else(Vec::new, |display| display.captures(bytes))
    }

    pub(crate) fn markdown_heading_prefix(&self, row: usize) -> Option<(u8, Range<usize>)> {
        let display = self.markdown_display(row)?;
        let (level, prefix) = display.observation.heading_prefix(display.base.clone())?;
        Some((level, display.current_range(prefix)?))
    }

    pub(crate) fn markdown_display_items(&self, row: usize) -> Vec<DisplayItem> {
        self.markdown_display(row)
            .map_or_else(Vec::new, |display| display.items())
    }
}
