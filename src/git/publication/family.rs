use super::*;

impl Repositories {
    pub(super) fn receive_family_plan(
        &mut self,
        plan: FamilyPlan,
        update: &mut Update,
    ) -> Option<ContextId> {
        self.family_plan = Some(Arc::new(plan));
        update.published = true;
        update.context
    }

    pub(super) fn receive_family_result(
        &mut self,
        mut result: FamilyResult,
        update: &mut Update,
    ) -> Option<ContextId> {
        let source_refreshed = result.source.is_some();
        if let Some(observation) = result.source.take()
            && let Some(context) = self
                .contexts
                .iter_mut()
                .find(|c| c.id == observation.context)
        {
            context.observation = Arc::new(observation);
            context.failure = None;
        }
        if result.plan.family.direct()
            && !source_refreshed
            && let Some(expected) = &result.plan.observation
            && let Some(context) = self.contexts.iter_mut().find(|c| c.id == expected.context)
            && context.observation.generation == expected.generation
        {
            context.failure = Some(update.operation);
        }
        self.family_result = Some(Arc::new(result));
        self.family_plan = None;
        update.published = true;
        update.context
    }
}
