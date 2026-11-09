use super::*;

impl Repositories {
    pub(super) fn receive_worktrees(
        &mut self,
        worktrees: Worktrees,
        update: &mut Update,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == worktrees.observation.context)
        {
            context.observation = std::sync::Arc::clone(&worktrees.observation);
            context.worktrees = Some(Arc::new(worktrees));
            context.worktree_plan = None;
            context.failure = None;
            update.published = true;
        }
        update.context
    }

    pub(super) fn receive_worktree_plan(
        &mut self,
        plan: WorktreePlan,
        update: &mut Update,
    ) -> Option<ContextId> {
        if let Some(context) = self.contexts.iter_mut().find(|c| {
            c.id == plan.worktrees.observation.context
                && c.worktrees
                    .as_ref()
                    .is_some_and(|w| Arc::ptr_eq(w, &plan.worktrees))
        }) {
            context.worktree_plan = Some(Arc::new(plan));
            update.published = true;
        }
        update.context
    }

    pub(super) fn receive_stashes(
        &mut self,
        stashes: Stashes,
        update: &mut Update,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == stashes.observation.context)
        {
            context.observation = std::sync::Arc::clone(&stashes.observation);
            context.stashes = Some(Arc::new(stashes));
            context.stash_plan = None;
            context.failure = None;
            update.published = true;
        }
        update.context
    }

    pub(super) fn receive_stash_detail(
        &mut self,
        detail: StashDetail,
        update: &mut Update,
    ) -> Option<ContextId> {
        if let Some(context) = self.contexts.iter_mut().find(|c| {
            c.id == detail.stashes.observation.context
                && c.stashes
                    .as_ref()
                    .is_some_and(|s| Arc::ptr_eq(s, &detail.stashes))
        }) {
            context.stash_detail = Some(Arc::new(detail));
            update.published = true;
        }
        update.context
    }

    pub(super) fn receive_stash_plan(
        &mut self,
        plan: StashPlan,
        update: &mut Update,
    ) -> Option<ContextId> {
        if let Some(context) = self.contexts.iter_mut().find(|c| {
            c.id == plan.stashes.observation.context
                && c.stashes
                    .as_ref()
                    .is_some_and(|s| Arc::ptr_eq(s, &plan.stashes))
        }) {
            context.stash_plan = Some(Arc::new(plan));
            update.published = true;
        }
        update.context
    }

    pub(super) fn receive_stash_result(
        &mut self,
        stashes: Stashes,
        sequence: Sequence,
        update: &mut Update,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == stashes.observation.context)
        {
            context.observation = std::sync::Arc::clone(&stashes.observation);
            context.refs = Some(std::sync::Arc::clone(&sequence.refs));
            context.sequence = Some(Arc::new(sequence));
            context.stashes = Some(Arc::new(stashes));
            context.stash_plan = None;
            context.stash_detail = None;
            context.failure = None;
            context.composition = None;
            update.published = true;
        }
        update.context
    }
}
