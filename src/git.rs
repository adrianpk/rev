//! Native repository observations and owned installed-Git operations. Git keeps
//! durable repository state; this owner retains finite session observations.
pub(crate) mod budget;
mod commit;
mod diff;
pub use commit::MAX_COMMIT_BYTES;
pub use commit::{CommitMode, Composition};
mod conflicts;
pub use conflicts::{Conflict, ConflictExplanation, ConflictRole, ConflictSide};
pub use conflicts::{Resolution, ResolutionAction, SavedResult};
mod discard;
mod history;
mod index;
mod integration;
pub use history::{
    Attribution, Blame, History, HistoryPage, Reflog, ReflogEntry, Revision, RevisionDetail,
    RevisionFile, RevisionPath,
};
pub use index::{IndexDirection, IndexTarget};
pub use integration::{
    AutosquashAction, AutosquashPlan, AutosquashStep, IntegrationMode, Sequence, SequenceControl,
    SequenceFamily,
};
mod bisect;
pub use bisect::{BisectAction, BisectPlan, BisectResult, BisectState};
mod sequences;
pub use sequences::{ApplyKind, ApplyPlan, ApplyProgress, ApplyStep, RevisionChoice};
mod repository;
pub use repository::{
    StashAction, StashDetail, StashEntry, StashPlan, Stashes, WorktreeAction, WorktreeEntry,
    WorktreePlan, Worktrees,
};
mod refs;
pub use refs::{BranchAction, PushIntent, RefKind, Reference, Refs, TagIntent, TagKind};
mod status;
pub use diff::{Baseline, Diff, DiffKind, DiffLine, Hunk, HunkId, LineRange};
mod work;
pub use status::{Completeness, Group, Head, PathChange, Status};

use crate::{
    config,
    process::{Cancellation, capture},
};
use budget::{Budget, Lease};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

pub const MAX_CONTEXTS: usize = 4;
pub const SESSION_BYTES: usize = budget::SESSION_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectFormat {
    Sha1,
    Sha256,
}

/// Full binary object identity; display abbreviations never become arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectId {
    bytes: [u8; 32],
    format: ObjectFormat,
}

impl ObjectId {
    pub fn parse(hex: &[u8], format: ObjectFormat) -> Result<Self, String> {
        let count = match format {
            ObjectFormat::Sha1 => 20,
            ObjectFormat::Sha256 => 32,
        };
        if hex.len() != count * 2 {
            return Err("Git object identity has an unsupported length".into());
        }
        let mut bytes = [0; 32];
        for (target, pair) in bytes[..count].iter_mut().zip(hex.as_chunks::<2>().0) {
            let nibble = |byte: u8| match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'f' => Ok(byte - b'a' + 10),
                _ => Err("Git object identity is not lowercase hexadecimal"),
            };
            *target = (nibble(pair[0])? << 4) | nibble(pair[1])?;
        }
        Ok(Self { bytes, format })
    }
    pub fn format(&self) -> ObjectFormat {
        self.format
    }
    pub fn hex(&self) -> String {
        let count = match self.format {
            ObjectFormat::Sha1 => 20,
            ObjectFormat::Sha256 => 32,
        };
        use std::fmt::Write;
        let mut hex = String::with_capacity(count * 2);
        for byte in &self.bytes[..count] {
            write!(&mut hex, "{byte:02x}").expect("write to String");
        }
        hex
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub worktree: Option<PathBuf>,
    pub git_directory: PathBuf,
    pub common_directory: PathBuf,
    pub format: ObjectFormat,
}

