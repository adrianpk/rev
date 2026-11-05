//! Checked integration and immutable observations of Git-owned sequence state.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegrationMode {
    FastForward,
    Merge,
    NoFastForward,
    Rebase,
}
impl IntegrationMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::FastForward => "fast-forward-only merge",
            Self::Merge => "merge with fast-forward allowed",
            Self::NoFastForward => "merge commit required",
            Self::Rebase => "ordinary rebase",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceFamily {
    Merge,
    Rebase,
    CherryPick,
    Revert,
    Mailbox,
    Unsupported,
}

#[derive(PartialEq, Eq)]
pub(super) struct SequenceFile {
    pub name: Vec<u8>,
    pub bytes: Vec<u8>,
}

/// Git alone owns sequence files. Private bounded bytes bind later requests;
/// displayed labels and conflict row positions cannot construct authority.
pub struct Sequence {
    pub id: OperationId,
    pub refs: Arc<Refs>,
    pub family: Option<SequenceFamily>,
    pub current: Option<ObjectId>,
    pub targets: Vec<ObjectId>,
    pub original: Option<ObjectId>,
    pub onto: Option<ObjectId>,
    pub branch: Vec<u8>,
    pub step: Option<usize>,
    pub total: Option<usize>,
    pub conflicts: Vec<PathBuf>,
    pub(in crate::git) files: Vec<SequenceFile>,
    pub(in crate::git) index: Vec<u8>,
    pub(in crate::git) worktree: Vec<u8>,
    pub(in crate::git) _admission: Lease,
}
impl std::fmt::Debug for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sequence")
            .field("id", &self.id)
            .field("family", &self.family)
            .field("current", &self.current)
            .field("original", &self.original)
            .field("onto", &self.onto)
            .field("step", &self.step)
            .field("total", &self.total)
            .field("conflicts", &self.conflicts)
            .finish_non_exhaustive()
    }
}
impl Sequence {
    pub(in crate::git) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.targets.capacity() * std::mem::size_of::<ObjectId>()
            + self.branch.capacity()
            + self.index.capacity()
            + self.worktree.capacity()
            + self.files.capacity() * std::mem::size_of::<SequenceFile>()
            + self
                .files
                .iter()
                .map(|f| f.name.capacity() + f.bytes.capacity())
                .sum::<usize>()
            + self.conflicts.capacity() * std::mem::size_of::<PathBuf>()
            + self.conflicts.iter().map(PathBuf::capacity).sum::<usize>()
    }
}
pub(super) enum Request {
    Observe(Arc<Observation>),
    Start {
        refs: Arc<Refs>,
        target: Vec<u8>,
        mode: IntegrationMode,
        _admission: Lease,
    },
}
impl Repositories {
    /// Refresh compatible Status, Refs and actual Git-owned sequence metadata.
    pub fn sequence(&mut self, context: ContextId) -> Result<OperationId, String> {
        let observation = self
            .context(context)
            .ok_or("Git context expired")?
            .observation
            .clone();
        let root = self.mutation_identity_root(&observation.identity)?;
        self.start(
            root,
            Some(context),
            work::Request::Integration(Request::Observe(observation)),
        )
    }
    /// Caller owns explicit policy/target confirmation and unsaved-buffer
    /// protection. The worker checks exact refs, clean disk and sequence state.
    pub fn integrate(
        &mut self,
        refs: Arc<Refs>,
        target: &[u8],
        mode: IntegrationMode,
    ) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.completeness != Completeness::Complete
            || refs.head.is_none()
            || refs.head_ref.is_empty()
        {
            return Err("Integration requires complete Refs and a checked current branch".into());
        }
        if refs.reference(target).and_then(Reference::commit).is_none() {
            return Err("Integration requires one exact selected ref with a commit target".into());
        }
        let status = refs
            .observation
            .status
            .as_ref()
            .ok_or("Integration requires a working tree")?;
        if status.completeness != Completeness::Complete || !status.paths.is_empty() {
            return Err("Integration requires a complete clean index and working tree".into());
        }
        let directory = self.mutation_identity_root(&refs.observation.identity)?;
        let admission = self.budget.reserve(target.len() + 4096)?;
        self.start(
            directory,
            Some(refs.observation.context),
            work::Request::Integration(Request::Start {
                refs,
                target: target.into(),
                mode,
                _admission: admission,
            }),
        )
    }
}
