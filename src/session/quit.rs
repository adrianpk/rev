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
    Roots(crate::context::Generation),
}
impl Goal {
    pub(super) fn label(self) -> &'static str {
        if matches!(self, Self::Roots(_)) {
            "Remove root"
        } else if self == Self::Exit {
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
    targets: Option<Vec<BufferId>>,
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
            .filter(|buffer| {
                buffer.is_modified()
                    && self
                        .targets
                        .as_ref()
                        .is_none_or(|ids| ids.contains(&buffer.id()))
            })
            .all(|buffer| self.discarded.contains(&buffer.stamp()))
    }
}

impl Session {
    pub(super) fn quit(&mut self) -> Action {
        if self.context_storage.busy() {
            return self
                .error("Project preparation is pending; cancel or finish it before quitting");
        }
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

    pub(in crate::session) fn begin_scoped_departure(
        &mut self,
        goal: Goal,
        targets: Vec<BufferId>,
    ) -> Action {
        let buffers = targets
            .iter()
            .filter(|id| self.buffers.get(**id).is_ok_and(|b| b.is_modified()))
            .copied()
            .collect();
        self.ask_quit(Quit {
            buffers,
            question: String::new(),
            sources: Sources {
                goal,
                targets: Some(targets),
                ..Sources::default()
            },
        })
    }

    fn ask_quit(&mut self, mut quit: Quit) -> Action {
        let Some(id) = quit.buffers.front() else {
            if let Goal::Roots(generation) = quit.sources.goal {
                return self.finish_workspace_sources(generation, quit.sources);
            }
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
        let decision = if quit.sources.goal == Goal::Exit {
            self.input.save_decision("Save before quitting?")
        } else {
            self.input.decision("Save before quitting?")
        };
        if let Err(error) = decision {
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
        if self.context_management.is_some() {
            return self.capture_management();
        }
        if self.context_checkpoint_needed() {
            return self.checkpoint_context(goal);
        }
        if self.project_confirmation_needed() {
            return self.confirm_project_context(goal);
        }
        self.finish_context_departure(goal)
    }

    pub(in crate::session) fn finish_context_departure(&mut self, goal: Goal) -> Action {
        if matches!(goal, Goal::Roots(_)) {
            return self.error("Root removal requires its scoped source owner");
        }
        if let Goal::Context(generation) = goal {
            return self.activate_context(generation);
        }
        self.end_timer_session();
        self.installation.cancel();
        self.shell.stop();
        self.plugins.host.retire();
        Action::Exit
    }

    /// Read-only admission guard for native operations that add sources after
    /// departure has already captured its consent and source set.
    pub(super) fn quit_pending(&self) -> bool {
        matches!(
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
        )
    }

    pub(super) fn cancel_quit(&mut self) -> bool {
        let quitting = self.quit_pending();
        if quitting {
            self.context_prepared = None;
            self.context_membership = None;
            self.context_management = None;
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
