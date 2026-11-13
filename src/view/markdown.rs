use super::View;

impl View {
    /// Effective Markdown decoration policy; Text mode suspends its application.
    pub fn pretty_markdown(&self) -> bool {
        self.markdown_pretty.unwrap_or(self.markdown_startup)
    }

    /// Explicit view choice, retained across buffer switches and copied by splits.
    pub fn markdown_presentation_override(&self) -> Option<bool> {
        self.markdown_pretty
    }

    pub(crate) fn toggle_markdown_presentation(&mut self) {
        self.markdown_pretty = Some(!self.pretty_markdown());
    }

    pub(crate) fn reset_markdown_presentation(&mut self) {
        self.markdown_pretty = None;
    }
}
