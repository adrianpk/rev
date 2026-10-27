use std::collections::VecDeque;

use crate::buffer::BufferId;

use super::{Action, AfterSave, Pending, Session};

pub(super) struct Quit {
    buffers: VecDeque<BufferId>,
    pub(super) question: String,
}

impl Session {
    pub(super) fn quit(&mut self) -> Action {
        let buffers = self
            .buffers
            .iter()
            .filter(|buffer| buffer.is_modified())
            .map(|buffer| buffer.id())
            .collect();
        self.ask_quit(Quit {
            buffers,
            question: String::new(),
        })
    }

    fn ask_quit(&mut self, mut quit: Quit) -> Action {
        let Some(id) = quit.buffers.front() else {
            return Action::Exit;
        };
        let buffer = match self.buffers.get(*id) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        quit.question = format!("Quit: {:?}? y=save, n=discard, C-g=cancel", buffer.name());
        if let Err(error) = self.input.decision("Save before quitting?") {
            return self.error(&error.to_string());
        }
        self.echo_kind = crate::display::Message::Normal;
        self.echo = quit.question.clone();
        self.pending = Some(Pending::Quit(quit));
        Action::Redraw
    }

    pub(super) fn quit_answer(&mut self, quit: Quit, accepted: bool) -> Action {
        if !accepted {
            return self.quit_next(quit);
        }
        let Some(id) = quit.buffers.front().copied() else {
            return self.message("No quit decision is pending");
        };
        let buffer = match self.buffers.get(id) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        if buffer.file().is_none() {
            return self.write_argument(id, AfterSave::Quit(quit));
        }
        self.save_to(id, None, AfterSave::Quit(quit))
    }

    pub(super) fn quit_saved(&mut self, quit: Quit) -> Action {
        let current = quit
            .buffers
            .front()
            .and_then(|id| self.buffers.get(*id).ok());
        if current.is_none_or(|buffer| buffer.is_modified()) {
            return self.message("Buffer changed; quit canceled");
        }
        self.quit_next(quit)
    }

    pub(super) fn quit_next(&mut self, mut quit: Quit) -> Action {
        // Discard is permission for this exit only; cancellation must not mark
        // unsaved text clean or remove buffers/windows processed earlier.
        quit.buffers.pop_front();
        self.ask_quit(quit)
    }

    pub(super) fn cancel_quit(&mut self) -> bool {
        let quitting = matches!(
            self.pending,
            Some(
                Pending::Quit(_)
                    | Pending::Write {
                        after: AfterSave::Quit(_),
                        ..
                    }
                    | Pending::Save {
                        after: AfterSave::Quit(_),
                        ..
                    }
            )
        );
        if quitting {
            self.pending = None;
            self.input.cancel();
        }
        quitting
    }
}

#[cfg(test)]
mod tests;
