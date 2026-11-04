use super::{Action, Session, SessionId, State};
use crate::{
    buffer::Kill,
    session::{Pending, quit::Sources},
};

const WAITING: &str = "Terminal cleanup pending; C-g cancels exit";

#[derive(Default)]
pub(super) enum Shutdown {
    #[default]
    Idle,
    Closing(Sources),
}
struct Closure {
    id: SessionId,
    kill: Option<Kill>,
}

impl Session {
    pub(in crate::session) fn terminal_shutdown_active(&self) -> bool {
        matches!(self.terminals.shutdown, Shutdown::Closing(_))
    }
    pub(in crate::session) fn terminal_shutdown_ready(&self) -> bool {
        self.terminal_shutdown_active()
            && (self.terminals.sessions.ids().next().is_none()
                || self.terminal_shutdown_failure().is_some())
    }
    pub(in crate::session) fn finish_terminal_quit(&mut self, sources: Sources) -> Action {
        if !sources.permits(&self.buffers) {
            return self.error("Source changed; quit canceled without terminating terminals");
        }
        let live = self
            .terminals
            .sessions
            .ids()
            .filter(|id| !self.terminals.closing.contains_key(id))
            .filter(|id| {
                self.terminals
                    .sessions
                    .get(*id)
                    .expect("owned session")
                    .owns_process()
            })
            .count();
        if live == 0 {
            return self.begin_terminal_shutdown(sources);
        }
        if let Err(error) = self.input.decision("Terminate terminal sessions?") {
            return self.error(&error.to_string());
        }
        let message =
            format!("Quit: terminate {live} terminal sessions? y=terminate, n=cancel, C-g=cancel");
        self.pending = Some(Pending::TerminalQuit {
            message: message.clone(),
            sources,
        });
        self.message(&message)
    }
    pub(in crate::session) fn terminal_quit_answer(
        &mut self,
        accepted: bool,
        sources: Sources,
    ) -> Action {
        self.input.cancel();
        if !accepted {
            self.cancel_git_shutdown();
            return self.message("Quit canceled; terminal sessions retained");
        }
        self.begin_terminal_shutdown(sources)
    }
    fn begin_terminal_shutdown(&mut self, sources: Sources) -> Action {
        if !sources.permits(&self.buffers) || !self.git_exit_permitted() {
            self.cancel_git_shutdown();
            return self
                .error("Source or owned Git work changed; quit canceled before termination");
        }
        if self.terminals.sessions.ids().next().is_none() {
            return self.begin_git_shutdown(sources);
        }
        match self.request_terminal_shutdown(sources) {
            Ok(()) => Action::Redraw,
            Err(error) => {
                self.cancel_terminal_shutdown();
                self.error(&format!("Quit canceled: {error}"))
            }
        }
    }
    fn request_terminal_shutdown(&mut self, sources: Sources) -> Result<(), String> {
        if !sources.permits(&self.buffers) {
            return Err("Source changed; terminal sessions retained".into());
        }
        if self.terminals.sessions.ids().next().is_none() {
            return Ok(());
        }
        let closures = self.prepare_terminal_shutdown()?;
        self.wait_terminal_shutdown()?;
        self.terminals.shutdown = Shutdown::Closing(sources);
        for closure in closures {
            self.terminals
                .sessions
                .close(closure.id)
                .map_err(|e| e.to_string())?;
            if let Some(kill) = closure.kill {
                self.terminals.closing.insert(closure.id, kill);
            }
        }
        Ok(())
    }
    fn prepare_terminal_shutdown(&self) -> Result<Vec<Closure>, String> {
        let mut closures = Vec::new();
        for id in self.terminals.sessions.ids() {
            if self.terminals.closing.contains_key(&id) {
                continue;
            }
            let kill = self
                .buffers
                .iter()
                .find(|buffer| buffer.terminal() == Some(id))
                .map(|buffer| self.buffers.prepare_kill(buffer.id()))
                .transpose()
                .map_err(|e| e.to_string())?;
            closures.push(Closure { id, kill });
        }
        Ok(closures)
    }
    fn wait_terminal_shutdown(&mut self) -> Result<(), String> {
        self.input
            .decision("Closing terminals")
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::TerminalShutdown {
            message: WAITING.into(),
        });
        self.echo_kind = crate::display::Message::Normal;
        self.echo = WAITING.into();
        Ok(())
    }
    pub(in crate::session) fn terminal_shutdown_answer(&mut self, accepted: bool) -> Action {
        if !accepted {
            self.cancel_terminal_shutdown();
            self.input.cancel();
            return self.message("Quit canceled; approved terminal cleanup continues");
        }
        match self.wait_terminal_shutdown() {
            Ok(()) => Action::Redraw,
            Err(error) => {
                self.cancel_terminal_shutdown();
                self.error(&error)
            }
        }
    }
    pub(in crate::session) fn cancel_terminal_shutdown(&mut self) {
        self.terminals.shutdown = Shutdown::Idle;
        if matches!(self.pending, Some(Pending::TerminalShutdown { .. })) {
            self.pending = None;
            self.input.cancel();
        }
    }
    fn terminal_shutdown_failure(&self) -> Option<String> {
        self.terminals.sessions.ids().find_map(|id| {
            match self
                .terminals
                .sessions
                .get(id)
                .expect("owned session")
                .state()
            {
                State::Failed(message) => Some(message.clone()),
                _ => None,
            }
        })
    }
    pub(in crate::session) fn advance_terminal_shutdown(&mut self) -> Action {
        let Shutdown::Closing(sources) = &self.terminals.shutdown else {
            return Action::Wait;
        };
        if !sources.permits(&self.buffers) {
            self.cancel_terminal_shutdown();
            return self
                .error("Source changed during terminal shutdown; quit canceled, buffers retained");
        }
        if let Some(error) = self.terminal_shutdown_failure() {
            self.cancel_terminal_shutdown();
            return self.error(&format!(
                "Terminal cleanup failed; quit canceled, buffers retained: {error}"
            ));
        }
        if self.terminals.sessions.ids().next().is_some() {
            return Action::Wait;
        }
        if !self.terminals.closing.is_empty() {
            return Action::Wait;
        }
        // Keep the closing state observable until the host consumes Exit.
        // frame() may service receipts without consuming the native action.
        if self.git.engine.busy() {
            let Shutdown::Closing(sources) = std::mem::take(&mut self.terminals.shutdown) else {
                unreachable!("terminal shutdown owns source decisions");
            };
            return self.begin_git_shutdown(sources);
        }
        self.finish_editor_quit()
    }
}
