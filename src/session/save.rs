use std::path::Path;

use crate::buffer::{BufferId, Confirmation, Kill, SaveRequest};
use crate::file::{Outcome, Publication};

use super::{Action, AfterSave, Pending, Session};

impl Session {
    pub(super) fn save_buffer(&mut self) -> Action {
        match self.buffer() {
            Ok(buffer) if buffer.file().is_none() => self.write_file(),
            Ok(buffer) => self.save_to(buffer.id(), None, AfterSave::Stay),
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn write_file(&mut self) -> Action {
        self.write_argument(self.windows.selected().point.stamp.buffer, AfterSave::Stay)
    }

    pub(super) fn write_argument(&mut self, buffer: BufferId, after: AfterSave) -> Action {
        // A path-changing lifecycle operation is a bookmark-owner demand too:
        // admit closed durable records before the canonical owner retargets them.
        let bookmark_notice = if self
            .buffers
            .get(buffer)
            .is_ok_and(|buffer| buffer.file().is_some())
        {
            self.ensure_bookmarks().err().unwrap_or_default()
        } else {
            String::new()
        };
        if let Err(error) = self.input.argument("Write file: ", "") {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Write { buffer, after });
        self.message(&bookmark_notice)
    }

    pub(super) fn save_to(
        &mut self,
        id: BufferId,
        target: Option<&Path>,
        after: AfterSave,
    ) -> Action {
        match self.buffers.request_save(id, target) {
            Ok(SaveRequest::Ready(save)) => {
                let state = save.state();
                let outcome = self.buffers.publish_save(save);
                self.saved_state(state, &outcome, after)
            }
            Ok(SaveRequest::Confirm(confirmation)) => self.confirm(confirmation, after),
            Err(error) => {
                self.note_refile_rejection(id, &error.to_string());
                self.error(&format!("save: {error}"))
            }
        }
    }

    fn confirm(&mut self, confirmation: Confirmation, after: AfterSave) -> Action {
        if let Err(error) = self.input.decision("Save? [y/n]") {
            return self.error(&error.to_string());
        }
        self.echo_kind = crate::display::Message::Normal;
        self.echo = confirmation.message().into();
        self.pending = Some(Pending::Save {
            confirmation: Box::new(confirmation),
            after,
        });
        Action::Redraw
    }

    pub(super) fn decision(&mut self, accepted: bool) -> Action {
        match self.pending.take() {
            Some(Pending::MacroLibraryConsent(consent)) => {
                self.macro_library_answer(&consent, accepted)
            }
            Some(Pending::MacroMenuConsent(consent)) => self.macro_menu_answer(&consent, accepted),
            Some(Pending::MacroConsent(consent)) => self.macro_name_consent(*consent, accepted),
            pending => self.git_pending_decision(pending, accepted),
        }
    }

    fn git_pending_decision(&mut self, pending: Option<Pending>, accepted: bool) -> Action {
        match pending {
            Some(Pending::GitDiscard(consent)) => self.git_discard_answer(*consent, accepted),
            Some(Pending::GitRef(consent)) => self.git_ref_answer(*consent, accepted),
            Some(Pending::GitQuit {
                operations,
                sources,
                ..
            }) => self.git_quit_answer(accepted, operations, sources),
            Some(Pending::GitShutdown { .. }) => self.git_shutdown_answer(accepted),
            pending => self.terminal_pending_decision(pending, accepted),
        }
    }

    fn terminal_pending_decision(&mut self, pending: Option<Pending>, accepted: bool) -> Action {
        match pending {
            Some(Pending::TerminalQuit { sources, .. }) => {
                self.terminal_quit_answer(accepted, sources)
            }
            Some(Pending::TerminalShutdown { .. }) => self.terminal_shutdown_answer(accepted),
            Some(Pending::TerminalClose { kill, .. }) => {
                if accepted {
                    self.close_buffer(kill)
                } else {
                    self.message("Terminal retained")
                }
            }
            pending => self.source_pending_decision(pending, accepted),
        }
    }

    fn source_pending_decision(&mut self, pending: Option<Pending>, accepted: bool) -> Action {
        match pending {
            Some(Pending::Reload(reload)) => self.reload_answer(*reload, accepted),
            Some(Pending::Quit(quit)) => self.quit_answer(quit, accepted),
            Some(Pending::BookmarkReload(candidate)) => {
                self.bookmark_reload_answer(*candidate, accepted)
            }
            Some(Pending::TreeDelete(deletion)) => self.delete_answer(deletion, accepted),
            Some(Pending::Replace(state)) => self.replace_answer(*state, accepted),
            Some(Pending::Kill { kill, .. }) => {
                if accepted {
                    self.save_killing(kill)
                } else {
                    self.close_buffer(kill)
                }
            }
            Some(Pending::Save {
                confirmation,
                after,
            }) => self.save_decision(*confirmation, after, accepted),
            _ => self.message("No decision is pending"),
        }
    }

    fn save_decision(
        &mut self,
        confirmation: Confirmation,
        after: AfterSave,
        accepted: bool,
    ) -> Action {
        if !accepted {
            return self.message("Save canceled");
        }
        let state = confirmation.state();
        let outcome = self.buffers.confirm_save(confirmation);
        self.saved_state(state, &outcome, after)
    }

    pub(super) fn saved_state(
        &mut self,
        state: crate::buffer::State,
        outcome: &Outcome,
        after: AfterSave,
    ) -> Action {
        self.note_refile_save(state, outcome);
        self.saved(outcome, after)
    }

    pub(super) fn saved(&mut self, outcome: &Outcome, after: AfterSave) -> Action {
        self.lsp_saved(outcome);
        let refreshed = if outcome.publication() != Publication::NotPublished {
            self.tree.observe_path(outcome.target())
        } else {
            Ok(())
        };
        if outcome.is_complete()
            && let Err(error) = self.checkpoint_saved_clock(outcome.target())
        {
            return self.error(&format!("Source saved; {error}"));
        }
        let bookmark_error = if outcome.is_complete()
            && !matches!(&after, AfterSave::Quit(_))
            && self.buffers.snapshot().settings().bookmarks.save
                == crate::config::BookmarkSave::OnChange
            && self.bookmarks().is_dirty()
        {
            self.save_bookmarks().err()
        } else {
            None
        };
        let action = self.after_saved(outcome, after);
        if let Some(error) = bookmark_error {
            self.echo
                .push_str(&format!("; {error}; bookmark changes remain unsaved"));
            self.echo_kind = crate::display::Message::Warning;
        }
        if let Err(error) = refreshed {
            self.echo
                .push_str(&format!("; tree refresh failed: {error}"));
        }
        action
    }

    fn after_saved(&mut self, outcome: &Outcome, after: AfterSave) -> Action {
        if !outcome.is_complete() {
            return self.save_result(outcome);
        }
        match after {
            AfterSave::Stay => self.save_result(outcome),
            AfterSave::Close(kill) => self.close_saved(kill),
            AfterSave::Quit(quit) => self.quit_saved(quit),
        }
    }

    fn close_saved(&mut self, kill: Kill) -> Action {
        let buffer = match self.buffers.get(kill.id()) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        if buffer.is_modified() {
            return self.message("Buffer changed; close canceled");
        }
        self.close_buffer(kill)
    }

    pub(super) fn focus_message(&mut self, message: &str) -> Action {
        self.echo_kind = crate::display::Message::Normal;
        if let Some(Pending::TreeDelete(deletion)) = &self.pending {
            self.echo_kind = crate::display::Message::Normal;
            self.echo = format!("{}; {message}", deletion.message());
            return Action::Redraw;
        }
        let question = self.focus_question();
        if let Some(question) = question {
            let echo = format!("{question}; {message}");
            self.echo_kind = crate::display::Message::Normal;
            self.echo = echo;
            return Action::Redraw;
        }
        self.message(message)
    }

    fn focus_question(&self) -> Option<&str> {
        if let Some(question) = self.pending.as_ref().and_then(Pending::decision_question) {
            return Some(question);
        }
        if self.input.focus() == crate::input::Focus::Decision
            && matches!(
                self.macro_wait(),
                Some(crate::macros::Wait::Query | crate::macros::Wait::Step)
            )
        {
            return self.input.label();
        }
        None
    }

    fn save_result(&mut self, outcome: &Outcome) -> Action {
        if outcome.is_complete() {
            return self.message(&format!("Saved {:?}", outcome.target()));
        }
        let publication = match outcome.publication() {
            Publication::NotPublished => "not published",
            _ => "published; completion unresolved",
        };
        let error = outcome.error().map(ToString::to_string).unwrap_or_default();
        self.error(&format!(
            "save: {publication}: {error}; target {:?}",
            outcome.target()
        ))
    }
}

impl Pending {
    fn decision_question(&self) -> Option<&str> {
        match self {
            Self::MacroLibraryConsent(consent) => Some(consent.question()),
            Self::MacroMenuConsent(consent) => Some(consent.question()),
            Self::MacroConsent(consent) => Some(consent.question()),
            Self::Replace(state) if state.deciding() => Some("Replace occurrence? [y/n/q]"),
            Self::Quit(quit) => Some(quit.question.as_str()),
            Self::Save { confirmation, .. } => Some(confirmation.message()),
            Self::Kill { message, .. } => Some(message.as_str()),
            _ => self.resource_question(),
        }
    }

    fn resource_question(&self) -> Option<&str> {
        match self {
            Self::GitDiscard(consent) => Some(consent.message.as_str()),
            Self::GitRef(consent) => Some(consent.message.as_str()),
            Self::TerminalClose { message, .. } => Some(message.as_str()),
            Self::TerminalQuit { message, .. } | Self::TerminalShutdown { message } => {
                Some(message.as_str())
            }
            Self::GitQuit { message, .. } | Self::GitShutdown { message } => Some(message.as_str()),
            _ => None,
        }
    }
}
