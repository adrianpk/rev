//! Bounded stash observations and exact, reviewed repository requests.
use super::*;
pub(in crate::git) mod worktrees;
pub use worktrees::{WorktreeAction, WorktreeEntry, WorktreePlan, Worktrees};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StashEntry {
    pub object: ObjectId,
    pub selector: Vec<u8>,
    pub subject: Vec<u8>,
}
#[derive(Debug)]
pub struct Stashes {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub entries: Vec<StashEntry>,
    pub completeness: Completeness,
    pub(super) _admission: Lease,
}
impl Stashes {
    pub fn entry(&self, object: ObjectId) -> Option<&StashEntry> {
        self.entries.iter().find(|e| e.object == object)
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        self.entries == other.entries && self.completeness == other.completeness
    }
}
#[derive(Debug)]
pub struct StashDetail {
    pub stashes: Arc<Stashes>,
    pub object: ObjectId,
    pub bytes: Vec<u8>,
    pub completeness: Completeness,
    pub(super) _admission: Lease,
}
#[derive(Clone, Debug)]
pub enum StashAction {
    Create { message: String },
    Apply(ObjectId),
    Pop(ObjectId),
    Branch { object: ObjectId, name: String },
    Drop(ObjectId),
}
impl StashAction {
    pub fn object(&self) -> Option<ObjectId> {
        match self {
            Self::Create { .. } => None,
            Self::Apply(o) | Self::Pop(o) | Self::Drop(o) | Self::Branch { object: o, .. } => {
                Some(*o)
            }
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Create { .. } => "create (tracked and untracked saved files)",
            Self::Apply(_) => "apply exact object; keep stash",
            Self::Pop(_) => "apply exact object, then checked drop on success",
            Self::Branch { .. } => {
                "create branch at stash base, apply exact object, then checked drop"
            }
            Self::Drop(_) => "drop exact stash entry",
        }
    }
    fn bytes(&self) -> usize {
        match self {
            Self::Create { message } => message.capacity(),
            Self::Branch { name, .. } => name.capacity(),
            _ => 0,
        }
    }
}
#[derive(Debug)]
pub struct StashPlan {
    pub stashes: Arc<Stashes>,
    pub source: Arc<Sequence>,
    pub action: StashAction,
    pub paths: Vec<PathBuf>,
    pub base: Option<ObjectId>,
    pub(super) evidence: [Vec<u8>; 3],
    pub(super) _admission: Lease,
}
pub(super) enum Request {
    Worktrees(worktrees::Request),
    Stashes(Arc<Observation>),
    Show {
        stashes: Arc<Stashes>,
        object: ObjectId,
    },
    Prepare {
        stashes: Arc<Stashes>,
        action: StashAction,
        _admission: Lease,
    },
    Stash(Arc<StashPlan>),
}
impl Repositories {
    pub fn stashes(&mut self, context: ContextId) -> Result<OperationId, String> {
        let observation = std::sync::Arc::clone(
            &self
                .context(context)
                .ok_or("Git context expired")?
                .observation,
        );
        let root = self.mutation_identity_root(&observation.identity)?;
        self.start(
            root,
            Some(context),
            work::Request::Repository(Request::Stashes(observation)),
        )
    }
    fn check_stashes(&self, stashes: &Arc<Stashes>) -> Result<PathBuf, String> {
        let context = self
            .context(stashes.observation.context)
            .ok_or("Stash context expired")?;
        if context.failure.is_some()
            || !context
                .stashes
                .as_ref()
                .is_some_and(|s| Arc::ptr_eq(s, stashes))
        {
            return Err("Stash observation expired; refresh the exact list".into());
        }
        self.mutation_identity_root(&stashes.observation.identity)
    }
    pub fn show_stash(
        &mut self,
        stashes: Arc<Stashes>,
        object: ObjectId,
    ) -> Result<OperationId, String> {
        let root = self.check_stashes(&stashes)?;
        stashes
            .entry(object)
            .ok_or("Select one exact stash object")?;
        self.start(
            root,
            Some(stashes.observation.context),
            work::Request::Repository(Request::Show { stashes, object }),
        )
    }
    pub fn prepare_stash(
        &mut self,
        stashes: Arc<Stashes>,
        action: StashAction,
    ) -> Result<OperationId, String> {
        let root = self.check_stashes(&stashes)?;
        if stashes.completeness != Completeness::Complete {
            return Err("Changing stash actions require a complete list".into());
        }
        if let Some(object) = action.object() {
            stashes
                .entry(object)
                .ok_or("Exact stash object disappeared")?;
            if matches!(
                action,
                StashAction::Pop(_) | StashAction::Branch { .. } | StashAction::Drop(_)
            ) && stashes
                .entries
                .iter()
                .filter(|e| e.object == object)
                .count()
                != 1
            {
                return Err("Duplicate stash objects make entry removal ambiguous; no changing request was started".into());
            }
        }
        match &action {
            StashAction::Create { message }
                if message.len() > 4096 || message.contains(['\0', '\n', '\r']) =>
            {
                return Err("Stash message requires at most 4096 bytes on one non-NUL line".into());
            }
            StashAction::Branch { name, .. }
                if name.is_empty() || name.len() > 4096 || name.contains('\0') =>
            {
                return Err("Stash branch requires 1..=4096 non-NUL bytes".into());
            }
            _ => {}
        }
        let admission = self.budget.reserve(action.bytes() + 4096)?;
        self.start(
            root,
            Some(stashes.observation.context),
            work::Request::Repository(Request::Prepare {
                stashes,
                action,
                _admission: admission,
            }),
        )
    }
    pub fn change_stash(&mut self, plan: Arc<StashPlan>) -> Result<OperationId, String> {
        let root = self.check_stashes(&plan.stashes)?;
        if !self
            .context(plan.stashes.observation.context)
            .and_then(|c| c.stash_plan.as_ref())
            .is_some_and(|p| Arc::ptr_eq(p, &plan))
        {
            return Err("Stash review expired".into());
        }
        self.start(
            root,
            Some(plan.stashes.observation.context),
            work::Request::Repository(Request::Stash(plan)),
        )
    }
}
