use super::{Action, Pending, Session};

impl Session {
    pub(super) fn extended(&mut self) -> Action {
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if let Err(error) = self.input.commands() {
            return self.error(&error.to_string());
        }
        // Entry is transient input; the eventual shared handler owns command
        // continuity, so a named yank-pop can consume the current yank capture.
        self.pending = Some(Pending::Command);
        self.message("")
    }

    pub(super) fn input_changed(&mut self) -> Action {
        if let Some(action) = self.refresh_timestamp() {
            return action;
        }
        if self.input.focus() == crate::input::Focus::Search {
            return self.search_step();
        }
        if self.input.prompt().is_some() {
            self.echo.clear();
        }
        self.sync_path();
        Action::Redraw
    }
}
