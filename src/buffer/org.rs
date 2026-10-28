use super::{Buffer, Error, Stamp};
use crate::org::Index;

impl Buffer {
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

impl Buffer {
    pub fn org_highlights(&self) -> Option<&crate::org::Highlights> {
        self.org_highlights.as_ref()
    }

    pub fn org_captures(&self, bytes: std::ops::Range<usize>) -> Vec<crate::syntax::Capture> {
        if !self.org_colors_admitted() {
            return Vec::new();
        }
        self.org_highlights
            .as_ref()
            .map_or_else(Vec::new, |colors| colors.captures(self, bytes))
    }

    pub fn install_org_coloring(&mut self, coloring: crate::org::Coloring) -> Result<(), Error> {
        let (index, highlights) = coloring.into_parts();
        index.check(self)?;
        highlights.check(self)?;
        self.org = Some(index);
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
            && self
                .org_highlights
                .as_ref()
                .is_some_and(|colors| colors.stamp() == self.stamp())
    }

    pub(crate) fn needs_org_colors(&self) -> bool {
        self.org_colors_admitted() && self.org_attempt != Some(self.stamp())
    }

    pub(crate) fn attempted_org_colors(&mut self) {
        self.org_attempt = Some(self.stamp());
    }
}
