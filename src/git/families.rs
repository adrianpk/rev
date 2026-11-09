//! Prepared native Git arguments retain exact input and checked destinations.
use super::*;

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
    pub fn label(self) -> &'static str {
        match self {
            Self::Clone => "regular clone",
            Self::CloneShallow => "shallow clone",
            Self::CloneBare => "bare clone",
            Self::CloneMirror => "mirror clone",
            Self::Init => "repository init",
            Self::SubmoduleAdd => "submodule add",
            Self::SubmoduleList => "submodule list",
            Self::SubmoduleUpdate => "submodule update",
            Self::SubmoduleSync => "submodule sync",
            Self::SubmoduleDeinit => "submodule deinit",
            Self::SubtreeAdd => "subtree add",
            Self::SubtreeMerge => "subtree merge",
            Self::SubtreePull => "subtree pull",
            Self::SubtreePush => "subtree push",
            Self::SubtreeSplit => "subtree split",
            Self::NotesAdd => "notes add",
            Self::NotesRemove => "notes remove",
            Self::NotesPrune => "notes prune",
            Self::SparseEnable => "sparse checkout enable",
            Self::SparseDisable => "sparse checkout disable",
            Self::SparseReapply => "sparse checkout reapply",
            Self::SparseSet => "sparse checkout set",
            Self::SparseAdd => "sparse checkout add",
            Self::FormatPatch => "format patch output",
            Self::SaveDiff => "saved diff output",
            Self::RequestPull => "request pull output",
            Self::ApplyPatch => "plain patch application",
            Self::ApplyMailbox => "mailbox application",
            Self::RunCommand => "direct Git command",
            Self::RunShellCommand => "explicit shell command",
        }
    }
    pub fn seed(self) -> Vec<String> {
        let args: &[&str] = match self {
            Self::Clone => &["clone", "--", "", ""],
            Self::CloneShallow => &["clone", "--depth=1", "--", "", ""],
            Self::CloneBare => &["clone", "--bare", "--", "", ""],
            Self::CloneMirror => &["clone", "--mirror", "--", "", ""],
            Self::Init => &["init", "--", ""],
            Self::SubmoduleAdd => &["submodule", "add", "--name", "", "--", "", ""],
            Self::SubmoduleList => &["submodule", "status", "--recursive"],
            Self::SubmoduleUpdate => &["submodule", "update", "--init", "--checkout", "--", ""],
            Self::SubmoduleSync => &["submodule", "sync", "--", ""],
            Self::SubmoduleDeinit => &["submodule", "deinit", "--", ""],
            Self::SubtreeAdd => &["subtree", "add", "--prefix", "", "--", "", ""],
            Self::SubtreeMerge => &["subtree", "merge", "--prefix", "", "--", ""],
            Self::SubtreePull => &["subtree", "pull", "--prefix", "", "--", "", ""],
            Self::SubtreePush => &["subtree", "push", "--prefix", "", "--", "", ""],
            Self::SubtreeSplit => &["subtree", "split", "--prefix", "", "--", ""],
            Self::NotesAdd => &["notes", "--ref", "", "add", "--message", "", "--", ""],
            Self::NotesRemove => &["notes", "--ref", "", "remove", "--", ""],
            Self::NotesPrune => &["notes", "--ref", "", "prune"],
            Self::SparseEnable => &["sparse-checkout", "init", "--cone", "--no-sparse-index"],
            Self::SparseDisable => &["sparse-checkout", "disable"],
            Self::SparseReapply => &["sparse-checkout", "reapply"],
            Self::SparseSet => &[
                "sparse-checkout",
                "set",
                "--cone",
                "--no-sparse-index",
                "--skip-checks",
                "--",
                "",
            ],
            Self::SparseAdd => &["sparse-checkout", "add", "--skip-checks", "--", ""],
            Self::FormatPatch => &[
                "format-patch",
                "--stdout",
                "--binary",
                "--full-index",
                "--no-signature",
                "--no-cover-letter",
                "--no-notes",
                "--no-ext-diff",
                "--no-textconv",
                "",
                "--",
            ],
            Self::SaveDiff => &[
                "diff",
                "--binary",
                "--full-index",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                "",
                "",
                "--",
            ],
            Self::RunCommand | Self::RunShellCommand => &[""],
            Self::RequestPull => &["request-pull", "", "", ""],
            Self::ApplyPatch => &[
                "apply",
                "--index",
                "--binary",
                "--whitespace=nowarn",
                "--",
                "",
            ],
            Self::ApplyMailbox => &[
                "am",
                "--patch-format=mboxrd",
                "--no-3way",
                "--no-signoff",
                "--no-message-id",
                "--no-scissors",
                "--no-keep-cr",
                "--no-keep",
                "--no-keep-non-patch",
                "--utf8",
                "--no-rerere-autoupdate",
                "--whitespace=nowarn",
                "--",
                "",
            ],
        };
        args.iter().map(|a| (*a).into()).collect()
    }
    pub(super) fn validate(self, args: &[OsString]) -> Result<(), String> {
        if self.direct() {
            if args.is_empty()
                || args[0].is_empty()
                || args.iter().any(|a| a.as_encoded_bytes().contains(&0))
                || (self == Self::RunShellCommand && args.len() != 1)
            {
                return Err("Direct Git requires literal arguments; shell mode requires exactly one nonempty command string without NUL".into());
            }
            return Ok(());
        }
        let expected = self.seed();
        let variadic = matches!(self, Self::SparseSet | Self::SparseAdd);
        if (if variadic {
            args.len() < expected.len()
        } else {
            args.len() != expected.len()
        }) || args
            .iter()
            .zip(&expected)
            .any(|(a, b)| !b.is_empty() && a != b.as_str())
        {
            return Err(
                "Keep the prepared family and its explicit option/parameter structure".into(),
            );
        }
        if args
            .iter()
            .any(|a| a.is_empty() || a.as_encoded_bytes().contains(&0))
        {
            return Err("Fill every prepared argument with nonempty, non-NUL input".into());
        }
        Ok(())
    }
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
        self.prepare_family_file(directory, context, family, args, None)
    }
    pub fn prepare_family_output(
        &mut self,
        directory: &Path,
        context: Option<ContextId>,
        family: Family,
        args: Vec<OsString>,
        output: &Path,
    ) -> Result<OperationId, String> {
        self.prepare_family_file(directory, context, family, args, Some(output.into()))
    }
    fn prepare_family_file(
        &mut self,
        directory: &Path,
        context: Option<ContextId>,
        family: Family,
        args: Vec<OsString>,
        output: Option<PathBuf>,
    ) -> Result<OperationId, String> {
        self.family_idle()?;
        if family.output() != output.is_some()
            || output.as_ref().is_some_and(|p| {
                p.as_os_str().is_empty()
                    || p.as_os_str().len() > 4096
                    || p.as_os_str().as_encoded_bytes().contains(&0)
            })
        {
            return Err("Output families require one explicit output path within 4096 bytes; other families take literal argv only".into());
        }
        if !directory.is_absolute() || directory.as_os_str().len() > 4096 {
            return Err("Prepared directory requires an absolute path within 4096 bytes".into());
        }
        if args.len() > 64
            || args.iter().any(|a| a.len() > 4096)
            || args.iter().map(|a| a.len()).sum::<usize>() > 65536
        {
            return Err(
                "Prepared input exceeds 64 arguments, 4096 bytes per argument or 64 KiB total"
                    .into(),
            );
        }
        family.validate(&args)?;
        let (args, shell) = if family == Family::RunShellCommand {
            let command = args[0]
                .to_str()
                .ok_or("Shell command requires UTF-8 input")?;
            let mut argv =
                crate::shell::arguments(&self.shell_policy, &self.shell_startup, command)?;
            let program = argv.remove(0);
            (
                argv.into_iter().map(OsString::from).collect(),
                Some(Shell {
                    program,
                    seconds: self
                        .shell_policy
                        .timeout_seconds
                        .min(self.policy.operation_timeout_seconds),
                    output_bytes: self
                        .shell_policy
                        .max_output_bytes
                        .min(self.policy.max_output_bytes),
                }),
            )
        } else {
            (args, None)
        };
        let observation = context
            .map(|id| {
                self.context(id)
                    .map(|c| std::sync::Arc::clone(&c.observation))
                    .ok_or("Prepared source context expired")
            })
            .transpose()?;
        if family.repository_action()
            && observation.as_ref().is_none_or(|o| {
                !(family.notes() || family.output()) && o.identity.worktree.is_none()
            })
        {
            return Err(
                "Repository family preparation requires a checked working-tree context".into(),
            );
        }
        let size = args.iter().map(OsString::capacity).sum::<usize>()
            + output.as_ref().map_or(0, PathBuf::capacity)
            + shell.as_ref().map_or(0, |s| s.program.capacity())
            + 8192;
        let admission = self.budget.reserve(size)?;
        self.family_plan = None;
        self.family_result = None;
        self.start(
            directory.into(),
            context,
            work::Request::Family(Request::Prepare {
                family,
                args,
                observation,
                output,
                shell,
                _admission: admission,
            }),
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
