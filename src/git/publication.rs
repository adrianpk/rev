use super::work::Publication;
use super::*;

mod conflicts;
mod family;
mod integration;
mod observation;
mod repository;

impl Repositories {
    pub(super) fn publish_finished(
        &mut self,
        result: Result<Publication, String>,
        progress: Progress,
        record: &mut Record,
    ) -> Update {
        let mut update = Update {
            operation: record.id,
            kind: progress.kind,
            context: progress.context,
            published: false,
        };
        match result {
            Ok(result) => update.context = self.receive_family(result, &mut update, record),
            Err(_) => self.fail_observation(&update, record),
        }
        update
    }

    fn receive_family(
        &mut self,
        result: Publication,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        match result {
            Publication::FamilyPlan(plan) => self.receive_family_plan(plan, update),
            Publication::FamilyResult(result) => self.receive_family_result(result, update),
            Publication::Status(observation) => self.receive_status(observation, update, record),
            result => self.receive_repository(result, update, record),
        }
    }

    fn receive_repository(
        &mut self,
        result: Publication,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        match result {
            Publication::Worktrees(worktrees) => self.receive_worktrees(worktrees, update),
            Publication::WorktreePlan(plan) => self.receive_worktree_plan(plan, update),
            Publication::Stashes(stashes) => self.receive_stashes(stashes, update),
            Publication::StashDetail(detail) => self.receive_stash_detail(detail, update),
            Publication::StashPlan(plan) => self.receive_stash_plan(plan, update),
            Publication::StashResult { stashes, sequence } => {
                self.receive_stash_result(stashes, sequence, update)
            }
            result => self.receive_integration(result, update, record),
        }
    }

    fn receive_integration(
        &mut self,
        result: Publication,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        match result {
            Publication::BisectPlan(plan) => self.receive_bisect_plan(plan, update, record),
            Publication::ApplicationPlan(plan) => {
                self.receive_application_plan(plan, update, record)
            }
            Publication::Autosquash(plan) => self.receive_autosquash(plan, update, record),
            Publication::Sequence(sequence) => self.receive_sequence(sequence, update, record),
            result => self.receive_conflicts(result, update, record),
        }
    }

    fn receive_conflicts(
        &mut self,
        result: Publication,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        match result {
            Publication::ConflictExplanation(snapshot) => {
                self.receive_conflict_explanation(snapshot, update, record)
            }
            Publication::Resolution(plan) => self.receive_resolution(plan, update, record),
            Publication::Conflict(conflict) => self.receive_conflict(conflict, update, record),
            result => self.receive_observation(result, update, record),
        }
    }

    fn receive_observation(
        &mut self,
        result: Publication,
        update: &mut Update,
        record: &mut Record,
    ) -> Option<ContextId> {
        match result {
            Publication::Refs { refs, changed } => self.receive_refs(refs, changed, update, record),
            Publication::Diff(detail) => self.receive_diff(detail, update, record),
            Publication::History(result) => self.receive_history(result, update, record),
            Publication::PushIntent(intent) => self.receive_push_intent(intent, update, record),
            Publication::TagIntent(intent) => self.receive_tag_intent(intent, update, record),
            Publication::Composition(composition) => {
                self.receive_composition(composition, update, record)
            }
            _ => unreachable!("publication dispatched by operation family"),
        }
    }

    fn fail_observation(&mut self, update: &Update, record: &Record) {
        let Some(context) = update
            .context
            .and_then(|id| self.contexts.iter_mut().find(|c| c.id == id))
        else {
            return;
        };
        if update.kind == OperationKind::Diff {
            context.detail_failure = Some(record.id);
        } else if OBSERVATION_FAILURES.contains(&update.kind) {
            context.failure = Some(record.id);
        }
    }

    pub(super) fn fail_cleanup(&mut self, progress: &Progress) {
        if CLEANUP_FAILURES.contains(&progress.kind)
            && let Some(context) = progress
                .context
                .and_then(|id| self.contexts.iter_mut().find(|c| c.id == id))
        {
            context.failure = Some(progress.operation);
        }
    }
}

const CLEANUP_FAILURES: &[OperationKind] = &[
    OperationKind::Family,
    OperationKind::Worktree,
    OperationKind::Stash,
    OperationKind::Stage,
    OperationKind::Unstage,
    OperationKind::Discard,
    OperationKind::Commit,
    OperationKind::TagCreate,
    OperationKind::TagDelete,
    OperationKind::SequenceControl,
    OperationKind::Application,
    OperationKind::ApplicationControl,
    OperationKind::Bisect,
    OperationKind::Integration,
    OperationKind::Push,
    OperationKind::Fetch,
    OperationKind::Branch,
];

const OBSERVATION_FAILURES: &[OperationKind] = &[
    OperationKind::Worktrees,
    OperationKind::Stashes,
    OperationKind::Status,
    OperationKind::Refs,
    OperationKind::Family,
    OperationKind::Worktree,
    OperationKind::Stash,
    OperationKind::Stage,
    OperationKind::ConflictResolve,
    OperationKind::Unstage,
    OperationKind::Discard,
    OperationKind::Commit,
    OperationKind::TagCreate,
    OperationKind::TagDelete,
    OperationKind::SequenceControl,
    OperationKind::Application,
    OperationKind::ApplicationControl,
    OperationKind::Bisect,
    OperationKind::Integration,
    OperationKind::Push,
    OperationKind::Fetch,
    OperationKind::Branch,
];
