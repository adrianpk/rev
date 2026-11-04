use super::*;
use crate::session::Pending;

impl Session {
    pub(in crate::session::git) fn prepare_git_amend(&mut self) -> Action {
        let result = self
            .commit_context()
            .and_then(|context| self.git.engine.prepare_amend(context));
        self.git_preparation_started(result)
    }

    pub(in crate::session::git) fn prompt_git_fixup(&mut self) -> Action {
        if self.git.page == Page::History {
            let target = self.git.ancestry.selected.filter(|id| {
                self.git.ancestry.request.is_none()
                    && !self.git.ancestry.failed
                    && self.git.history().is_some_and(|h| {
                        h.revision(*id).is_some()
                            && self
                                .git
                                .observation()
                                .is_some_and(|o| std::ptr::eq(o, h.observation.as_ref()))
                    })
            });
            let Some(target) = target else {
                return self.message("Select an exact History revision for fixup");
            };
            let result = self
                .commit_context()
                .and_then(|context| self.git.engine.commit_observation(context))
                .and_then(|observation| self.git.engine.prepare_fixup(observation, &target.hex()));
            return self.git_preparation_started(result);
        }
        let result = self
            .commit_context()
            .and_then(|context| self.git.engine.commit_observation(context));
        let observation = match result {
            Ok(o) => o,
            Err(e) => return self.error(&e),
        };
        if let Err(error) = self.input.argument("Fixup target revision: ", "") {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::GitFixup(observation));
        self.message("Enter one fixup revision; preparation resolves one full immutable commit")
    }

    pub(in crate::session) fn git_fixup_argument(
        &mut self,
        observation: Arc<Observation>,
        revision: &str,
    ) -> Action {
        let result = self.git.engine.prepare_fixup(observation, revision);
        self.git_preparation_started(result)
    }

    fn commit_context(&self) -> Result<ContextId, String> {
        if !self.git.visible
            || !matches!(self.git.page, Page::Status | Page::History)
            || self.windows.retained_view_active()
            || self.layout_active()
        {
            return Err(
                "Commit preparation requires Git Status and an available retained source view"
                    .into(),
            );
        }
        self.git
            .state
            .context
            .ok_or("Git Status is still checking context".into())
    }

    fn git_preparation_started(&mut self, result: Result<OperationId, String>) -> Action {
        match result {
            Ok(operation) => {
                self.git.commit_preparing = Some(operation);
                self.message("Preparing exact commit composition; no repository change authorized")
            }
            Err(error) => self.error(&error),
        }
    }

    pub(in crate::session::git) fn publish_git_preparation(&mut self, update: &crate::git::Update) {
        if self
            .git
            .engine
            .progress()
            .all(|p| p.operation != update.operation)
        {
            self.git.commit_preparing = None;
        }
        if !self.git.visible
            || !matches!(self.git.page, Page::Status | Page::History)
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let composition = update
            .context
            .filter(|_| update.published)
            .and_then(|id| self.git.engine.composition(id))
            .cloned();
        let Some(composition) = composition else {
            self.message("Commit preparation failed; $ inspects actual diagnostics");
            return;
        };
        let target = match composition.mode {
            crate::git::CommitMode::Amend { head } => format!("Amend exact HEAD {}", head.hex()),
            crate::git::CommitMode::Fixup { target } => {
                format!("Fixup exact target {}", target.hex())
            }
        };
        match self.install_git_commit(Some(composition)) {
            Ok(()) => {
                self.message(&format!(
                    "{target}; edit message, then C-Enter opens exact review"
                ));
            }
            Err(error) => {
                self.error(&error);
            }
        }
    }
}
