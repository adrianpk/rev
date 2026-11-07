use std::collections::VecDeque;

use crate::buffer::BufferId;

use super::{Action, AfterSave, Pending, Session};

pub(super) struct Quit {
    buffers: VecDeque<BufferId>,
    pub(super) question: String,
    sources: Sources,
}
impl Quit {
    pub(super) fn goal(&self) -> Goal {
        self.sources.goal
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Goal {
    #[default]
    Exit,
    Context(crate::context::Generation),
}
impl Goal {
    pub(super) fn label(self) -> &'static str {
        if self == Self::Exit {
            "Quit"
        } else {
            "Leave context"
        }
    }
}

/// Permission to discard specific modified source revisions for this exit.
/// A later publication requires another source decision, even during cleanup.
#[derive(Default)]
pub(super) struct Sources {
    discarded: Vec<crate::buffer::Stamp>,
    pub(super) goal: Goal,
}
impl Sources {
    fn remember(&mut self, buffer: &crate::buffer::Buffer) {
        if buffer.is_modified() {
            self.discarded.push(buffer.stamp());
        }
    }
    pub(super) fn permits(&self, buffers: &crate::buffer::Buffers) -> bool {
        buffers
            .iter()
            .filter(|buffer| buffer.is_modified())
            .all(|buffer| self.discarded.contains(&buffer.stamp()))
    }
}

impl Session {
    pub(super) fn quit(&mut self) -> Action {
        if self.git_shutdown_active() {
            return self.advance_git_shutdown();
        }
        if self.terminal_shutdown_active() {
            return self.advance_terminal_shutdown();
        }
        if self.active_clock.is_some() {
            return self.clock_departure(super::outline::clocks::Departure::Quit);
        }
        self.begin_quit()
    }
    pub(super) fn begin_quit(&mut self) -> Action {
        self.context_prepared = None;
        self.begin_departure(Goal::Exit)
    }
    pub(super) fn begin_departure(&mut self, goal: Goal) -> Action {
        self.cancel_git_shutdown();
        let buffers = self
            .buffers
            .iter()
            .filter(|buffer| buffer.is_modified())
            .map(|buffer| buffer.id())
            .collect();
        self.ask_quit(Quit {
            buffers,
            question: String::new(),
            sources: Sources {
                goal,
                ..Sources::default()
            },
        })
    }

    fn ask_quit(&mut self, mut quit: Quit) -> Action {
        let Some(id) = quit.buffers.front() else {
            return self.finish_bookmark_quit(quit.sources);
        };
        let buffer = match self.buffers.get(*id) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        let operation = quit.sources.goal.label();
        quit.question = format!(
            "{operation}: {:?}? y=save, n=discard, C-g=cancel",
            buffer.name()
        );
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
        if let Some(id) = quit.buffers.pop_front()
            && let Ok(buffer) = self.buffers.get(id)
        {
            quit.sources.remember(buffer);
        }
        self.ask_quit(quit)
    }

    pub(super) fn finish_editor_quit(&mut self, goal: Goal) -> Action {
        if self.git.engine.busy() {
            return self.error("Git cleanup remains owned; exit requires its guarded completion");
        }
        if let Goal::Context(generation) = goal {
            return self.activate_context(generation);
        }
        self.end_timer_session();
        self.installation.cancel();
        self.shell.stop();
        Action::Exit
    }

    pub(super) fn cancel_quit(&mut self) -> bool {
        let quitting = matches!(
            self.pending,
            Some(
                Pending::Quit(_)
                    | Pending::BookmarkQuit(_)
                    | Pending::TerminalQuit { .. }
                    | Pending::TerminalShutdown { .. }
                    | Pending::GitQuit { .. }
                    | Pending::GitShutdown { .. }
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
            self.context_prepared = None;
            self.cancel_terminal_shutdown();
            self.cancel_git_shutdown();
            self.pending = None;
            self.input.cancel();
        }
        quitting
    }
}

#[cfg(test)]
mod tests;
