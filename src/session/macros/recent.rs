use super::*;
use crate::input::Outcome;
use crate::macros::{Argument, Recorder, Role};

impl Session {
    /// Reuse the exact typed acceptance path in a detached capture draft. It is
    /// never installed across dispatch and cannot make normal input recording.
    pub(in crate::session) fn macro_recent_outcome(&mut self, outcome: &Outcome, source: bool) {
        if self.macros.expanded || self.macro_manual() || self.macro_definition_selected() {
            return;
        }
        let command = match outcome {
            Outcome::Command(c) | Outcome::ArgumentPrefix(c) => Some(*c),
            Outcome::Invocation(i) => Some(i.command),
            _ => None,
        };
        if command == Some(Command::KeyboardQuit) {
            self.macros.recent.draft = None;
            return;
        }
        if command.is_some_and(|c| {
            !super::supported(c) && !matches!(c, Command::Macro(_) | Command::KeyboardQuit)
        }) {
            self.macros.recent.draft = None;
            return;
        }
        let pending = self
            .macros
            .recent
            .draft
            .as_ref()
            .is_some_and(|r| r.command.is_some() || r.selecting);
        if !source && !pending {
            self.macros.recent.draft = None;
            return;
        }
        let mut recent = self
            .macros
            .recent
            .draft
            .take()
            .unwrap_or_else(|| Recorder::new(self.macro_entry()));
        if recent.fresh_pending() {
            recent.entry = self.macro_entry();
        }
        let recording = self.macros.recorder.take();
        self.macros.recorder = Some(recent);
        self.macros.recent_capturing = true;
        let _ = self.macro_capture_outcome(outcome);
        self.macros.recent_capturing = false;
        self.macros.recent.draft = self.macros.recorder.take();
        self.macros.recorder = recording;
        self.macros.recent.drain();
    }
    pub(super) fn macro_recent_argument(&mut self, role: Role, value: &str) {
        if self.macros.expanded {
            return;
        }
        if let Some(recorder) = &mut self.macros.recent.draft
            && recorder.command.is_some()
            && recorder
                .argument(Argument {
                    role,
                    value: value.into(),
                })
                .is_err()
        {
            self.macros.recent.draft = None;
        }
        self.macros.recent.drain();
    }
    pub(super) fn macro_recent_finish(&mut self) {
        if self.macros.expanded {
            return;
        }
        if self.pending.is_none()
            && !self.layout_active()
            && self.input.focus() == Focus::Editing
            && let Some(recorder) = &mut self.macros.recent.draft
            && recorder.finish_command().is_err()
        {
            self.macros.recent.draft = None;
        }
        self.macros.recent.drain();
        if self.macros.admit_memory(None).is_err() {
            self.macros.recent.clear();
        }
    }
}
