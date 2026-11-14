mod display;

use super::{Buffer, Buffers, Error, Stamp};
use crate::org::Index;

impl Buffer {
    /// Effective visual indentation; ordinary text buffers retain source geometry.
    pub fn org_indentation(&self) -> bool {
        self.is_org() && self.org_indent_override.unwrap_or(self.org_indent)
    }

    /// Runtime presentation belongs to this buffer, including readonly results.
    /// It survives source/mode changes and never enters text or edit history.
    pub(crate) fn set_org_indentation(&mut self, enabled: bool) {
        self.org_indent_override = Some(enabled);
        if enabled {
            self.request_org_outline();
        }
    }

    /// No source scan: the existing native outline owns complete evidence.
    pub(crate) fn org_line_indentation(&self, row: usize) -> Option<crate::org::Indentation> {
        self.org_display()?
            .indentation(row, self.text.try_line_to_byte(row).ok()?)
    }

    pub(crate) fn org_heading_level(&self, row: usize) -> Option<usize> {
        self.org_display()?
            .heading(row, self.text.try_line_to_byte(row).ok()?)
    }

    /// Optional derived state: ordinary buffers never start analysis implicitly.
    pub fn org_index(&self) -> Option<&Index> {
        self.org.as_ref()
    }

    /// Publish only observations of this exact canonical revision.
    pub fn install_org_index(&mut self, index: Index) -> Result<(), Error> {
        index.check(self)?;
        self.org = Some(index);
        Ok(())
    }

    /// Move retained offsets into owned analysis without cloning the index.
    pub fn take_org_index(&mut self) -> Option<Index> {
        self.org.take()
    }

    pub(super) fn edit_org(
        &mut self,
        before: Stamp,
        after: Stamp,
        edits: &[tree_sitter::InputEdit],
    ) {
        if let Some(index) = &mut self.org {
            index.edited(before, after, edits);
        }
    }
}

impl Buffers {
    pub(crate) fn retire_org_presentations(&mut self) {
        for buffer in self.entries.values_mut().filter(|buffer| buffer.is_org()) {
            if buffer.presentation.take().is_some() {
                buffer.org_attempt = Some(buffer.stamp());
            }
        }
    }
}

impl Buffer {
    /// Derived styles are available only when syntax policy admits coloring.
    pub fn org_highlights(&self) -> Option<&crate::org::Highlights> {
        self.org_highlights
            .as_ref()
            .filter(|_| self.org_colors_admitted())
    }

    pub fn org_captures(&self, bytes: std::ops::Range<usize>) -> Vec<crate::syntax::Capture> {
        if !self.org_colors_admitted() {
            return Vec::new();
        }
        if self.org_outline().is_none() {
            return self.org_structural_capture(bytes);
        }
        self.org_highlights
            .as_ref()
            .map_or_else(Vec::new, |colors| colors.captures(self, bytes))
    }

    pub fn install_org_coloring(&mut self, coloring: crate::org::Coloring) -> Result<(), Error> {
        let (index, highlights, outline) = coloring.into_parts();
        index.check(self)?;
        highlights.check(self)?;
        if !self
            .org_display()
            .is_some_and(|display| display.same_geometry(&outline))
        {
            self.org_geometry = self
                .org_geometry
                .checked_add(1)
                .expect("Org geometry identity exhausted");
        }
        self.presentation = None;
        self.org = Some(index);
        self.org_outline = Some(outline);
        self.org_attempt = Some(highlights.stamp());
        self.org_highlights = Some(highlights);
        Ok(())
    }

    pub(crate) fn org_colors_admitted(&self) -> bool {
        self.is_org()
            && self.syntax_policy.enabled
            && (self.syntax_policy.max_bytes == 0
                || self.len_bytes() <= self.syntax_policy.max_bytes)
    }

    pub(crate) fn has_org_colors(&self) -> bool {
        self.org_colors_admitted()
            && (self.org_display().is_some()
                || self
                    .org_highlights
                    .as_ref()
                    .is_some_and(|colors| colors.stamp() == self.stamp()))
    }

    pub(crate) fn needs_org_colors(&self) -> bool {
        self.org_colors_admitted() && self.org_attempt != Some(self.stamp())
    }

    pub(crate) fn needs_org_outline(&self) -> bool {
        self.is_org() && self.org_attempt != Some(self.stamp())
    }

    /// A new explicit presentation request can retry a canceled observation.
    /// The existing session worker still owns admission, bounds and retirement.
    pub(crate) fn request_org_outline(&mut self) {
        if self.is_org() && self.org_outline().is_none() {
            self.org_attempt = None;
        }
    }

    pub(crate) fn attempted_org_colors(&mut self) {
        self.org_attempt = Some(self.stamp());
    }
}

impl Buffer {
    fn org_structural_capture(
        &self,
        mut bytes: std::ops::Range<usize>,
    ) -> Vec<crate::syntax::Capture> {
        let Ok(row) = self.text.try_byte_to_line(bytes.start) else {
            return Vec::new();
        };
        if self.org_heading_level(row).is_none() || bytes.end > self.len_bytes() {
            return Vec::new();
        }
        bytes.end = bytes.end.min(
            self.text
                .try_line_to_byte(row + 1)
                .unwrap_or(self.len_bytes()),
        );
        if bytes.is_empty() {
            return Vec::new();
        }
        // Base lexical contexts are not re-proven by structural admission.
        // Suppress them as a whole, including cross-row captures, until exact
        // analysis replaces the carry. Heading faces use current source bytes.
        vec![crate::syntax::Capture {
            bytes,
            style: crate::syntax::Style::Org(crate::org::Role::Heading),
        }]
    }
}

impl Buffer {
    pub(crate) fn org_outline(&self) -> Option<&std::sync::Arc<crate::org::Outline>> {
        self.org_outline
            .as_ref()
            .filter(|outline| self.is_org() && outline.stamp == self.stamp())
    }
}
