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
    pub targets: Vec<IndexTarget>,
    pub _admission: Lease,
}

fn path_names(diff: &Diff) -> impl Iterator<Item = &Path> {
    std::iter::once(diff.change.path.as_path()).chain(
        diff.change
            .old_path
            .as_deref()
            .filter(|_| diff.change.renamed_in(diff.group)),
    )
}
fn overlaps(a: LineRange, b: LineRange) -> bool {
    // Empty ranges at the same insertion boundary are not independent edits.
    if a.count == 0 || b.count == 0 {
        a.start <= b.start.saturating_add(b.count) && b.start <= a.start.saturating_add(a.count)
    } else {
        a.start < b.start.saturating_add(b.count) && b.start < a.start.saturating_add(a.count)
    }
}

impl Repositories {
    pub(crate) fn index_marks_admission(&self) -> Result<Lease, String> {
        self.budget
            .reserve(256 * std::mem::size_of::<IndexTarget>())
    }

    /// Launch one checked disk/index action. No visiting buffer is saved.
    /// Complete comparison evidence is revalidated on the worker before any
    /// index-changing child; possible effects invalidate prior authority.
    pub fn change_index(
        &mut self,
        direction: IndexDirection,
        target: IndexTarget,
    ) -> Result<OperationId, String> {
        self.change_index_group(direction, &[target])
    }

    /// Admit the complete group before any worker or changing child. Paths may
    /// share a direction; hunks must share one immutable parent comparison.
    pub fn change_index_group(
        &mut self,
        direction: IndexDirection,
        targets: &[IndexTarget],
    ) -> Result<OperationId, String> {
        if targets.is_empty() || targets.len() > 256 {
            return Err("Index selection requires 1..=256 exact targets".into());
        }
        let diff = targets[0].diff();
        for (i, target) in targets.iter().enumerate() {
            let current = target.diff();
            if !self.detail_current(current)
                || current.observation.context != diff.observation.context
                || current.observation.generation != diff.observation.generation
                || current
                    .observation
                    .status
                    .as_ref()
                    .is_none_or(|s| s.completeness != Completeness::Complete)
            {
                return Err(
                    "Entire index group requires complete current comparisons and Status".into(),
                );
            }
            let allowed = match direction {
                IndexDirection::Stage => {
                    matches!(current.group, Group::Untracked | Group::Unstaged)
                }
                IndexDirection::Unstage => current.group == Group::Staged,
            };
            if !allowed || matches!(current.kind, DiffKind::Unmerged | DiffKind::Malformed(_)) {
                return Err("Entire selected group must support the same index direction".into());
            }
            if let IndexTarget::Hunk { hunk, .. } = target
                && (!current.supports_hunks() || current.hunk(*hunk).is_none())
            {
                return Err("Selected hunk is unsupported or belongs to another comparison".into());
            }
            for previous in &targets[..i] {
                match (previous, target) {
                    (IndexTarget::Path(a), IndexTarget::Path(b)) => {
                        if path_names(a)
                            .any(|p| path_names(b).any(|q| p.starts_with(q) || q.starts_with(p)))
                        {
                            return Err(
                                "Index path group contains duplicate or overlapping paths".into()
                            );
                        }
                    }
                    (
                        IndexTarget::Hunk { diff: a, hunk: x },
                        IndexTarget::Hunk { diff: b, hunk: y },
                    ) => {
                        if a.id != b.id || x == y {
                            return Err(
                                "Hunk group requires unique hunks from one path/layer comparison"
                                    .into(),
                            );
                        }
                        let x = a.hunk(*x).ok_or("Hunk identity expired")?;
                        let y = b.hunk(*y).ok_or("Hunk identity expired")?;
                        if overlaps(x.old, y.old) || overlaps(x.new, y.new) {
                            return Err("Hunk group contains overlapping ranges".into());
                        }
                    }
                    _ => {
                        return Err("Whole paths and text hunks cannot share an index group".into());
                    }
                }
            }
        }
        let root = self.mutation_root(diff)?;
        let context = diff.observation.context;
        let admission = self.budget.reserve(std::mem::size_of_val(targets))?;
        self.start(
            root,
            Some(context),
            work::Request::Index(Request {
                direction,
                targets: targets.to_vec(),
                _admission: admission,
            }),
        )
    }
    pub(super) fn mutation_root(&self, diff: &Diff) -> Result<PathBuf, String> {
        self.mutation_identity_root(&diff.observation.identity)
    }
    pub(super) fn mutation_identity_root(&self, identity: &Identity) -> Result<PathBuf, String> {
        if self.progress().any(|p| {
            p.context.is_none()
                || p.context.and_then(|id| self.context(id)).is_some_and(|c| {
                    c.observation.identity.common_directory == identity.common_directory
                })
        }) {
            return Err("Repository common storage already owns an active Git operation".into());
        }
        identity
            .worktree
            .clone()
            .ok_or_else(|| "Repository has no working tree".into())
    }
}
