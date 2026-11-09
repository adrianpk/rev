use super::*;

impl Repositories {
    pub(super) fn receive_conflict_explanation(
        &mut self,
        snapshot: ConflictExplanation,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == snapshot.conflict.sequence.refs.observation.context)
            && context
                .conflict
                .as_ref()
                .is_some_and(|c| Arc::ptr_eq(c, &snapshot.conflict))
        {
            context.conflict_explanation = Some(Arc::new(snapshot));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic =
                Some("Conflict explanation context expired before publication".into());
        }
        update.context
    }

    pub(super) fn receive_resolution(
        &mut self,
        plan: Resolution,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == plan.conflict.sequence.refs.observation.context)
            && context
                .conflict
                .as_ref()
                .is_some_and(|c| Arc::ptr_eq(c, &plan.conflict))
        {
            context.resolution = Some(Arc::new(plan));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Conflict resolution context expired before review".into());
        }
        update.context
    }

    pub(super) fn receive_conflict(
        &mut self,
        conflict: Conflict,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        if let Some(context) = self
            .contexts
            .iter_mut()
            .find(|c| c.id == conflict.sequence.refs.observation.context)
            && context
                .sequence
                .as_ref()
                .is_some_and(|s| Arc::ptr_eq(s, &conflict.sequence))
        {
            context.conflict = Some(Arc::new(conflict));
            update.published = true;
        } else {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some("Conflict context expired before publication".into());
        }
        update.context
    }
}