impl Identity {
    fn bytes(&self) -> usize {
        self.worktree.as_ref().map_or(0, PathBuf::capacity)
            + self.git_directory.capacity()
            + self.common_directory.capacity()
            + std::mem::size_of::<Self>()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContextId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OperationId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    Worktrees,
    WorktreePrepare,
    WorktreeVisit,
    Worktree,
    Stashes,
    StashShow,
    StashPrepare,
    Stash,
    Status,
    Diff,
    Stage,
    Unstage,
    Discard,
    Commit,
    CommitPrepare,
    History,
    Refs,
    Branch,
    TagPrepare,
    TagCreate,
    TagDelete,
    Fetch,
    PushPrepare,
    Integration,
    Sequence,
    Conflict,
    ConflictPrepare,
    ConflictResolve,
    ConflictExplain,
    SequenceControl,
    AutosquashPrepare,
    ApplicationPrepare,
    Application,
    ApplicationControl,
    BisectPrepare,
    Bisect,
    Push,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Checking,
    Executing,
    Refreshing,
    Verifying,
    Cleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Verified,
    Failed,
    Cancelled,
    TimedOut,
    OutputLimit,
    CleanupPending,
    Indeterminate,
    InProgress,
    Skipped,
    Aborted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Effects {
    #[default]
    NotStarted,
    Possible,
    Verified,
}

/// Child facts survive failure and cancellation. Raw bytes are inspection data,
/// never authority; presentation must escape controls and redact credentials.
#[derive(Debug)]
pub struct Record {
    pub id: OperationId,
    pub kind: OperationKind,
    pub context: Option<ContextId>,
    pub started: Instant,
    pub ended: Instant,
    pub outcome: Outcome,
    pub effects: Effects,
    pub argv: Vec<OsString>,
    pub exit: Option<std::process::ExitStatus>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub output_truncated: bool,
    pub diagnostic: Option<String>,
    admission: Lease,
}

impl Record {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.argv.capacity() * std::mem::size_of::<OsString>()
            + self.argv.iter().map(OsString::capacity).sum::<usize>()
            + self.stdout.capacity()
            + self.stderr.capacity()
            + self.diagnostic.as_ref().map_or(0, String::capacity)
    }
}

#[derive(Debug)]
pub struct Observation {
    pub context: ContextId,
    pub generation: u64,
    pub identity: Identity,
    /// Bare repositories have a validated identity but no working-tree Status.
    pub status: Option<Status>,
    admission: Lease,
}

impl Observation {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.identity.bytes()
            + self.status.as_ref().map_or(0, Status::bytes)
    }
}

pub struct Context {
    pub id: ContextId,
    pub observation: Arc<Observation>,
    pub failure: Option<OperationId>,
    pub detail: Option<Arc<Diff>>,
    pub detail_failure: Option<OperationId>,
    pub worktrees: Option<Arc<Worktrees>>,
    pub worktree_plan: Option<Arc<WorktreePlan>>,
    pub stashes: Option<Arc<Stashes>>,
    pub stash_detail: Option<Arc<StashDetail>>,
    pub stash_plan: Option<Arc<StashPlan>>,
    pub refs: Option<Arc<Refs>>,
    pub sequence: Option<Arc<Sequence>>,
    pub conflict: Option<Arc<Conflict>>,
    pub resolution: Option<Arc<Resolution>>,
    pub conflict_explanation: Option<Arc<ConflictExplanation>>,
    pub autosquash: Option<Arc<AutosquashPlan>>,
    pub application_plan: Option<Arc<ApplyPlan>>,
    pub bisect_plan: Option<Arc<BisectPlan>>,
    pub history: Option<Arc<History>>,
    pub reflog: Option<Arc<Reflog>>,
    pub blame: Option<Arc<Blame>>,
    pub revision_file: Option<Arc<RevisionFile>>,
    pub revision: Option<Arc<RevisionDetail>>,
    tag_intent: Option<Arc<TagIntent>>,
    push_intent: Option<Arc<PushIntent>>,
    tag_draft: Option<refs::TagDraft>,
    draft: Option<commit::Draft>,
    composition: Option<Arc<Composition>>,
}

#[derive(Clone, Copy, Debug)]
pub struct Progress {
    pub operation: OperationId,
    pub kind: OperationKind,
    pub context: Option<ContextId>,
    pub phase: Phase,
    pub child: Option<u32>,
    pub started: Instant,
    pub stopping: bool,
}

struct Active {
    progress: Arc<Mutex<Progress>>,
    cancel: Cancellation,
    thread: Option<JoinHandle<work::Finished>>,
    retained: Option<work::Retained>,
    record: Option<Record>,
}

