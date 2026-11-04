use super::{Action, Session};
use crate::{
    git::OperationId,
    session::{Pending, quit::Sources},
};

const WAITING: &str = "Git cleanup pending; C-g cancels exit, approved cleanup continues";

impl Session {
    pub(in crate::session) fn prepare_git_quit(&mut self, sources: Sources) -> Action {
        if !sources.permits(&self.buffers) {
            return self.error("Source changed; quit canceled without terminating Git");
        }
        let operations: Vec<_> = self.git.engine.progress().map(|p| p.operation).collect();
        if operations.is_empty() {
            return self.finish_terminal_quit(sources);
        }
        if let Err(error) = self.input.decision("Terminate owned Git operations?") {
            return self.error(&error.to_string());
        }
        let message = format!(
            "Quit: terminate owned Git operations {operations:?}? y=terminate, n=cancel, C-g=cancel"
        );
        self.pending = Some(Pending::GitQuit {
            operations,
            sources,
            message: message.clone(),
        });
        self.message(&message)
    }
    pub(in crate::session) fn git_quit_answer(
        &mut self,
        accepted: bool,
        operations: Vec<OperationId>,
        sources: Sources,
    ) -> Action {
        self.input.cancel();
        if !accepted {
            self.cancel_git_shutdown();
            return self.message("Quit canceled; Git operations retained");
        }
        self.git.exit_operations = operations;
        self.finish_terminal_quit(sources)
    }
    pub(in crate::session) fn git_exit_permitted(&self) -> bool {
        self.git
            .engine
            .progress()
            .all(|p| self.git.exit_operations.contains(&p.operation))
    }
    pub(in crate::session) fn begin_git_shutdown(&mut self, sources: Sources) -> Action {
        if !sources.permits(&self.buffers) || !self.git_exit_permitted() {
            self.cancel_git_shutdown();
            return self
                .error("Source or owned Git work changed; quit canceled before Git termination");
        }
        if !self.git.engine.busy() {
            return self.finish_editor_quit();
        }
        // Consent is for captured identities only. A completed job cannot make
        // that consent apply to a replacement operation in the same context.
        let operations: Vec<_> = self.git.engine.progress().map(|p| p.operation).collect();
        for id in operations {
            if let Err(error) = self.git.engine.cancel(id) {
                self.cancel_git_shutdown();
                return self.error(&format!("Git exit cleanup could not start: {error}"));
            }
        }
        self.git.shutdown = Some(sources);
        self.wait_git_shutdown()
    }
    fn wait_git_shutdown(&mut self) -> Action {
        // Terminal cleanup can finish while its waiting prompt still owns
        // input. Transfer that coordinator focus before acquiring Git's wait.
        self.input.cancel();
        if let Err(error) = self.input.decision("Stopping owned Git operations") {
            self.cancel_git_shutdown();
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::GitShutdown {
            message: WAITING.into(),
        });
        self.message(WAITING)
    }
    pub(in crate::session) fn git_shutdown_answer(&mut self, accepted: bool) -> Action {
        if !accepted {
            self.cancel_git_shutdown();
            return self.message("Quit canceled; approved Git cleanup continues");
        }
        self.wait_git_shutdown()
    }
    pub(in crate::session) fn git_shutdown_active(&self) -> bool {
        self.git.shutdown.is_some()
    }
    pub(in crate::session) fn cancel_git_shutdown(&mut self) {
        self.git.shutdown = None;
        self.git.exit_operations.clear();
        if matches!(self.pending, Some(Pending::GitShutdown { .. })) {
            self.pending = None;
            self.input.cancel();
        }
    }
    pub(in crate::session) fn advance_git_shutdown(&mut self) -> Action {
        let Some(sources) = &self.git.shutdown else {
            return Action::Wait;
        };
        if !sources.permits(&self.buffers) {
            self.cancel_git_shutdown();
            return self
                .error("Source changed during Git cleanup; quit canceled, buffers retained");
        }
        if self.git.engine.cleanup_pending() {
            self.cancel_git_shutdown();
            return self.error(
                "Git cleanup failed; quit canceled and ownership retained in git-processes",
            );
        }
        if self.git.engine.busy() {
            return Action::Wait;
        }
        self.finish_editor_quit()
    }
}
