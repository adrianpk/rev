use super::{Action, Pending, Session};

impl Session {
    pub(super) fn extended(&mut self) -> Action {
        if let Err(error) = self.stop_typing() {
            return self.message(&error.to_string());
        }
        if let Err(error) = self.input.commands() {
            return self.message(&error.to_string());
        }
        // Entry is transient input; the eventual shared handler owns command
        // continuity, so a named yank-pop can consume the current yank capture.
        self.pending = Some(Pending::Command);
        self.message("")
    }

    pub(super) fn input_changed(&mut self) -> Action {
        if self.input.prompt().is_some() {
            self.echo.clear();
        }
        Action::Redraw
    }
}
