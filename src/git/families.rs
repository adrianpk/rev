//! Prepared native Git arguments retain exact input and checked destinations.
use super::*;

mod commands;
mod preparation;

use preparation::Preparation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Clone,
    CloneShallow,
    CloneBare,
    CloneMirror,
    Init,
    SubmoduleAdd,
    SubmoduleList,
    SubmoduleUpdate,
    SubmoduleSync,
    SubmoduleDeinit,
    SubtreeAdd,
    SubtreeMerge,
    SubtreePull,
    SubtreePush,
    SubtreeSplit,
    NotesAdd,
    NotesRemove,
    NotesPrune,
    SparseEnable,
    SparseDisable,
    SparseReapply,
    SparseSet,
    SparseAdd,
    FormatPatch,
    SaveDiff,
    RequestPull,
    ApplyPatch,
    ApplyMailbox,
    RunCommand,
    RunShellCommand,
}
impl Family {
    pub fn repository_action(self) -> bool {
        self.submodule()
            || self.subtree()
            || self.notes()
            || self.sparse()
            || self.output()
            || self.patch()
    }
    pub fn direct(self) -> bool {
        matches!(self, Self::RunCommand | Self::RunShellCommand)
    }
    pub(super) fn patch(self) -> bool {
        matches!(self, Self::ApplyPatch | Self::ApplyMailbox)
    }
    pub fn output(self) -> bool {
        matches!(self, Self::FormatPatch | Self::SaveDiff | Self::RequestPull)
    }
    pub(super) fn notes(self) -> bool {
        matches!(self, Self::NotesAdd | Self::NotesRemove | Self::NotesPrune)
    }
    pub(super) fn sparse(self) -> bool {
        matches!(
            self,
            Self::SparseEnable
                | Self::SparseDisable
                | Self::SparseReapply
                | Self::SparseSet
                | Self::SparseAdd
        )
    }
    pub(super) fn submodule(self) -> bool {
        matches!(
            self,
            Self::SubmoduleAdd
                | Self::SubmoduleList
                | Self::SubmoduleUpdate
                | Self::SubmoduleSync
                | Self::SubmoduleDeinit
        )
    }
    pub(super) fn subtree(self) -> bool {
        matches!(
            self,
            Self::SubtreeAdd
                | Self::SubtreeMerge
                | Self::SubtreePull
                | Self::SubtreePush
                | Self::SubtreeSplit
        )
    }
    pub(super) fn transport(self) -> bool {
        matches!(
            self,
            Self::Clone
                | Self::CloneShallow
                | Self::CloneBare
                | Self::CloneMirror
                | Self::SubmoduleAdd
                | Self::SubmoduleUpdate
                | Self::SubtreeAdd
                | Self::SubtreePull
                | Self::SubtreePush
                | Self::RequestPull
        )
    }
}

pub(super) struct Subtree {
    pub configuration: Vec<u8>,
    pub target: ObjectId,
    pub tree: Option<ObjectId>,
    pub remote: Option<(OsString, OsString, Option<ObjectId>)>,
}
impl std::fmt::Debug for Subtree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subtree")
            .field("target", &self.target)
            .field("tree", &self.tree)
            .finish_non_exhaustive()
    }
}
impl Subtree {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.configuration.capacity()
            + self
                .remote
                .as_ref()
                .map_or(0, |(url, name, _)| url.capacity() + name.capacity())
    }
}

pub(super) struct Submodule {
    pub configuration: Vec<u8>,
    pub name: Option<OsString>,
    pub object: Option<ObjectId>,
    pub url: Vec<u8>,
    pub child: Option<Arc<Sequence>>,
    pub evidence: [Vec<u8>; 3],
}
impl std::fmt::Debug for Submodule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Submodule")
            .field("name", &self.name)
            .field("object", &self.object)
            .finish_non_exhaustive()
    }
}
impl Submodule {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.configuration.capacity()
            + self.name.as_ref().map_or(0, OsString::capacity)
            + self.url.capacity()
            + self.evidence.iter().map(Vec::capacity).sum::<usize>()
    }
    pub fn same(&self, other: &Self) -> bool {
        self.configuration == other.configuration
            && self.name == other.name
            && self.object == other.object
            && self.url == other.url
            && self.evidence == other.evidence
            && match (&self.child, &other.child) {
                (Some(a), Some(b)) => a.same(b),
                (None, None) => true,
                _ => false,
            }
    }
}

