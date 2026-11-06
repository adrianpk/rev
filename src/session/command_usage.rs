use super::{Action, Session};
use crate::command::usage::History;

impl Session {
    /// Attach an explicitly loaded state owner before embedding starts input.
    /// Pure new/empty construction keeps isolated usage and discovers no files.
    pub fn with_command_usage(mut self, history: History) -> Self {
        self.input.set_command_usage(history);
        self.expire_command_usage();
        self
    }

    /// Request publication after an owned input batch; no filesystem work runs
    /// on this caller. Embeddings can use expire_input to collect completion.
    pub fn publish_command_usage(&mut self) {
        self.input.publish_command_usage();
    }

    /// Final bounded publication for an embedding or ordinary terminal shutdown.
    /// State failures are returned separately from editing/terminal outcomes.
    pub fn flush_command_usage(&mut self) -> Result<(), String> {
        self.input.flush_command_usage()
    }

    pub(super) fn expire_command_usage(&mut self) -> Action {
        match self.input.poll_command_usage() {
            Some(message) => {
                self.notice(&message);
                Action::Redraw
            }
            None => Action::Wait,
        }
    }

    pub(super) fn startup_command_usage(&mut self) -> Option<String> {
        match crate::config::Roots::from_env() {
            Ok(roots) => {
                self.input
                    .set_command_usage(History::open(&roots.state.join("command-history.toml")));
                self.input.poll_command_usage()
            }
            Err(_) => Some(
                "Command usage: cannot resolve the state root; session ranking remains available"
                    .into(),
            ),
        }
    }
}
