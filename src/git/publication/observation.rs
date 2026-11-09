use super::*;

impl Repositories {
    pub(super) fn receive_status(
        &mut self,
        mut observation: Observation,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        let prior = self
            .contexts
            .iter()
            .position(|c| c.observation.identity == observation.identity);
        if let Some(prior) = prior {
            observation.context = self.contexts[prior].id;
            let id = observation.context;
            self.contexts[prior].observation = Arc::new(observation);
            self.contexts[prior].failure = None;
            self.contexts[prior].composition = None;
            if update.kind == OperationKind::Commit && record.outcome == Outcome::Verified {
                self.contexts[prior].draft = None;
            }
            update.published = true;
            record.context = Some(id);
            Some(id)
        } else if self.contexts.len() < MAX_CONTEXTS {
            let id = observation.context;
            self.contexts.push(Context {
                id,
                observation: Arc::new(observation),
                failure: None,
                detail: None,
                detail_failure: None,
                worktrees: None,
                worktree_plan: None,
                stashes: None,
                stash_detail: None,
                stash_plan: None,
                refs: None,
                sequence: None,
                conflict: None,
                resolution: None,
                conflict_explanation: None,
                autosquash: None,
                application_plan: None,
                bisect_plan: None,
                history: None,
                reflog: None,
                blame: None,
                revision_file: None,
                revision: None,
                tag_intent: None,
                push_intent: None,
                tag_draft: None,
                draft: None,
                composition: None,
            });
            update.published = true;
            record.context = Some(id);
            Some(id)
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic =
                Some("Git retained context limit reached; existing contexts were preserved".into());
            None
        }
    }

    pub(super) fn receive_refs(
        &mut self,
        mut refs: Refs,
        changed: bool,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == refs.observation.context)
            && (changed
                || (Arc::ptr_eq(&context.observation, &refs.observation)
                    && context.failure.is_none()))
        {
            if changed {
                context.observation = std::sync::Arc::clone(&refs.observation);
                context.failure = None;
                context.composition = None;
            }
            if update.kind == OperationKind::TagCreate && record.outcome == Outcome::Verified {
                context.tag_draft = None;
            }
            refs.id = record.id;
            context.refs = Some(Arc::new(refs));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Refs generation changed before publication".into());
        }
        update.context
    }

    pub(super) fn receive_diff(
        &mut self,
        detail: Diff,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        let context = self
            .contexts
            .iter_mut()
            .find(|context| context.id == detail.observation.context);
        if let Some(context) = context
            && context.observation.generation == detail.observation.generation
            && context.failure.is_none()
        {
            context.detail = Some(Arc::new(detail));
            context.detail_failure = None;
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Git comparison generation changed before publication".into());
        }
        update.context
    }

    pub(super) fn receive_history(
        &mut self,
        result: history::ResultData,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        let observation = result.observation();
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == observation.context)
            && Arc::ptr_eq(&context.observation, observation)
            && context.failure.is_none()
        {
            match result {
                history::ResultData::Reflog(r) => context.reflog = Some(Arc::new(r)),
                history::ResultData::Blame(b) => context.blame = Some(Arc::new(b)),
                history::ResultData::File(f) => context.revision_file = Some(Arc::new(f)),
                history::ResultData::Log(h) => {
                    context.history = Some(Arc::new(h));
                }
                history::ResultData::Show(d) => {
                    context.revision = Some(Arc::new(d));
                }
            }
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("History generation changed before publication".into());
        }
        update.context
    }

    pub(super) fn receive_push_intent(
        &mut self,
        intent: PushIntent,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == intent.refs().observation.context)
            && Arc::ptr_eq(&context.observation, &intent.refs().observation)
            && context.failure.is_none()
        {
            context.push_intent = Some(Arc::new(intent));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Push preparation generation changed".into());
        }
        update.context
    }

    pub(super) fn receive_tag_intent(
        &mut self,
        intent: TagIntent,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == intent.refs.observation.context)
            && Arc::ptr_eq(&context.observation, &intent.refs.observation)
            && context.failure.is_none()
        {
            context.tag_intent = Some(Arc::new(intent));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Tag preparation generation changed".into());
        }
        update.context
    }

    pub(super) fn receive_composition(
        &mut self,
        composition: Composition,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == composition.observation.context)
            && context.observation.generation == composition.observation.generation
            && context.failure.is_none()
        {
            context.composition = Some(Arc::new(composition));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic =
                Some("Commit preparation generation changed before publication".into());
        }
        update.context
    }
}
