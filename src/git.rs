//! Native repository observations and owned installed-Git operations. Git keeps
//! durable repository state; this owner retains finite session observations.
pub(crate) mod budget;
mod families;
pub use families::{Family, FamilyOutput, FamilyPlan, FamilyResult};
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
pub use diff::{Baseline, Diff, DiffKind, DiffLine, DiffLineKind, Hunk, HunkId, LineRange};
mod publication;
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
    FamilyPrepare,
    Family,
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
    exclusive: bool,
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
    shell_policy: config::Shell,
    shell_startup: String,
    family_plan: Option<Arc<FamilyPlan>>,
    family_result: Option<Arc<FamilyResult>>,
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
        let shell = &snapshot.settings().shell;
        let shell_startup = crate::shell::startup_program();
        let size = 65536
            + shell.program.capacity()
            + shell.args.capacity() * std::mem::size_of::<String>()
            + shell.args.iter().map(String::capacity).sum::<usize>()
            + shell_startup.capacity();
        let admission = budget
            .reserve(size)
            .expect("fixed Git owner metadata fits an empty budget");
        Self {
            policy: snapshot.settings().git.clone(),
            shell_policy: shell.clone(),
            shell_startup,
            family_plan: None,
            family_result: None,
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
            observation: std::sync::Arc::clone(&known.observation),
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
        let exclusive = matches!(&request, work::Request::Family(families::Request::Execute(plan)) if plan.family.direct());
        if self.active.iter().any(|a| a.exclusive) || (exclusive && !self.active.is_empty()) {
            return Err(
                "Direct commands retain exclusive Git ownership until child cleanup completes"
                    .into(),
            );
        }
        self.admit_record_capacity()?;
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
        let thread = work::Worker {
            policy: self.policy.clone(),
            directory,
            request,
            progress: Arc::clone(&progress),
            cancel: cancel.clone(),
            admission,
            wake: Arc::clone(&self.wake),
        }
        .start()?;
        self.active.push(Active {
            exclusive,
            progress,
            cancel,
            thread: Some(thread),
            retained: None,
            record: None,
        });
        Ok(id)
    }

    fn admit_record_capacity(&mut self) -> Result<(), String> {
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
        Ok(())
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
            let progress = std::sync::Arc::clone(&active.progress);
            let wake = std::sync::Arc::clone(&self.wake);
            let owned = Arc::new(Mutex::new(None::<(work::Retained, Record)>));
            let transfer = std::sync::Arc::clone(&owned);
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
                self.fail_cleanup(&progress);
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
            updates.push(self.publish_finished(finished.publication, progress, &mut record));
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