/// Result publication is generation-bound. A failed replacement leaves the
/// last confirmed observation intact and records a visible failure.
pub struct Update {
    pub operation: OperationId,
    pub kind: OperationKind,
    pub context: Option<ContextId>,
    pub published: bool,
}

/// One owner per editor. Discovery is serialized until repository identity is
/// known. Refreshes of distinct admitted contexts can run concurrently; common
/// storage is serialized for future changing requests by this same owner.
pub struct Repositories {
    policy: config::Git,
    contexts: Vec<Context>,
    active: Vec<Active>,
    records: Vec<Record>,
    next: u64,
    budget: Budget,
    _admission: Lease,
    wake: crate::term::Waker,
}

impl Repositories {
    pub fn new(snapshot: &config::Snapshot, wake: crate::term::Waker) -> Self {
        let budget = Budget::default();
        let admission = budget
            .reserve(65536)
            .expect("fixed Git owner metadata fits an empty budget");
        Self {
            policy: snapshot.settings().git.clone(),
            contexts: Vec::new(),
            active: Vec::new(),
            records: Vec::new(),
            next: 0,
            budget,
            _admission: admission,
            wake,
        }
    }
    pub fn contexts(&self) -> &[Context] {
        &self.contexts
    }
    pub fn context(&self, id: ContextId) -> Option<&Context> {
        self.contexts.iter().find(|c| c.id == id)
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    /// Inspection includes a retained failed-cleanup record without releasing
    /// its live child owner or moving the operation into completed history.
    pub fn record(&self, id: OperationId) -> Option<&Record> {
        self.active
            .iter()
            .filter_map(|active| active.record.as_ref())
            .chain(self.records.iter())
            .find(|record| record.id == id)
    }
    pub fn cleanup_pending(&self) -> bool {
        self.active.iter().any(|active| active.retained.is_some())
    }
    pub fn admitted_bytes(&self) -> usize {
        self.budget.used()
    }
    pub(crate) fn presentation(
        &self,
        width: u16,
        height: u16,
    ) -> Result<Lease, crate::view::Error> {
        // A path can expand to escaped OS bytes and combining graphemes. This
        // conservative row charge also covers formatting scratch and Row data.
        let bytes = usize::from(height)
            .checked_mul(32768usize.max(usize::from(width) * 8) + 512)
            .and_then(|rows| rows.checked_add(4096))
            .ok_or(crate::view::Error::GitCapacity)?;
        self.budget
            .reserve(bytes)
            .map_err(|_| crate::view::Error::GitCapacity)
    }
    pub fn busy(&self) -> bool {
        !self.active.is_empty()
    }
    pub fn progress(&self) -> impl Iterator<Item = Progress> + '_ {
        self.active
            .iter()
            .map(|active| *active.progress.lock().expect("Git progress owner"))
    }

    /// Discover only after explicit invocation. Unknown contexts do not race
    /// an already running child in a possibly identical repository.
    pub fn open(&mut self, directory: &Path) -> Result<OperationId, String> {
        if self.busy() {
            return Err("Git context checking waits for the active owned operation".into());
        }
        if directory.as_os_str().as_encoded_bytes().len() > 4096 {
            return Err("Git directory exceeds its 4096-byte path bound".into());
        }
        self.start(directory.to_owned(), None, work::Request::Status)
    }

    pub fn refresh(&mut self, context: ContextId) -> Result<OperationId, String> {
        let known = self
            .context(context)
            .ok_or("Git context no longer exists")?;
        if self
            .progress()
            .any(|p| p.context == Some(context) || p.context.is_none())
        {
            return Err("Git repository already owns an active operation".into());
        }
        let directory = known
            .observation
            .identity
            .worktree
            .as_ref()
            .unwrap_or(&known.observation.identity.git_directory)
            .clone();
        self.start(directory, Some(context), work::Request::Status)
    }

