use super::{Action, Session};

impl Session {
    pub(super) fn paste_text(&mut self, text: &str) -> Action {
        self.timestamp_repeat = None;
        if self.tree.focused() {
            return self.message("File tree owns input focus");
        }
        self.yank = None;
        self.kills.break_chain();
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if text.contains('\0') {
            return self.message("Paste contains NUL; binary input rejected");
        }
        if text.is_empty() {
            return self.message("Empty paste");
        }
        let point = self.windows.selected().point.position.0;
        let result = self.replace(point..point, text);
        if matches!(result, Ok(true)) {
            self.windows.selected_mut().reset_navigation();
        }
        self.finish(result)
    }
}
