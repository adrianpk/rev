use super::*;

impl Repositories {
    pub(super) fn receive_bisect_plan(
        &mut self,
        plan: BisectPlan,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self.contexts.iter_mut().find(|c| {
            c.id == plan.refs.observation.context
                && c.refs.as_ref().is_some_and(|r| Arc::ptr_eq(r, &plan.refs))
        }) {
            context.bisect_plan = Some(Arc::new(plan));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Bisect source expired before publication".into());
        }
        update.context
    }

    pub(super) fn receive_application_plan(
        &mut self,
        plan: ApplyPlan,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self.contexts.iter_mut().find(|c| {
            c.id == plan.refs.observation.context
                && Arc::ptr_eq(&c.observation, &plan.refs.observation)
        }) {
            context.refs = Some(std::sync::Arc::clone(&plan.refs));
            context.application_plan = Some(Arc::new(plan));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Ordered revision source expired before publication".into());
        }
        update.context
    }

    pub(super) fn receive_autosquash(
        &mut self,
        plan: AutosquashPlan,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self.contexts.iter_mut().find(|c| {
            c.id == plan.refs.observation.context
                && c.refs.as_ref().is_some_and(|r| Arc::ptr_eq(r, &plan.refs))
        }) {
            context.autosquash = Some(Arc::new(plan));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Autosquash source expired before publication".into());
        }
        update.context
    }

    pub(super) fn receive_sequence(
        &mut self,
        sequence: Sequence,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == sequence.refs.observation.context)
        {
            context.observation = std::sync::Arc::clone(&sequence.refs.observation);
            context.refs = Some(std::sync::Arc::clone(&sequence.refs));
            context.sequence = Some(Arc::new(sequence));
            context.resolution = None;
            context.failure = None;
            context.composition = None;
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Sequence context expired before publication".into());
        }
        update.context
    }
}
