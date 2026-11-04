use super::{Action, Session, SessionId, State};
use crate::buffer::Kill;
use crate::session::Pending;

impl Session {
    /// Finishes owned workers outside the interactive event loop. Embedders
    /// must call this before process exit; ordinary buffer closure is async.
    pub fn shutdown_terminals(&mut self) -> std::io::Result<()> {
        let link = self.stop_terminal_link();
        let sessions = self.terminals.sessions.shutdown();
        link.and(sessions)
    }

    pub(in crate::session) fn terminal_closing(&self, id: SessionId) -> bool {
        self.terminals.closing.contains_key(&id)
    }

    pub(in crate::session) fn cancel_terminal_close(&mut self) -> bool {
        if !matches!(self.pending, Some(Pending::TerminalClose { .. })) {
            return false;
        }
        self.pending = None;
        self.input.cancel();
        true
    }

    pub(in crate::session) fn close_terminal(&mut self, id: SessionId, kill: Kill) -> Action {
        if self.terminals.closing.contains_key(&id) {
            self.input.cancel();
            self.pending = None;
            return self.message("Terminal cleanup already pending; buffer remains owned");
        }
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
                    let buffer = kill.id();
                    action = self.remove_buffer(kill);
                    if self.buffers.get(buffer).is_ok() {
                        self.cancel_terminal_shutdown();
                    }
                }
                Some(State::Failed(error)) => {
                    let message = format!("Terminal buffer retained: {error}");
                    self.terminals.closing.remove(&id);
                    self.cancel_terminal_shutdown();
                    action = self.error(&message);
                }
                _ => {}
            }
        }
        action
    }
}
