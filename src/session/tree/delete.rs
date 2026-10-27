use crate::display::Message;
use crate::file::{DeleteDecision, Deleted, Deletion};

use super::super::{Action, Pending, Session};

impl Session {
    pub(super) fn tree_delete(&mut self) -> Action {
        if !self.tree.focused() {
            return self.message("File tree is not focused");
        }
        match self.tree.prepare_delete() {
            Ok(deletion) => self.delete_question(deletion),
            Err(error) => self.delete_notice(error.to_string(), Message::Error),
        }
    }

    fn delete_question(&mut self, deletion: Deletion) -> Action {
        if let Err(error) = self.input.decision("Delete entry? [y/n]") {
            return self.error(&error.to_string());
        }
        self.echo_kind = crate::display::Message::Normal;
        self.echo = deletion.message();
        self.pending = Some(Pending::TreeDelete(deletion));
        Action::Redraw
    }

    pub(in crate::session) fn delete_answer(
        &mut self,
        deletion: Deletion,
        accepted: bool,
    ) -> Action {
        if !accepted {
            return self.message("Delete canceled");
        }
        match deletion.confirm() {
            Ok(DeleteDecision::Recursive(deletion)) => self.delete_question(deletion),
            Ok(DeleteDecision::Ready(deletion)) => {
                let deleted = self.buffers.delete(deletion);
                self.deleted(deleted)
            }
            Err(error) => self.delete_notice(error.to_string(), Message::Error),
        }
    }

    fn deleted(&mut self, deleted: Deleted) -> Action {
        let mut notice = match deleted.error() {
            Some(error) => format!("Delete failed; may be partial: {error}; open text retained"),
            None => format!(
                "Deleted {}; open text retained; explicit save recreates files",
                deleted.source().display()
            ),
        };
        if let Some(warning) = deleted.warning() {
            notice.push_str(&format!("; {warning}"));
        }
        let severity = if deleted.error().is_some() {
            Message::Error
        } else if deleted.warning().is_some() {
            Message::Warning
        } else {
            Message::Normal
        };
        self.delete_notice(notice, severity)
    }

    fn delete_notice(&mut self, mut notice: String, mut severity: Message) -> Action {
        if let Err(error) = self.tree.refresh() {
            notice.push_str(&format!("; tree refresh failed: {error}"));
            severity = if severity == Message::Error {
                Message::Error
            } else {
                Message::Warning
            };
        }
        let action = self.message(&notice);
        self.echo_kind = severity;
        action
    }
}
