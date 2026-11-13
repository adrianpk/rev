use super::{Input, keymap::Context};

#[cfg(test)]
mod tests;

impl Input {
    /// The host selects native Markdown only for the active source buffer.
    pub fn set_markdown_context(&mut self, active: bool) {
        if self.workspace_markdown != active {
            self.retire_command_hints();
            self.clear_prefix();
            self.suppress_repeat = true;
            self.workspace_markdown = active;
            if !self.plugin_keys.values.is_empty() {
                self.plugin_keymaps(self.plugin_keys.values.clone());
            }
        }
    }

    pub(super) fn editing_local_context(&self) -> Option<Context> {
        self.workspace_org
            .map(super::OrgContext::keymap)
            .or(self.workspace_markdown.then_some(Context::Markdown))
    }
}
