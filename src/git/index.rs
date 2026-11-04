//! Generation-bound index requests. Selection is immutable comparison data;
//! neither display rows nor escaped path labels supply mutation authority.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexDirection {
    Stage,
    Unstage,
}
impl IndexDirection {
    pub(super) fn kind(self) -> OperationKind {
        match self {
            Self::Stage => OperationKind::Stage,
            Self::Unstage => OperationKind::Unstage,
        }
    }
}

#[derive(Clone, Debug)]
pub enum IndexTarget {
    Path(Arc<Diff>),
    Hunk { diff: Arc<Diff>, hunk: HunkId },
}
impl IndexTarget {
    pub fn diff(&self) -> &Arc<Diff> {
        match self {
            Self::Path(diff) | Self::Hunk { diff, .. } => diff,
        }
    }
}

pub(super) struct Request {
    pub direction: IndexDirection,
    pub target: IndexTarget,
}

impl Repositories {
    /// Launch one checked disk/index action. No visiting buffer is saved.
    /// Complete comparison evidence is revalidated on the worker before any
    /// index-changing child; possible effects invalidate prior authority.
    pub fn change_index(
        &mut self,
        direction: IndexDirection,
        target: IndexTarget,
    ) -> Result<OperationId, String> {
        let diff = target.diff();
        if !self.detail_current(diff)
            || diff
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete)
        {
            return Err("Index action requires a complete current comparison and Status".into());
        }
        let allowed = match direction {
            IndexDirection::Stage => matches!(diff.group, Group::Untracked | Group::Unstaged),
            IndexDirection::Unstage => diff.group == Group::Staged,
        };
        if !allowed || matches!(diff.kind, DiffKind::Unmerged | DiffKind::Malformed(_)) {
            return Err("Selected path/layer does not support this index direction".into());
        }
        if let IndexTarget::Hunk { hunk, .. } = &target
            && (!diff.supports_hunks() || diff.hunk(*hunk).is_none())
        {
            return Err("Selected hunk is unsupported or belongs to another comparison".into());
        }
        let identity = &diff.observation.identity;
        if self.progress().any(|p| {
            p.context.is_none()
                || p.context.and_then(|id| self.context(id)).is_some_and(|c| {
                    c.observation.identity.common_directory == identity.common_directory
                })
        }) {
            return Err("Repository common storage already owns an active Git operation".into());
        }
        let root = identity
            .worktree
            .as_ref()
            .ok_or("Index actions require a working tree")?
            .clone();
        let context = diff.observation.context;
        self.start(
            root,
            Some(context),
            work::Request::Index(Request { direction, target }),
        )
    }
}
