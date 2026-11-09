use super::*;

impl Session {
    pub(super) fn route_git_update(&mut self, update: &crate::git::Update) -> bool {
        self.route_git_repository_update(update)
    }
    fn route_git_repository_update(&mut self, update: &crate::git::Update) -> bool {
        if matches!(
            update.kind,
            crate::git::OperationKind::FamilyPrepare | crate::git::OperationKind::Family
        ) {
            self.publish_git_family(update);
            return true;
        }
        if self.git.worktrees.request == Some(update.operation)
            || self.git.worktrees.preparing == Some(update.operation)
        {
            self.publish_git_worktrees(update);
            return true;
        }
        if self.git.repository.request == Some(update.operation)
            || self.git.repository.preparing == Some(update.operation)
        {
            self.publish_git_repository(update);
            return true;
        }
        self.route_git_sequence_update(update)
    }
    fn route_git_sequence_update(&mut self, update: &crate::git::Update) -> bool {
        if self
            .git
            .refs
            .bisect_preparing
            .is_some_and(|(id, _)| id == update.operation)
        {
            self.publish_git_bisect_preparation(update);
            return true;
        }
        if self.git.refs.application_preparing == Some(update.operation) {
            self.publish_git_application_preparation(update);
            return true;
        }
        if self.git.refs.autosquash_preparing == Some(update.operation) {
            self.publish_git_autosquash_preparation(update);
            return true;
        }
        if self.git.conflicts.explaining == Some(update.operation) {
            self.publish_git_explanation(update);
            return true;
        }
        if self.git.conflicts.preparing == Some(update.operation) {
            self.publish_git_resolution_preparation(update);
            return true;
        }
        if self.git.conflicts.resolving == Some(update.operation) {
            self.publish_git_resolution(update);
            return true;
        }
        if self.git.conflicts.request == Some(update.operation) {
            self.publish_git_conflict(update);
            return true;
        }
        if self.git.refs.integration_request == Some(update.operation) {
            self.publish_git_integration(update);
            return true;
        }
        self.route_git_refs_update(update)
    }
    fn route_git_refs_update(&mut self, update: &crate::git::Update) -> bool {
        if self.git.refs.push_preparing == Some(update.operation) {
            self.publish_git_push_preparation(update);
            return true;
        }
        if self.git.refs.tag_preparing == Some(update.operation) {
            self.publish_git_tag_preparation(update);
            return true;
        }
        if self.git.refs.request == Some(update.operation) {
            self.publish_git_refs(update);
            return true;
        }
        if self
            .git
            .ancestry
            .request
            .is_some_and(|(id, _)| id == update.operation)
        {
            self.publish_git_history(update);
            return true;
        }
        self.route_git_editing_update(update)
    }
    fn route_git_editing_update(&mut self, update: &crate::git::Update) -> bool {
        if self.git.commit_preparing == Some(update.operation) {
            self.publish_git_preparation(update);
            return true;
        }
        if self.git.commit_request == Some(update.operation) {
            self.publish_git_commit(update);
            return true;
        }
        if self.git.discard_request == Some(update.operation) {
            self.publish_git_discard(update);
            return true;
        }
        if self.git.index_request == Some(update.operation) {
            self.publish_git_index(update);
            return true;
        }
        if self
            .git
            .inspection
            .request
            .as_ref()
            .is_some_and(|r| r.operation == update.operation)
        {
            self.publish_git_diff(update);
            self.finish_git_index_preparation(update);
            return true;
        }
        false
    }
    pub(super) fn publish_git_status(&mut self, update: &crate::git::Update) -> Option<Action> {
        if self
            .git
            .state
            .request
            .is_none_or(|(request, _)| request != update.operation)
        {
            return None;
        }
        let opening = self.git.state.request.take().expect("Git request").1;
        if update.published {
            self.publish_git_status_observation(update);
            None
        } else {
            self.publish_git_status_failure(update, opening)
        }
    }
    fn publish_git_status_observation(&mut self, update: &crate::git::Update) {
        if self.git.state.context != update.context {
            self.git.state.selected = None;
            self.git.state.collapsed = [false; 4];
            self.git.state.top = 0;
            self.git.inspection = comparisons::Inspection::default();
        }
        self.git.state.context = update.context;
        self.git.reconcile(self.git_extent());
        if let Some(resume) = self.git.inspection.after_refresh.take() {
            self.request_git_diff(&resume);
            return;
        }
        if self.git.visible && self.input.focus() == Focus::Editing && self.pending.is_none() {
            self.message("Git Status refreshed");
        }
    }
    fn publish_git_status_failure(
        &mut self,
        update: &crate::git::Update,
        opening: bool,
    ) -> Option<Action> {
        if opening && self.git.visible && self.git.page == Page::Status {
            self.git.visible = false;
            self.input.set_git(false);
            self.sync_input_context();
            let diagnostic = self
                .git
                .engine
                .records()
                .iter()
                .find(|r| r.id == update.operation)
                .and_then(|r| r.diagnostic.as_deref())
                .unwrap_or("Repository observation unavailable");
            let error = format!("Git unavailable: {}", label(diagnostic.as_bytes()));
            return Some(self.error(&error));
        } else if self.git.visible && self.input.focus() == Focus::Editing && self.pending.is_none()
        {
            self.git.inspection.after_refresh = None;
            self.message("Git refresh failed; confirmed observation retained");
        }
        None
    }
}