#[derive(PartialEq, Eq)]
pub(super) struct NotesRef {
    pub name: Vec<u8>,
    pub object: ObjectId,
    pub symbolic: Vec<u8>,
}
pub(super) struct Notes {
    pub configuration: Vec<u8>,
    pub name: OsString,
    pub target: Option<ObjectId>,
    pub references: Vec<NotesRef>,
    /// Exact target-to-note object pairs; labels never select notes.
    pub entries: Vec<(ObjectId, ObjectId)>,
    pub prune: Vec<ObjectId>,
    pub message: Vec<u8>,
}
impl std::fmt::Debug for Notes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Notes")
            .field("name", &self.name)
            .field("target", &self.target)
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}
impl Notes {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.configuration.capacity()
            + self.name.capacity()
            + self.references.capacity() * std::mem::size_of::<NotesRef>()
            + self
                .references
                .iter()
                .map(|r| r.name.capacity() + r.symbolic.capacity())
                .sum::<usize>()
            + self.entries.capacity() * std::mem::size_of::<(ObjectId, ObjectId)>()
            + self.prune.capacity() * std::mem::size_of::<ObjectId>()
            + self.message.capacity()
    }
}

#[derive(PartialEq, Eq)]
pub(super) struct Sparse {
    pub configuration: Vec<u8>,
    pub enabled: bool,
    pub cone: bool,
    pub patterns: Vec<u8>,
    pub directories: Vec<PathBuf>,
    pub flags: Vec<u8>,
}
impl std::fmt::Debug for Sparse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sparse")
            .field("enabled", &self.enabled)
            .field("cone", &self.cone)
            .field("directories", &self.directories)
            .finish_non_exhaustive()
    }
}
impl Sparse {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.configuration.capacity()
            + self.patterns.capacity()
            + self.flags.capacity()
            + self.directories.capacity() * std::mem::size_of::<PathBuf>()
            + self
                .directories
                .iter()
                .map(PathBuf::capacity)
                .sum::<usize>()
    }
}

pub(super) struct Output {
    pub configuration: Vec<u8>,
    pub bytes: Vec<u8>,
    pub hash: ObjectId,
    pub remote: Option<(OsString, OsString, ObjectId)>,
}
impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("bytes", &self.bytes.len())
            .field("hash", &self.hash)
            .finish_non_exhaustive()
    }
}
impl Output {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.configuration.capacity()
            + self.bytes.capacity()
            + self.remote.as_ref().map_or(0, |(endpoint, reference, _)| {
                endpoint.capacity() + reference.capacity()
            })
    }
}

pub(super) struct Patch {
    pub configuration: Vec<u8>,
    pub path: PathBuf,
    pub stamp: [u64; 8],
    pub bytes: Vec<u8>,
    pub hash: ObjectId,
    pub mails: usize,
}
impl std::fmt::Debug for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Patch")
            .field("path", &self.path)
            .field("bytes", &self.bytes.len())
            .field("hash", &self.hash)
            .finish_non_exhaustive()
    }
}
impl Patch {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.configuration.capacity()
            + self.path.capacity()
            + self.bytes.capacity()
    }
}

pub(super) struct Direct {
    pub configuration: Option<Vec<u8>>,
    pub seconds: usize,
    pub output_bytes: usize,
}
impl std::fmt::Debug for Direct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Direct")
            .field("seconds", &self.seconds)
            .field("output_bytes", &self.output_bytes)
            .finish_non_exhaustive()
    }
}
pub(super) struct Shell {
    pub program: String,
    pub seconds: usize,
    pub output_bytes: usize,
}

#[derive(Debug)]
pub(super) enum FamilyProof {
    Creation,
    Submodule(Submodule),
    Subtree(Subtree),
    Notes(Notes),
    Sparse(Sparse),
    Output(Output),
    Patch(Patch),
    Direct(Direct),
}
impl FamilyProof {
    pub fn bytes(&self) -> usize {
        match self {
            Self::Creation => 0,
            Self::Submodule(state) => state.bytes(),
            Self::Subtree(state) => state.bytes(),
            Self::Notes(state) => state.bytes(),
            Self::Sparse(state) => state.bytes(),
            Self::Output(state) => state.bytes(),
            Self::Patch(state) => state.bytes(),
            Self::Direct(state) => {
                std::mem::size_of::<Direct>()
                    + state.configuration.as_ref().map_or(0, Vec::capacity)
            }
        }
    }
}

