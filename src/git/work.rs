use super::*;
use std::os::unix::ffi::OsStringExt;

mod bisect;
mod command;
mod commit;
mod completion;
use command::Invocation;
mod conflicts;
mod discard;
mod families;
mod history;
mod index;
mod integration;
mod observation;
mod refs;
mod repository;
mod request;
mod sequences;
mod worker;

pub(super) enum Request {
    Family(super::families::Request),
    Repository(super::repository::Request),
    Status,
    Conflict(super::conflicts::Request),
    Resolution(super::conflicts::ResolutionRequest),
    ConflictExplain(Arc<Conflict>),
    Application(super::sequences::Request),
    Bisect(super::bisect::Request),
    Integration(super::integration::Request),
    Refs(super::refs::Request),
    History(super::history::Request),
    Diff(diff::Target),
    Index(super::index::Request),
    Discard(Arc<Diff>),
    Commit(super::commit::Request),
    CommitPrepare(super::commit::Preparation),
}

pub(super) enum Publication {
    FamilyPlan(FamilyPlan),
    FamilyResult(FamilyResult),
    Worktrees(Worktrees),
    WorktreePlan(WorktreePlan),
    Stashes(Stashes),
    StashDetail(StashDetail),
    StashPlan(StashPlan),
    StashResult {
        stashes: Stashes,
        sequence: Sequence,
    },
    Status(Observation),
    Refs {
        refs: Refs,
        changed: bool,
    },
    History(super::history::ResultData),
    Diff(Diff),
    Composition(Composition),
    TagIntent(TagIntent),
    PushIntent(PushIntent),
    Sequence(Sequence),
    Conflict(Conflict),
    Resolution(Resolution),
    ConflictExplanation(ConflictExplanation),
    Autosquash(AutosquashPlan),
    ApplicationPlan(ApplyPlan),
    BisectPlan(BisectPlan),
}

enum Candidate {
    FamilyPlan(FamilyPlan),
    FamilyResult(FamilyResult),
    Worktrees(Worktrees),
    WorktreePlan(WorktreePlan),
    Stashes(Stashes),
    StashDetail(StashDetail),
    StashPlan(StashPlan),
    StashResult {
        stashes: Stashes,
        sequence: Sequence,
    },
    Status(Identity, Option<Status>),
    Refs {
        refs: Refs,
        changed: bool,
    },
    History(super::history::ResultData),
    Diff(diff::Prepared),
    Composition(commit::Prepared),
    TagIntent(TagIntent),
    PushIntent(PushIntent),
    Sequence(Sequence),
    Conflict(Conflict),
    Resolution(Resolution),
    ConflictExplanation(ConflictExplanation),
    Autosquash(AutosquashPlan),
    ApplicationPlan(ApplyPlan),
    BisectPlan(BisectPlan),
}

/// A retained changing child keeps its sealed driver reservation until the
/// same owner has completed cleanup. Private workspaces use the same retained
/// reservation; diagnostic records have a separate lease.
pub(super) struct Retained {
    pub child: capture::Owned,
    _driver: Option<Lease>,
}

pub(super) struct Finished {
    pub publication: Result<Publication, String>,
    pub record: Record,
    pub retained: Option<Retained>,
}

pub(super) use worker::Worker;

struct Runner {
    policy: config::Git,
    directory: PathBuf,
    deadline: Instant,
    seconds: usize,
    progress: Arc<Mutex<Progress>>,
    cancel: Cancellation,
    wake: crate::term::Waker,
    captured: usize,
    // Transport/helper and raw configuration output may contain credentials.
    // Parse it privately; public records retain safe argv, exit and outcome.
    private_output: bool,
    transport_diagnostic: Option<&'static str>,
    effects: Effects,
    in_progress: bool,
    completion: Option<Outcome>,
    accept_message: bool,
    todo_editor: Option<std::fs::File>,
    driver_admission: Option<Lease>,
    private_index: Option<PathBuf>,
    workspace: Option<Arc<crate::file::temporary::Directory>>,
    operation_failure: Option<String>,
    failure_verified: bool,
    argv: Vec<OsString>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    truncated: bool,
    exit: Option<std::process::ExitStatus>,
    end: capture::End,
    retained: Option<capture::Owned>,
}

impl Runner {}

mod detail;
