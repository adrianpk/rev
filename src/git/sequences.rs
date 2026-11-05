//! Bounded ordered revision requests; Git owns each actual changing step.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyKind {
    CherryPick,
    Revert,
}
impl ApplyKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::CherryPick => "cherry-pick",
            Self::Revert => "revert",
        }
    }
    pub fn family(self) -> SequenceFamily {
        match self {
            Self::CherryPick => SequenceFamily::CherryPick,
            Self::Revert => SequenceFamily::Revert,
        }
    }
}
/// An explicit one-based mainline position is required only for a merge.
/// Preparation binds that position to its full immutable parent object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RevisionChoice {
    pub object: ObjectId,
    pub mainline: Option<usize>,
}
#[derive(Debug)]
pub struct ApplyStep {
    pub object: ObjectId,
    pub mainline: Option<usize>,
    pub parent: Option<ObjectId>,
    pub subject: Vec<u8>,
    pub(in crate::git) message: Vec<u8>,
    pub(in crate::git) author: Vec<u8>,
}
#[derive(Debug)]
pub struct ApplyPlan {
    pub refs: Arc<Refs>,
    pub history: Arc<History>,
    pub kind: ApplyKind,
    pub steps: Vec<ApplyStep>,
    pub(in crate::git) _admission: Lease,
}
impl ApplyPlan {
    pub(in crate::git) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.steps.capacity() * std::mem::size_of::<ApplyStep>()
            + self
                .steps
                .iter()
                .map(|s| s.subject.capacity() + s.message.capacity() + s.author.capacity())
                .sum::<usize>()
    }
}
/// Retained authority for the finite reviewed request, not a persistent queue.
/// The exact stopped Git target, HEAD and branch must still match this lineage.
#[derive(Debug)]
pub struct ApplyProgress {
    pub plan: Arc<ApplyPlan>,
    pub next: usize,
    pub before: ObjectId,
    pub completed: Vec<ObjectId>,
    pub(in crate::git) _admission: Lease,
}
impl ApplyProgress {
    pub(in crate::git) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.completed.capacity() * std::mem::size_of::<ObjectId>()
    }
}
pub(super) enum Request {
    Prepare {
        history: Arc<History>,
        kind: ApplyKind,
        choices: Vec<RevisionChoice>,
        _admission: Lease,
    },
    Start(Arc<ApplyPlan>),
}
impl Repositories {
    /// Prepare the exact user order from current admitted History. No changing
    /// Git child starts until the returned plan receives explicit confirmation.
    pub fn prepare_sequence(
        &mut self,
        history: Arc<History>,
        kind: ApplyKind,
        choices: &[RevisionChoice],
    ) -> Result<OperationId, String> {
        self.check_history(&history)?;
        self.history_directory(&history.observation)?;
        if choices.is_empty() || choices.len() > 256 {
            return Err("Ordered revisions require 1..256 exact targets".into());
        }
        for (i, choice) in choices.iter().enumerate() {
            if choices[..i].iter().any(|c| c.object == choice.object) {
                return Err("Ordered revisions contain a duplicate target".into());
            }
            let revision = history
                .revision(choice.object)
                .ok_or("Ordered target is not in the captured History")?;
            if revision.parents.len() > 1 {
                if choice
                    .mainline
                    .is_none_or(|n| n == 0 || n > revision.parents.len())
                {
                    return Err("Merge revision requires one explicit valid mainline parent".into());
                }
            } else if choice.mainline.is_some() {
                return Err("A non-merge revision has no selectable mainline".into());
            }
        }
        let root = self.mutation_identity_root(&history.observation.identity)?;
        let admission = self.budget.reserve(std::mem::size_of_val(choices) + 4096)?;
        self.start(
            root,
            Some(history.observation.context),
            work::Request::Application(Request::Prepare {
                history,
                kind,
                choices: choices.into(),
                _admission: admission,
            }),
        )
    }
    /// Caller confirms the entire finite plan and guards visiting sources.
    /// The worker checks exact refs, clean disk/index and immutable source IDs.
    pub fn apply_sequence(&mut self, plan: Arc<ApplyPlan>) -> Result<OperationId, String> {
        self.check_refs(&plan.refs)?;
        if !self
            .context(plan.refs.observation.context)
            .and_then(|c| c.application_plan.as_ref())
            .is_some_and(|p| Arc::ptr_eq(p, &plan))
        {
            return Err("Ordered revision plan expired; prepare it again".into());
        }
        let root = self.mutation_identity_root(&plan.refs.observation.identity)?;
        self.start(
            root,
            Some(plan.refs.observation.context),
            work::Request::Application(Request::Start(plan)),
        )
    }
}
