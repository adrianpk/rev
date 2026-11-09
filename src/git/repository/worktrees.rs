use super::*;

/// Exact Git path and per-worktree metadata stay distinct from common storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeEntry {
    pub path: PathBuf,
    pub git_directory: PathBuf,
    pub head: Option<ObjectId>,
    pub branch: Vec<u8>,
    pub bare: bool,
    pub detached: bool,
    pub locked: Option<Vec<u8>>,
    pub prunable: Option<Vec<u8>>,
    pub available: bool,
    pub(in crate::git) directory_stamp: [u64; 8],
    pub(in crate::git) root_stamp: Option<[u64; 8]>,
}
impl WorktreeEntry {
    pub(in crate::git) fn same_record(&self, other: &Self) -> bool {
        self.path == other.path
            && self.git_directory == other.git_directory
            && self.head == other.head
            && self.branch == other.branch
            && self.bare == other.bare
            && self.detached == other.detached
            && self.locked == other.locked
            && self.prunable == other.prunable
            && self.available == other.available
    }
}
#[derive(Debug)]
pub struct Worktrees {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub entries: Vec<WorktreeEntry>,
    pub completeness: Completeness,
    pub(in crate::git) _admission: Lease,
}
impl Worktrees {
    pub fn entry(&self, path: &Path) -> Option<&WorktreeEntry> {
        self.entries.iter().find(|e| e.path == path)
    }
    pub(in crate::git) fn same(&self, other: &Self) -> bool {
        self.entries == other.entries && self.completeness == other.completeness
    }
}
#[derive(Clone, Debug)]
pub enum WorktreeAction {
    Create {
        path: PathBuf,
        branch: String,
        target: ObjectId,
    },
    Lock {
        path: PathBuf,
        reason: String,
    },
    Unlock(PathBuf),
    Remove(PathBuf),
    Prune,
}
impl WorktreeAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Create { .. } => "create new branch and linked worktree",
            Self::Lock { .. } => "lock with reason",
            Self::Unlock(_) => "unlock",
            Self::Remove(_) => "remove without force",
            Self::Prune => "prune reviewed missing unlocked records",
        }
    }
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Create { path, .. }
            | Self::Lock { path, .. }
            | Self::Unlock(path)
            | Self::Remove(path) => Some(path),
            Self::Prune => None,
        }
    }
    fn bytes(&self) -> usize {
        self.path().map_or(0, |p| p.as_os_str().len())
            + match self {
                Self::Create { branch, .. } => branch.capacity(),
                Self::Lock { reason, .. } => reason.capacity(),
                _ => 0,
            }
    }
}
#[derive(Debug)]
pub struct WorktreePlan {
    pub worktrees: Arc<Worktrees>,
    pub source: Arc<Sequence>,
    pub action: WorktreeAction,
    pub paths: Vec<PathBuf>,
    pub target_state: Option<Arc<Sequence>>,
    pub(in crate::git) evidence: [Vec<u8>; 3],
    pub(in crate::git) parent_stamp: Option<[u64; 8]>,
    pub(in crate::git) _admission: Lease,
}
pub(in crate::git) enum Request {
    List(Arc<Observation>),
    Visit {
        worktrees: Arc<Worktrees>,
        path: PathBuf,
        _admission: Lease,
    },
    Prepare {
        worktrees: Arc<Worktrees>,
        action: WorktreeAction,
        _admission: Lease,
    },
    Change(Arc<WorktreePlan>),
}
impl Repositories {
    fn worktree_directory(&self, identity: &Identity) -> Result<PathBuf, String> {
        if self.progress().any(|p| {
            p.context.is_none()
                || p.context.and_then(|c| self.context(c)).is_some_and(|c| {
                    c.observation.identity.common_directory == identity.common_directory
                })
        }) {
            return Err("Common repository storage already owns active Git work".into());
        }
        Ok(identity
            .worktree
            .as_ref()
            .unwrap_or(&identity.git_directory)
            .clone())
    }
    pub fn worktrees(&mut self, context: ContextId) -> Result<OperationId, String> {
        let observation = std::sync::Arc::clone(
            &self
                .context(context)
                .ok_or("Worktree context expired")?
                .observation,
        );
        let root = self.worktree_directory(&observation.identity)?;
        self.start(
            root,
            Some(context),
            work::Request::Repository(super::Request::Worktrees(Request::List(observation))),
        )
    }
    fn check_worktrees(&self, worktrees: &Arc<Worktrees>) -> Result<PathBuf, String> {
        let context = self
            .context(worktrees.observation.context)
            .ok_or("Worktree context expired")?;
        if context.failure.is_some()
            || !context
                .worktrees
                .as_ref()
                .is_some_and(|w| Arc::ptr_eq(w, worktrees))
        {
            return Err("Worktree observation expired; refresh exact paths".into());
        }
        self.worktree_directory(&worktrees.observation.identity)
    }
    pub fn visit_worktree(
        &mut self,
        worktrees: Arc<Worktrees>,
        path: &Path,
    ) -> Result<OperationId, String> {
        let root = self.check_worktrees(&worktrees)?;
        let entry = worktrees.entry(path).ok_or("Select one exact worktree")?;
        if !entry.available || entry.bare || entry.prunable.is_some() {
            return Err("Selected worktree is unavailable for a checked Status visit".into());
        }
        if self.contexts.len() == MAX_CONTEXTS
            && !self
                .contexts
                .iter()
                .any(|c| c.observation.identity.worktree.as_deref() == Some(path))
        {
            return Err(
                "Retained repository context limit reached; existing sources were preserved".into(),
            );
        }
        let admission = self.budget.reserve(path.as_os_str().len() + 512)?;
        // A visit discovers a distinct context. It must never reuse its source ID.
        self.start(
            root,
            None,
            work::Request::Repository(super::Request::Worktrees(Request::Visit {
                worktrees,
                path: path.into(),
                _admission: admission,
            })),
        )
    }
    pub fn prepare_worktree(
        &mut self,
        worktrees: Arc<Worktrees>,
        action: WorktreeAction,
    ) -> Result<OperationId, String> {
        let root = self.check_worktrees(&worktrees)?;
        if worktrees.completeness != Completeness::Complete {
            return Err("Changing worktree actions require a complete list".into());
        }
        if action
            .path()
            .is_some_and(|p| p.as_os_str().len() > 4096 || !p.is_absolute())
        {
            return Err("Worktree action requires an absolute path within 4096 bytes".into());
        }
        match &action {
            WorktreeAction::Create { branch, .. }
                if branch.is_empty() || branch.len() > 4096 || branch.contains('\0') =>
            {
                return Err("New worktree branch requires 1..=4096 non-NUL bytes".into());
            }
            WorktreeAction::Lock { reason, .. }
                if reason.is_empty()
                    || reason.len() > 4096
                    || reason.contains(['\0', '\r', '\n']) =>
            {
                return Err(
                    "Worktree lock reason requires 1..=4096 non-NUL bytes on one line".into(),
                );
            }
            _ => {}
        }
        let admission = self.budget.reserve(action.bytes() + 4096)?;
        self.start(
            root,
            Some(worktrees.observation.context),
            work::Request::Repository(super::Request::Worktrees(Request::Prepare {
                worktrees,
                action,
                _admission: admission,
            })),
        )
    }
    pub fn change_worktree(&mut self, plan: Arc<WorktreePlan>) -> Result<OperationId, String> {
        let root = self.check_worktrees(&plan.worktrees)?;
        if !self
            .context(plan.worktrees.observation.context)
            .and_then(|c| c.worktree_plan.as_ref())
            .is_some_and(|p| Arc::ptr_eq(p, &plan))
        {
            return Err("Worktree review expired".into());
        }
        self.start(
            root,
            Some(plan.worktrees.observation.context),
            work::Request::Repository(super::Request::Worktrees(Request::Change(plan))),
        )
    }
}