    /// Compare one exact path in its captured Status generation. An escaped
    /// label, path in another layer, or stale observation cannot supply a target.
    pub fn compare(
        &mut self,
        context: ContextId,
        generation: u64,
        group: Group,
        path: &Path,
    ) -> Result<OperationId, String> {
        let known = self
            .context(context)
            .ok_or("Git context no longer exists")?;
        if known.failure.is_some() || known.observation.generation != generation {
            return Err("Git comparison requires a current confirmed Status generation".into());
        }
        if self
            .progress()
            .any(|p| p.context == Some(context) || p.context.is_none())
        {
            return Err("Git repository already owns an active operation".into());
        }
        let root = known
            .observation
            .identity
            .worktree
            .as_ref()
            .ok_or("Bare repositories have no working-tree comparison")?
            .clone();
        let change = known
            .observation
            .status
            .as_ref()
            .and_then(|status| status.group(group).find(|change| change.path == path))
            .ok_or("Selected Git path/layer is no longer admitted")?
            .clone();
        let target = diff::Target {
            observation: known.observation.clone(),
            group,
            change,
        };
        self.start(root, Some(context), work::Request::Diff(target))
    }

    pub fn detail_current(&self, detail: &Diff) -> bool {
        self.context(detail.observation.context)
            .is_some_and(|context| {
                context.observation.generation == detail.observation.generation
                    && context.failure.is_none()
                    && context.detail_failure.is_none()
                    && detail.verified
                    && detail.completeness == Completeness::Complete
            })
    }

    fn start(
        &mut self,
        directory: PathBuf,
        context: Option<ContextId>,
        request: work::Request,
    ) -> Result<OperationId, String> {
        if self.active.len() >= MAX_CONTEXTS {
            return Err("Git active context limit reached".into());
        }
        if self.active.len() >= self.policy.max_process_records {
            return Err("Git active process record limit reached".into());
        }
        let record_bytes = self
            .policy
            .max_output_bytes
            .min(self.policy.max_process_record_bytes - 16384)
            + 16384;
        let active_bytes = self.active.len() * record_bytes;
        if active_bytes + record_bytes > self.policy.max_process_record_bytes {
            return Err(
                "Git process-record byte admission is occupied; live work was preserved".into(),
            );
        }
        while !self.records.is_empty()
            && (self.active.len() + self.records.len() >= self.policy.max_process_records
                || self.records.iter().map(Record::bytes).sum::<usize>()
                    + active_bytes
                    + record_bytes
                    > self.policy.max_process_record_bytes)
        {
            self.records.remove(0);
        }
        // Reserve raw output, scratch and diagnostic capacities before launch.
        // The worker admits the actual bounded path candidate before parsing;
        // a large configured ceiling does not allocate an empty worst-case list.
        let bytes = self.policy.max_output_bytes
            + budget::SCRATCH_BYTES
            + 2 * self.policy.max_diagnostic_bytes
            + 131072;
        let admission = self.budget.reserve(bytes)?;
        self.next = self
            .next
            .checked_add(1)
            .ok_or("Git operation identity exhausted")?;
        let id = OperationId(self.next);
        let progress = Arc::new(Mutex::new(Progress {
            operation: id,
            kind: request.kind(),
            context,
            phase: Phase::Checking,
            child: None,
            started: Instant::now(),
            stopping: false,
        }));
        let cancel = Cancellation::default();
        let thread = work::start(
            self.policy.clone(),
            directory,
            request,
            progress.clone(),
            cancel.clone(),
            admission,
            self.wake.clone(),
        )?;
        self.active.push(Active {
            progress,
            cancel,
            thread: Some(thread),
            retained: None,
            record: None,
        });
        Ok(id)
    }

