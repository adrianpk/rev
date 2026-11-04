use super::{Action, Session, SessionId, State};
use crate::buffer::Kill;

impl Session {
    /// Finishes owned workers outside the interactive event loop. Embedders
    /// must call this before process exit; ordinary buffer closure is async.
    pub fn shutdown_terminals(&mut self) -> std::io::Result<()> {
        self.terminals.sessions.shutdown()
    }

    pub(in crate::session) fn close_terminal(&mut self, id: SessionId, kill: Kill) -> Action {
        if let Err(error) = self.terminals.sessions.close(id) {
            return self.error(&format!("Close terminal: {error}"));
        }
        self.terminals.closing.insert(id, kill);
        self.input.cancel();
        self.pending = None;
        self.message("Terminal cleanup pending; buffer remains owned")
    }

    pub(super) fn finish_terminal_closures(&mut self) -> Action {
        let ids: Vec<_> = self.terminals.closing.keys().copied().collect();
        let mut action = Action::Wait;
        for id in ids {
            match self.terminals.sessions.get(id).map(|s| s.state()) {
                None => {
                    let kill = self.terminals.closing.remove(&id).expect("pending close");
                    self.terminal_closed(id);
                    action = self.remove_buffer(kill);
                }
                Some(State::Failed(error)) => {
                    let message = format!("Terminal buffer retained: {error}");
                    self.terminals.closing.remove(&id);
                    action = self.error(&message);
                }
                _ => {}
            }
        }
        action
    }
}