/// Canonical preparation is read-only; only execute_family authorizes effects.
#[derive(Debug)]
pub struct FamilyPlan {
    pub family: Family,
    pub directory: PathBuf,
    pub program: String,
    pub args: Vec<OsString>,
    pub destination: PathBuf,
    pub observation: Option<Arc<Observation>>,
    pub(super) source: Option<Arc<Sequence>>,
    pub(super) evidence: [Vec<u8>; 3],
    pub(super) directory_stamp: [u64; 8],
    pub(super) parent_stamp: Option<[u64; 8]>,
    pub(super) proof: FamilyProof,
    pub(super) _admission: Lease,
}
impl FamilyPlan {
    pub(super) fn submodule(&self) -> Option<&Submodule> {
        if let FamilyProof::Submodule(state) = &self.proof {
            Some(state)
        } else {
            None
        }
    }
    pub(super) fn subtree(&self) -> Option<&Subtree> {
        if let FamilyProof::Subtree(state) = &self.proof {
            Some(state)
        } else {
            None
        }
    }
    pub fn target_object(&self) -> Option<ObjectId> {
        match &self.proof {
            FamilyProof::Notes(state) => state.target,
            _ => self.subtree().map(|s| s.target),
        }
    }
    pub fn prune_objects(&self) -> &[ObjectId] {
        match &self.proof {
            FamilyProof::Notes(state) => &state.prune,
            _ => &[],
        }
    }
    pub fn direct_limits(&self) -> Option<(usize, usize)> {
        match &self.proof {
            FamilyProof::Direct(s) => Some((s.seconds, s.output_bytes)),
            _ => None,
        }
    }
    pub fn input_preview(&self) -> Option<(&Path, usize, ObjectId)> {
        match &self.proof {
            FamilyProof::Patch(input) => Some((&input.path, input.bytes.len(), input.hash)),
            _ => None,
        }
    }
    pub fn output_preview(&self) -> Option<(usize, ObjectId)> {
        match &self.proof {
            FamilyProof::Output(output) => Some((output.bytes.len(), output.hash)),
            _ => None,
        }
    }
    pub fn remote_branch(&self) -> Option<&std::ffi::OsStr> {
        self.subtree()
            .and_then(|s| s.remote.as_ref())
            .map(|(_, reference, _)| reference.as_os_str())
    }
}
#[derive(Debug)]
pub struct FamilyResult {
    pub plan: Arc<FamilyPlan>,
    pub created: Option<Identity>,
    pub object: Option<ObjectId>,
    pub(super) source: Option<Observation>,
    pub(super) _admission: Lease,
}
pub(super) enum Request {
    Prepare {
        family: Family,
        args: Vec<OsString>,
        observation: Option<Arc<Observation>>,
        output: Option<PathBuf>,
        shell: Option<Shell>,
        _admission: Lease,
    },
    Execute(Arc<FamilyPlan>),
}
impl Repositories {
    pub fn family_plan(&self) -> Option<&Arc<FamilyPlan>> {
        self.family_plan.as_ref()
    }
    pub fn family_result(&self) -> Option<&Arc<FamilyResult>> {
        self.family_result.as_ref()
    }
    fn family_idle(&self) -> Result<(), String> {
        if self.progress().next().is_some() {
            return Err("Prepared command waits for owned Git work and common storage".into());
        }
        Ok(())
    }
    pub fn prepare_family(
        &mut self,
        directory: &Path,
        context: Option<ContextId>,
        family: Family,
        args: Vec<OsString>,
    ) -> Result<OperationId, String> {
        self.prepare_family_file(
            directory,
            context,
            Preparation {
                family,
                args,
                output: None,
            },
        )
    }
    pub fn prepare_family_output(
        &mut self,
        directory: &Path,
        context: Option<ContextId>,
        family: Family,
        args: Vec<OsString>,
        output: &Path,
    ) -> Result<OperationId, String> {
        self.prepare_family_file(
            directory,
            context,
            Preparation {
                family,
                args,
                output: Some(output.into()),
            },
        )
    }
    pub fn execute_family(&mut self, plan: Arc<FamilyPlan>) -> Result<OperationId, String> {
        self.family_idle()?;
        if !self
            .family_plan
            .as_ref()
            .is_some_and(|p| Arc::ptr_eq(p, &plan))
        {
            return Err("Prepared command review expired".into());
        }
        if let Some(o) = &plan.observation
            && !self
                .context(o.context)
                .is_some_and(|c| Arc::ptr_eq(&c.observation, o) && c.failure.is_none())
        {
            return Err("Prepared command source observation changed".into());
        }
        self.start(
            plan.directory.clone(),
            plan.observation.as_ref().map(|o| o.context),
            work::Request::Family(Request::Execute(plan)),
        )
    }
}