    /// Cancellation targets an exact live operation. Inspection and UI hiding
    /// never call this method. Failed cleanup can be retried through this path.
    pub fn cancel(&mut self, operation: OperationId) -> Result<(), String> {
        let active = self
            .active
            .iter_mut()
            .find(|a| a.progress.lock().expect("Git progress owner").operation == operation)
            .ok_or("Selected Git operation is no longer active")?;
        active.cancel.cancel();
        active.progress.lock().expect("Git progress owner").stopping = true;
        if active.retained.is_some() {
            let progress = active.progress.clone();
            let wake = self.wake.clone();
            let owned = Arc::new(Mutex::new(None::<(work::Retained, Record)>));
            let transfer = owned.clone();
            // The worker must exist before ownership leaves the registry.
            let (send, receive) = std::sync::mpsc::sync_channel::<()>(1);
            let thread = std::thread::Builder::new()
                .name("git-cleanup".into())
                .spawn(move || {
                    receive.recv().expect("Git cleanup transfer");
                    let (mut child, mut record) = transfer
                        .lock()
                        .expect("Git cleanup owner")
                        .take()
                        .expect("owned Git cleanup");
                    let result = child.child.stop(false);
                    record.exit = result.as_ref().ok().copied();
                    record.ended = Instant::now();
                    record.outcome = if result.is_ok() {
                        Outcome::Cancelled
                    } else {
                        Outcome::CleanupPending
                    };
                    record.diagnostic = result.err();
                    if !record_cleanup_pending(&child.child) {
                        progress.lock().expect("Git progress owner").child = None;
                    }
                    wake();
                    work::Finished {
                        publication: Err("Git cancelled".into()),
                        record,
                        retained: (record_cleanup_pending(&child.child)).then_some(child),
                    }
                })
                .map_err(|e| format!("Start Git cleanup worker: {e}"))?;
            *owned.lock().expect("Git cleanup owner") = Some((
                active.retained.take().expect("retained child"),
                active.record.take().expect("retained record"),
            ));
            send.send(()).expect("Git cleanup worker started");
            active.thread = Some(thread);
        }
        Ok(())
    }

