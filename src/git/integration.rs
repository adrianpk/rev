//! Checked integration and immutable observations of Git-owned sequence state.
use super::*;
mod autosquash;
pub use autosquash::{AutosquashAction, AutosquashPlan, AutosquashStep};

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
pub enum SequenceControl {
    Continue,
    Skip,
    Abort,
}
impl SequenceControl {
    pub fn label(self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Skip => "skip",
            Self::Abort => "abort",
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
    pub application: Option<Arc<ApplyProgress>>,
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
    pub(in crate::git) fn same(&self, other: &Self) -> bool {
        self.family == other.family
            && self.current == other.current
            && self.targets == other.targets
            && self.original == other.original
            && self.onto == other.onto
            && self.branch == other.branch
            && self.step == other.step
            && self.total == other.total
            && self.conflicts == other.conflicts
            && self.files == other.files
            && self.index == other.index
            && self.worktree == other.worktree
            && match (&self.application, &other.application) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && self.refs.same(&other.refs)
            && self.refs.observation.status == other.refs.observation.status
    }
    /// Eligibility comes from exact Git-owned evidence, never a view label.
    pub fn control_admission(
        &self,
        action: SequenceControl,
        family: Option<SequenceFamily>,
    ) -> Result<(), String> {
        let actual = self.family.ok_or("No Git-owned sequence is active")?;
        if family.is_some_and(|f| f != actual) {
            return Err("Requested control belongs to a different Git sequence family".into());
        }
        if !matches!(actual, SequenceFamily::Merge | SequenceFamily::Rebase) {
            return Err(
                "This observed Git sequence family has no compatible native control here".into(),
            );
        }
        if self.refs.completeness != Completeness::Complete
            || self
                .refs
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete)
        {
            return Err("Sequence control requires complete Status and Refs".into());
        }
        if self
            .files
            .iter()
            .any(|f| f.name.ends_with(b"autostash") || f.name == b"MERGE_AUTOSTASH")
        {
            return Err("Sequence includes autostash; native control cannot authorize its implicit application".into());
        }
        if actual == SequenceFamily::Rebase
            && (self.total.is_some_and(|n| n > 256)
                || self
                    .files
                    .iter()
                    .any(|f| f.name.ends_with(b"update-refs") && !f.bytes.is_empty()))
        {
            return Err(
                "Rebase expands beyond the admitted finite sequence or updates additional refs"
                    .into(),
            );
        }
        let original = self
            .original
            .ok_or("Git sequence lacks the exact original object; lineage is unavailable")?;
        if actual == SequenceFamily::Merge {
            if action == SequenceControl::Skip {
                return Err("Skip is unavailable for a merge sequence".into());
            }
            if self.targets.is_empty() || self.refs.head != Some(original) {
                return Err("Merge original HEAD/targets are missing or incompatible".into());
            }
        } else {
            if self.onto.is_none() || self.branch.is_empty() {
                return Err("Rebase original branch/onto lineage is unavailable".into());
            }
            if self.branch != b"detached HEAD"
                && (!self.branch.starts_with(b"refs/heads/")
                    || self
                        .refs
                        .reference(&self.branch)
                        .is_none_or(|r| r.object != original))
            {
                return Err(
                    "Rebase original branch no longer matches its Git-owned original object".into(),
                );
            }
            self.checked_todo()?;
        }
        if action == SequenceControl::Continue && !self.conflicts.is_empty() {
            return Err(
                "Continue requires saved and staged resolutions with no unmerged entries".into(),
            );
        }
        Ok(())
    }
    fn checked_todo(&self) -> Result<(), String> {
        let mut count = 0;
        for file in self
            .files
            .iter()
            .filter(|f| f.name.ends_with(b"git-rebase-todo") || f.name.ends_with(b"done"))
        {
            for line in file.bytes.split(|b| *b == b'\n') {
                if line.len() > 4096 {
                    return Err("Sequence todo line exceeds 4096-byte admission".into());
                }
                let mut fields = line
                    .split(|b| b.is_ascii_whitespace())
                    .filter(|f| !f.is_empty());
                let Some(action) = fields.next() else {
                    continue;
                };
                if action.starts_with(b"#") {
                    continue;
                }
                if action == b"noop" {
                    continue;
                }
                count += 1;
                if count > 256 {
                    return Err("Sequence todo exceeds 256 admitted actions".into());
                }
                if ![
                    b"pick".as_slice(),
                    b"p",
                    b"fixup",
                    b"f",
                    b"squash",
                    b"s",
                    b"drop",
                    b"d",
                ]
                .contains(&action)
                {
                    return Err("Sequence todo contains an unsupported or executable action; native control is unavailable".into());
                }
                let mut object = fields.next().ok_or("Sequence todo object is missing")?;
                if matches!(action, b"fixup" | b"f") && object == b"-C" {
                    object = fields.next().ok_or("Fixup todo object is missing")?;
                }
                if object.len() < 4
                    || object.len() > 64
                    || !object.iter().all(u8::is_ascii_hexdigit)
                {
                    return Err("Sequence todo object is not an admitted Git object prefix".into());
                }
            }
        }
        Ok(())
    }

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
    Observe {
        observation: Arc<Observation>,
        application: Option<Arc<ApplyProgress>>,
    },
    PrepareAutosquash {
        refs: Arc<Refs>,
        target: Vec<u8>,
        _admission: Lease,
    },
    Autosquash(Arc<AutosquashPlan>),
    Control {
        sequence: Arc<Sequence>,
        action: SequenceControl,
        family: Option<SequenceFamily>,
    },
    Start {
        refs: Arc<Refs>,
        target: Vec<u8>,
        mode: IntegrationMode,
        _admission: Lease,
    },
}
impl Repositories {
    pub fn check_sequence(&self, sequence: &Arc<Sequence>) -> Result<(), String> {
        self.check_refs(&sequence.refs)?;
        if !self
            .context(sequence.refs.observation.context)
            .and_then(|c| c.sequence.as_ref())
            .is_some_and(|current| Arc::ptr_eq(current, sequence))
        {
            return Err("Sequence observation expired; refresh actual Git state".into());
        }
        Ok(())
    }
    /// Caller confirms one fresh family/action and protects unsaved sources.
    /// Actual Git metadata, index and working-tree evidence are checked again.
    pub fn control_sequence(
        &mut self,
        sequence: Arc<Sequence>,
        action: SequenceControl,
        family: Option<SequenceFamily>,
    ) -> Result<OperationId, String> {
        self.check_sequence(&sequence)?;
        sequence.control_admission(action, family)?;
        let root = self.mutation_identity_root(&sequence.refs.observation.identity)?;
        self.start(
            root,
            Some(sequence.refs.observation.context),
            work::Request::Integration(Request::Control {
                sequence,
                action,
                family,
            }),
        )
    }

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
            work::Request::Integration(Request::Observe {
                observation,
                application: self
                    .context(context)
                    .and_then(|c| c.sequence.as_ref())
                    .and_then(|s| s.application.clone()),
            }),
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
