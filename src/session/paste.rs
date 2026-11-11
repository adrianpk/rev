use super::{Action, Session};

impl Session {
    pub(super) fn paste_text(&mut self, text: &str) -> Action {
        if self.terminals.focused {
            return self.terminal_send(text.as_bytes());
        }
        if self.tangling.is_some() || self.execution.is_some() || self.feeds.is_some() {
            return self.message("Source output owns input; C-g cancels remaining files");
        }
        if self.capture_blocks_input() {
            return self.message("Capture input pending; C-g cancels");
        }
        self.timestamp_repeat = None;
        if self.tree.focused() {
            return self.message("File tree owns input focus");
        }
        self.yank = None;
        self.kills.break_chain();
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        self.publish_paste(text)
    }

    fn publish_paste(&mut self, text: &str) -> Action {
        if text.contains('\0') {
            return self.message("Paste contains NUL; binary input rejected");
        }
        if text.is_empty() {
            return self.message("Empty paste");
        }
        let result = self.paste_entry(text);
        if matches!(result, Ok(true)) {
            self.windows.selected_mut().reset_navigation();
        }
        self.finish(result)
    }
    pub(super) fn paste_entry(&mut self, text: &str) -> Result<bool, crate::buffer::edit::Error> {
        if self
            .editing_state()
            .is_some_and(crate::input::EditingState::text_entry)
        {
            self.entry_text(text)
        } else {
            let point = self.view().point.position.0;
            self.replace(point..point, text)
        }
    }
}
