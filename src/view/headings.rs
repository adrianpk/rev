use super::View;

impl View {
    /// Effective marker presentation; canonical source positions are unchanged.
    pub fn pretty_headings(&self) -> bool {
        self.heading_pretty.unwrap_or(self.headings.pretty)
    }

    /// Explicit session choice, including a value equal to the startup default.
    pub fn heading_presentation_override(&self) -> Option<bool> {
        self.heading_pretty
    }

    pub(crate) fn toggle_heading_presentation(&mut self) {
        self.heading_pretty = Some(!self.pretty_headings());
        self.reset_heading_layout();
    }

    pub(crate) fn reset_heading_presentation(&mut self) {
        self.heading_pretty = None;
        self.reset_heading_layout();
    }

    /// Restoring source anchors from a retained view must keep the live choice.
    pub(crate) fn retain_heading_presentation(&mut self, current: &Self) {
        self.headings = current.headings.clone();
        self.heading_pretty = current.heading_pretty;
        self.reset_heading_layout();
    }

    pub(crate) fn compact_headings(&self) -> bool {
        self.pretty_headings() && self.headings.hide_leading_stars
    }

    pub(super) fn reset_heading_layout(&mut self) {
        self.wrap_cache = super::wrap::Cache::headings(self.compact_headings());
        self.reset_navigation();
    }
}