    /// At most four result events are consumed. Worker-produced observations
    /// are moved, not copied or reparsed on the editing thread.
    pub fn poll(&mut self) -> Vec<Update> {
        let mut updates = Vec::new();
        let mut index = 0;
        while index < self.active.len() {
            let active = &mut self.active[index];
            if !active.thread.as_ref().is_some_and(JoinHandle::is_finished) {
                index += 1;
                continue;
            }
            let finished = active
                .thread
                .take()
                .expect("finished Git worker")
                .join()
                .expect("Git operation worker failed");
            if let Some(child) = finished.retained {
                active.retained = Some(child);
                active.record = Some(finished.record);
                active.progress.lock().expect("Git progress owner").phase = Phase::Cleanup;
                let progress = *active.progress.lock().expect("Git progress owner");
                if matches!(
                    progress.kind,
                    OperationKind::Worktree
                        | OperationKind::Stash
                        | OperationKind::Stage
                        | OperationKind::Unstage
                        | OperationKind::Discard
                        | OperationKind::Commit
                        | OperationKind::TagCreate
                        | OperationKind::TagDelete
                        | OperationKind::SequenceControl
                        | OperationKind::Application
                        | OperationKind::ApplicationControl
                        | OperationKind::Bisect
                        | OperationKind::Integration
                        | OperationKind::Push
                        | OperationKind::Fetch
                        | OperationKind::Branch
                ) && let Some(context) = progress
                    .context
                    .and_then(|id| self.contexts.iter_mut().find(|c| c.id == id))
                {
                    context.failure = Some(progress.operation);
                }
                updates.push(Update {
                    operation: progress.operation,
                    kind: progress.kind,
                    context: progress.context,
                    published: false,
                });
                index += 1;
                continue;
            }
            let progress = *active.progress.lock().expect("Git progress owner");
            let mut record = finished.record;
            let mut published = false;
            let context = match finished.publication {
                Ok(work::Publication::Status(mut observation)) => {
                    let prior = self
                        .contexts
                        .iter()
                        .position(|c| c.observation.identity == observation.identity);
                    if let Some(prior) = prior {
                        observation.context = self.contexts[prior].id;
                        let id = observation.context;
                        self.contexts[prior].observation = Arc::new(observation);
                        self.contexts[prior].failure = None;
                        self.contexts[prior].composition = None;
                        if progress.kind == OperationKind::Commit
                            && record.outcome == Outcome::Verified
                        {
                            self.contexts[prior].draft = None;
                        }
                        published = true;
                        record.context = Some(id);
                        Some(id)
                    } else if self.contexts.len() < MAX_CONTEXTS {
                        let id = observation.context;
                        self.contexts.push(Context {
                            id,
                            observation: Arc::new(observation),
                            failure: None,
                            detail: None,
                            detail_failure: None,
                            worktrees: None,
                            worktree_plan: None,
                            stashes: None,
                            stash_detail: None,
                            stash_plan: None,
                            refs: None,
                            sequence: None,
                            conflict: None,
                            resolution: None,
                            conflict_explanation: None,
                            autosquash: None,
                            application_plan: None,
                            bisect_plan: None,
                            history: None,
                            reflog: None,
                            blame: None,
                            revision_file: None,
                            revision: None,
                            tag_intent: None,
                            push_intent: None,
                            tag_draft: None,
                            draft: None,
                            composition: None,
                        });
                        published = true;
                        record.context = Some(id);
                        Some(id)
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic = Some(
                            "Git retained context limit reached; existing contexts were preserved"
                                .into(),
                        );
                        None
                    }
                }
                Ok(work::Publication::Worktrees(worktrees)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == worktrees.observation.context)
                    {
                        context.observation = worktrees.observation.clone();
                        context.worktrees = Some(Arc::new(worktrees));
                        context.worktree_plan = None;
                        context.failure = None;
                        published = true;
                    }
                    progress.context
                }
                Ok(work::Publication::WorktreePlan(plan)) => {
                    if let Some(context) = self.contexts.iter_mut().find(|c| {
                        c.id == plan.worktrees.observation.context
                            && c.worktrees
                                .as_ref()
                                .is_some_and(|w| Arc::ptr_eq(w, &plan.worktrees))
                    }) {
                        context.worktree_plan = Some(Arc::new(plan));
                        published = true;
                    }
                    progress.context
                }
                Ok(work::Publication::Stashes(stashes)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == stashes.observation.context)
                    {
                        context.observation = stashes.observation.clone();
                        context.stashes = Some(Arc::new(stashes));
                        context.stash_plan = None;
                        context.failure = None;
                        published = true;
                    }
                    progress.context
                }
                Ok(work::Publication::StashDetail(detail)) => {
                    if let Some(context) = self.contexts.iter_mut().find(|c| {
                        c.id == detail.stashes.observation.context
                            && c.stashes
                                .as_ref()
                                .is_some_and(|s| Arc::ptr_eq(s, &detail.stashes))
                    }) {
                        context.stash_detail = Some(Arc::new(detail));
                        published = true;
                    }
                    progress.context
                }
                Ok(work::Publication::StashPlan(plan)) => {
                    if let Some(context) = self.contexts.iter_mut().find(|c| {
                        c.id == plan.stashes.observation.context
                            && c.stashes
                                .as_ref()
                                .is_some_and(|s| Arc::ptr_eq(s, &plan.stashes))
                    }) {
                        context.stash_plan = Some(Arc::new(plan));
                        published = true;
                    }
                    progress.context
                }
                Ok(work::Publication::StashResult { stashes, sequence }) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == stashes.observation.context)
                    {
                        context.observation = stashes.observation.clone();
                        context.refs = Some(sequence.refs.clone());
                        context.sequence = Some(Arc::new(sequence));
                        context.stashes = Some(Arc::new(stashes));
                        context.stash_plan = None;
                        context.stash_detail = None;
                        context.failure = None;
                        context.composition = None;
                        published = true;
                    }
                    progress.context
                }
                Ok(work::Publication::BisectPlan(plan)) => {
                    if let Some(context) = self.contexts.iter_mut().find(|c| {
                        c.id == plan.refs.observation.context
                            && c.refs.as_ref().is_some_and(|r| Arc::ptr_eq(r, &plan.refs))
                    }) {
                        context.bisect_plan = Some(Arc::new(plan));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic = Some("Bisect source expired before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::ApplicationPlan(plan)) => {
                    if let Some(context) = self.contexts.iter_mut().find(|c| {
                        c.id == plan.refs.observation.context
                            && Arc::ptr_eq(&c.observation, &plan.refs.observation)
                    }) {
                        context.refs = Some(plan.refs.clone());
                        context.application_plan = Some(Arc::new(plan));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Ordered revision source expired before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::Autosquash(plan)) => {
                    if let Some(context) = self.contexts.iter_mut().find(|c| {
                        c.id == plan.refs.observation.context
                            && c.refs.as_ref().is_some_and(|r| Arc::ptr_eq(r, &plan.refs))
                    }) {
                        context.autosquash = Some(Arc::new(plan));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Autosquash source expired before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::ConflictExplanation(snapshot)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == snapshot.conflict.sequence.refs.observation.context)
                        && context
                            .conflict
                            .as_ref()
                            .is_some_and(|c| Arc::ptr_eq(c, &snapshot.conflict))
                    {
                        context.conflict_explanation = Some(Arc::new(snapshot));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Conflict explanation context expired before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::Resolution(plan)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == plan.conflict.sequence.refs.observation.context)
                        && context
                            .conflict
                            .as_ref()
                            .is_some_and(|c| Arc::ptr_eq(c, &plan.conflict))
                    {
                        context.resolution = Some(Arc::new(plan));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Conflict resolution context expired before review".into());
                    }
                    progress.context
                }
                Ok(work::Publication::Conflict(conflict)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == conflict.sequence.refs.observation.context)
                        && context
                            .sequence
                            .as_ref()
                            .is_some_and(|s| Arc::ptr_eq(s, &conflict.sequence))
                    {
                        context.conflict = Some(Arc::new(conflict));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Conflict context expired before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::Sequence(sequence)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == sequence.refs.observation.context)
                    {
                        context.observation = sequence.refs.observation.clone();
                        context.refs = Some(sequence.refs.clone());
                        context.sequence = Some(Arc::new(sequence));
                        context.resolution = None;
                        context.failure = None;
                        context.composition = None;
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Sequence context expired before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::Refs { mut refs, changed }) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == refs.observation.context)
                        && (changed
                            || (Arc::ptr_eq(&context.observation, &refs.observation)
                                && context.failure.is_none()))
                    {
                        if changed {
                            context.observation = refs.observation.clone();
                            context.failure = None;
                            context.composition = None;
                        }
                        if progress.kind == OperationKind::TagCreate
                            && record.outcome == Outcome::Verified
                        {
                            context.tag_draft = None;
                        }
                        refs.id = record.id;
                        context.refs = Some(Arc::new(refs));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Refs generation changed before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::Diff(detail)) => {
                    let context = self
                        .contexts
                        .iter_mut()
                        .find(|context| context.id == detail.observation.context);
                    if let Some(context) = context
                        && context.observation.generation == detail.observation.generation
                        && context.failure.is_none()
                    {
                        context.detail = Some(Arc::new(detail));
                        context.detail_failure = None;
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Git comparison generation changed before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::History(result)) => {
                    let observation = result.observation();
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == observation.context)
                        && Arc::ptr_eq(&context.observation, observation)
                        && context.failure.is_none()
                    {
                        match result {
                            history::ResultData::Reflog(r) => context.reflog = Some(Arc::new(r)),
                            history::ResultData::Blame(b) => context.blame = Some(Arc::new(b)),
                            history::ResultData::File(f) => {
                                context.revision_file = Some(Arc::new(f))
                            }
                            history::ResultData::Log(h) => {
                                context.history = Some(Arc::new(h));
                            }
                            history::ResultData::Show(d) => {
                                context.revision = Some(Arc::new(d));
                            }
                        }
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("History generation changed before publication".into());
                    }
                    progress.context
                }
                Ok(work::Publication::PushIntent(intent)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == intent.refs().observation.context)
                        && Arc::ptr_eq(&context.observation, &intent.refs().observation)
                        && context.failure.is_none()
                    {
                        context.push_intent = Some(Arc::new(intent));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic = Some("Push preparation generation changed".into());
                    }
                    progress.context
                }
                Ok(work::Publication::TagIntent(intent)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == intent.refs.observation.context)
                        && Arc::ptr_eq(&context.observation, &intent.refs.observation)
                        && context.failure.is_none()
                    {
                        context.tag_intent = Some(Arc::new(intent));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic = Some("Tag preparation generation changed".into());
                    }
                    progress.context
                }
                Ok(work::Publication::Composition(composition)) => {
                    if let Some(context) = self
                        .contexts
                        .iter_mut()
                        .find(|c| c.id == composition.observation.context)
                        && context.observation.generation == composition.observation.generation
                        && context.failure.is_none()
                    {
                        context.composition = Some(Arc::new(composition));
                        published = true;
                    } else {
                        record.outcome = Outcome::Failed;
                        record.diagnostic =
                            Some("Commit preparation generation changed before publication".into());
                    }
                    progress.context
                }
                Err(_error) => {
                    if let Some(context) = progress
                        .context
                        .and_then(|id| self.contexts.iter_mut().find(|c| c.id == id))
                    {
                        match progress.kind {
                            OperationKind::Worktrees
                            | OperationKind::Stashes
                            | OperationKind::Status
                            | OperationKind::Refs => context.failure = Some(record.id),
                            OperationKind::Diff => context.detail_failure = Some(record.id),
                            OperationKind::WorktreeVisit
                            | OperationKind::WorktreePrepare
                            | OperationKind::StashShow
                            | OperationKind::StashPrepare
                            | OperationKind::CommitPrepare
                            | OperationKind::Conflict
                            | OperationKind::ConflictExplain
                            | OperationKind::ConflictPrepare
                            | OperationKind::History
                            | OperationKind::Sequence
                            | OperationKind::ApplicationPrepare
                            | OperationKind::BisectPrepare
                            | OperationKind::AutosquashPrepare
                            | OperationKind::PushPrepare
                            | OperationKind::TagPrepare => {}
                            OperationKind::Worktree
                            | OperationKind::Stash
                            | OperationKind::Stage
                            | OperationKind::ConflictResolve
                            | OperationKind::Unstage
                            | OperationKind::Discard
                            | OperationKind::Commit
                            | OperationKind::TagCreate
                            | OperationKind::TagDelete
                            | OperationKind::SequenceControl
                            | OperationKind::Application
                            | OperationKind::ApplicationControl
                            | OperationKind::Bisect
                            | OperationKind::Integration
                            | OperationKind::Push
                            | OperationKind::Fetch
                            | OperationKind::Branch => context.failure = Some(record.id),
                        }
                    }
                    progress.context
                }
            };
            updates.push(Update {
                operation: record.id,
                kind: progress.kind,
                context,
                published,
            });
            self.active.remove(index);
            self.retain_record(record);
        }
        updates
    }

    fn retain_record(&mut self, record: Record) {
        while !self.records.is_empty()
            && (self.records.len() >= self.policy.max_process_records
                || self.records.iter().map(Record::bytes).sum::<usize>() + record.bytes()
                    > self.policy.max_process_record_bytes)
        {
            self.records.remove(0);
        }
        self.records.push(record);
    }

    /// No deadline is introduced while quiet. A host wake services completed
    /// workers; active-only fallbacks are for headless library sessions.
    pub fn remaining(&self) -> Option<Duration> {
        self.active
            .iter()
            .any(|a| a.thread.is_some())
            .then_some(Duration::from_millis(20))
    }
}

impl Drop for Repositories {
    fn drop(&mut self) {
        for active in &self.active {
            active.cancel.cancel();
        }
        for mut active in self.active.drain(..) {
            if let Some(thread) = active.thread.take() {
                // Bounded capture cleanup belongs to the worker. A reaper can
                // outlive a dropped library handle without blocking its UI.
                std::thread::spawn(move || {
                    let _ = thread.join();
                });
            }
        }
    }
}

fn record_cleanup_pending(child: &capture::Owned) -> bool {
    child.pending()
}
