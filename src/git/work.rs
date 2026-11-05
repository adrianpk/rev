use super::*;
use std::os::unix::ffi::OsStringExt;

mod bisect;
mod commit;
mod conflicts;
mod discard;
mod history;
mod index;
mod integration;
mod refs;
mod repository;
mod sequences;

pub(super) enum Request {
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
impl Request {
    pub fn kind(&self) -> OperationKind {
        match self {
            Self::Repository(super::repository::Request::Stashes(_)) => OperationKind::Stashes,
            Self::Repository(super::repository::Request::Show { .. }) => OperationKind::StashShow,
            Self::Repository(super::repository::Request::Prepare { .. }) => {
                OperationKind::StashPrepare
            }
            Self::Repository(_) => OperationKind::Stash,
            Self::ConflictExplain(_) => OperationKind::ConflictExplain,
            Self::Resolution(super::conflicts::ResolutionRequest::Prepare { .. }) => {
                OperationKind::ConflictPrepare
            }
            Self::Resolution(_) => OperationKind::ConflictResolve,
            Self::Conflict(_) => OperationKind::Conflict,
            Self::Bisect(super::bisect::Request::Prepare { .. }) => OperationKind::BisectPrepare,
            Self::Bisect(_) => OperationKind::Bisect,
            Self::Application(super::sequences::Request::Prepare { .. }) => {
                OperationKind::ApplicationPrepare
            }
            Self::Application(_) => OperationKind::Application,
            Self::Integration(super::integration::Request::Observe { .. }) => {
                OperationKind::Sequence
            }
            Self::Integration(super::integration::Request::PrepareAutosquash { .. }) => {
                OperationKind::AutosquashPrepare
            }
            Self::Integration(super::integration::Request::Control { sequence, .. })
                if sequence.application.is_some() =>
            {
                OperationKind::ApplicationControl
            }
            Self::Integration(super::integration::Request::Control { .. }) => {
                OperationKind::SequenceControl
            }
            Self::Integration(_) => OperationKind::Integration,
            Self::Refs(super::refs::Request::PreparePush { .. }) => OperationKind::PushPrepare,
            Self::Refs(super::refs::Request::Push(_)) => OperationKind::Push,
            Self::Refs(super::refs::Request::Fetch { .. }) => OperationKind::Fetch,
            Self::Refs(super::refs::Request::PrepareTag { .. }) => OperationKind::TagPrepare,
            Self::Refs(super::refs::Request::CreateTag { .. }) => OperationKind::TagCreate,
            Self::Refs(super::refs::Request::DeleteTag { .. }) => OperationKind::TagDelete,
            Self::Refs(super::refs::Request::Observe(_)) => OperationKind::Refs,
            Self::Refs(super::refs::Request::Branch { .. }) => OperationKind::Branch,
            Self::History(_) => OperationKind::History,
            Self::Status => OperationKind::Status,
            Self::Diff(_) => OperationKind::Diff,
            Self::Index(request) => request.direction.kind(),
            Self::Discard(_) => OperationKind::Discard,
            Self::Commit(_) => OperationKind::Commit,
            Self::CommitPrepare(_) => OperationKind::CommitPrepare,
        }
    }
}

pub(super) enum Publication {
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
/// same owner has completed cleanup. Diagnostic records have a separate lease.
pub(super) struct Retained {
    pub child: capture::Owned,
    _driver: Option<Lease>,
}

pub(super) struct Finished {
    pub publication: Result<Publication, String>,
    pub record: Record,
    pub retained: Option<Retained>,
}

pub(super) fn start(
    policy: config::Git,
    directory: PathBuf,
    request: Request,
    progress: Arc<Mutex<Progress>>,
    cancel: Cancellation,
    mut admission: Lease,
    wake: crate::term::Waker,
) -> Result<JoinHandle<Finished>, String> {
    std::thread::Builder::new()
        .name("git-observation".into())
        .spawn(move || {
            let private_output = matches!(
                request.kind(),
                OperationKind::Fetch | OperationKind::PushPrepare | OperationKind::Push
            );
            let seconds = match request {
                Request::Refs(
                    super::refs::Request::Fetch { .. } | super::refs::Request::Push(_),
                ) => policy.transport_timeout_seconds,
                Request::Repository(super::repository::Request::Stash(_))
                | Request::Bisect(
                    super::bisect::Request::Start(_) | super::bisect::Request::Control { .. },
                )
                | Request::Application(super::sequences::Request::Start(_))
                | Request::Integration(
                    super::integration::Request::Start { .. }
                    | super::integration::Request::Autosquash(_)
                    | super::integration::Request::Control { .. },
                )
                | Request::Resolution(super::conflicts::ResolutionRequest::Apply(_))
                | Request::Index(_)
                | Request::Discard(_)
                | Request::Commit(_)
                | Request::Refs(
                    super::refs::Request::Branch { .. }
                    | super::refs::Request::CreateTag { .. }
                    | super::refs::Request::DeleteTag { .. },
                ) => policy.operation_timeout_seconds,
                _ => policy.observation_timeout_seconds,
            };
            let mut runner = Runner {
                deadline: Instant::now() + Duration::from_secs(seconds as u64),
                seconds,
                policy,
                directory,
                progress,
                cancel,
                wake: wake.clone(),
                captured: 0,
                private_output,
                transport_diagnostic: None,
                effects: Effects::NotStarted,
                in_progress: false,
                completion: None,
                accept_message: false,
                todo_editor: None,
                driver_admission: None,
                operation_failure: None,
                failure_verified: false,
                argv: Vec::new(),
                stdout: Vec::new(),
                stderr: Vec::new(),
                truncated: false,
                exit: None,
                end: capture::End::Exited,
                retained: None,
            };
            let observed = match request {
                Request::Repository(request) => runner.repository(request, &mut admission),
                Request::ConflictExplain(conflict) => {
                    runner.explain_conflict(conflict, &mut admission)
                }
                Request::Resolution(request) => runner.resolution(request, &mut admission),
                Request::Conflict(request) => runner.conflict(request, &mut admission),
                Request::Bisect(request) => runner.bisect(request, &mut admission),
                Request::Application(request) => runner.application(request, &mut admission),
                Request::Integration(request) => runner.integration(request, &mut admission),
                Request::Refs(request) => runner.refs(request, &mut admission),
                Request::History(request) => runner
                    .history(request, &mut admission)
                    .map(Candidate::History),
                Request::Index(request) => runner.change_index(request, &mut admission),
                Request::Discard(diff) => runner.discard(diff, &mut admission),
                Request::Commit(request) => runner.commit(request, &mut admission),
                Request::CommitPrepare(request) => runner
                    .prepare_commit(request, &mut admission)
                    .map(Candidate::Composition),
                Request::Diff(target) => {
                    runner.compare(target, &mut admission).map(Candidate::Diff)
                }
                Request::Status => runner
                    .observe(&mut admission)
                    .map(|(identity, status)| Candidate::Status(identity, status)),
            };
            let mut finished = runner.finish(observed, admission);
            finished.record.admission.retain(finished.record.bytes());
            if let Ok(Publication::Status(observation)) = &mut finished.publication {
                observation.admission.retain(observation.bytes());
            }
            wake();
            finished
        })
        .map_err(|e| format!("Start native Git worker: {e}"))
}

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

impl Runner {
    fn observe(&mut self, admission: &mut Lease) -> Result<(Identity, Option<Status>), String> {
        let origin = std::fs::canonicalize(&self.directory)
            .map_err(|e| format!("Resolve Git directory: {e}"))?;
        if !origin.is_dir() {
            return Err("Git entry requires a captured directory".into());
        }
        self.directory = origin;
        let git_directory = self.directory(&["rev-parse", "--absolute-git-dir"])?;
        let common_directory =
            self.directory(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
        let bare = self.command(
            &["rev-parse", "--is-bare-repository"],
            Phase::Checking,
            false,
        )?;
        let bare = match bare.as_slice() {
            b"true\n" => true,
            b"false\n" => false,
            _ => return Err("Git lacks a machine-readable bare repository capability".into()),
        };
        let format = self.command(
            &["rev-parse", "--show-object-format"],
            Phase::Checking,
            false,
        )?;
        let format = match format.as_slice() {
            b"sha1\n" => ObjectFormat::Sha1,
            b"sha256\n" => ObjectFormat::Sha256,
            _ => return Err("Git object format is unavailable or unsupported".into()),
        };
        let worktree = if bare {
            None
        } else {
            Some(self.directory(&["rev-parse", "--show-toplevel"])?)
        };
        let identity = Identity {
            worktree,
            git_directory,
            common_directory,
            format,
        };
        if identity
            .worktree
            .as_ref()
            .is_some_and(|root| !self.directory.starts_with(root))
        {
            return Err("Git resolved a different working tree from the captured directory".into());
        }
        let status = if bare {
            None
        } else {
            let bytes = self.command(
                &[
                    "status",
                    "--porcelain=v2",
                    "--branch",
                    "--ahead-behind",
                    "--untracked-files=all",
                    "-z",
                ],
                Phase::Refreshing,
                false,
            )?;
            let records = bytes
                .iter()
                .filter(|b| **b == 0)
                .count()
                .min(self.policy.max_status_paths);
            admission.grow(
                records * (2 * std::mem::size_of::<PathChange>() + 256)
                    + 2 * bytes.len().min(self.policy.max_path_bytes),
            )?;
            let status = status::parse(
                &bytes,
                &identity,
                &self.policy,
                &self.cancel.bounded(self.deadline),
            )?;
            drop(bytes);
            self.verify_head(&status)?;
            Some(status)
        };
        Ok((identity, status))
    }

    fn verify_head(&mut self, status: &Status) -> Result<(), String> {
        let reference =
            self.command(&["symbolic-ref", "--quiet", "HEAD"], Phase::Verifying, true)?;
        let reference = reference.strip_suffix(b"\n").unwrap_or(&reference);
        let object = self.command(
            &["rev-parse", "--verify", "--quiet", "HEAD"],
            Phase::Verifying,
            true,
        )?;
        let object = object.strip_suffix(b"\n").unwrap_or(&object);
        let expected_branch = match &status.head {
            Head::Branch { name, .. } | Head::Unborn { branch: name } => Some(name),
            Head::Detached { .. } => None,
        };
        match expected_branch {
            Some(name) if reference.strip_prefix(b"refs/heads/") == Some(name.as_slice()) => (),
            None if reference.is_empty() => (),
            _ => return Err("Git HEAD changed during Status observation; refresh required".into()),
        }
        match &status.head {
            Head::Unborn { .. } if object.is_empty() => Ok(()),
            Head::Branch {
                object: expected, ..
            }
            | Head::Detached { object: expected }
                if object == expected.hex().as_bytes() =>
            {
                Ok(())
            }
            _ => Err("Git HEAD changed during Status observation; refresh required".into()),
        }
    }

    fn directory(&mut self, args: &[&str]) -> Result<PathBuf, String> {
        let bytes = self.command(args, Phase::Checking, false)?;
        let bytes = bytes
            .strip_suffix(b"\n")
            .ok_or("Git directory response lacks a final newline")?;
        if bytes.is_empty() || bytes.len() > 4096 || bytes.contains(&0) {
            return Err("Git directory exceeds its exact 4096-byte path bound".into());
        }
        let path = PathBuf::from(OsString::from_vec(bytes.to_vec()));
        if !path.is_absolute() {
            return Err("Git directory response is not absolute".into());
        }
        let path =
            std::fs::canonicalize(path).map_err(|e| format!("Validate Git directory: {e}"))?;
        if !path.is_dir() || path.as_os_str().as_encoded_bytes().len() > 4096 {
            return Err("Git returned an unavailable or overlong directory".into());
        }
        Ok(path)
    }

    fn command(
        &mut self,
        args: &[&str],
        phase: Phase,
        allow_absent: bool,
    ) -> Result<Vec<u8>, String> {
        self.command_os(
            &args.iter().map(OsString::from).collect::<Vec<_>>(),
            phase,
            allow_absent,
            None,
        )
        .map(|(bytes, _)| bytes)
    }

    fn command_os(
        &mut self,
        args: &[OsString],
        phase: Phase,
        allow_absent: bool,
        detail_cap: Option<usize>,
    ) -> Result<(Vec<u8>, bool), String> {
        self.command_input(args, phase, allow_absent, detail_cap, None)
    }
    fn command_input(
        &mut self,
        args: &[OsString],
        phase: Phase,
        allow_absent: bool,
        detail_cap: Option<usize>,
        input: Option<std::fs::File>,
    ) -> Result<(Vec<u8>, bool), String> {
        if Instant::now() >= self.deadline {
            self.end = capture::End::TimedOut;
            return Err("Git observation timed out".into());
        }
        let capture_limit = self
            .policy
            .max_output_bytes
            .min(self.policy.max_process_record_bytes - 16384);
        let remaining = capture_limit.saturating_sub(self.captured);
        if remaining == 0 {
            self.end = capture::End::OutputLimit;
            return Err("Git aggregate job output limit reached".into());
        }
        self.argv = std::iter::once(OsString::from(&self.policy.program))
            .chain(std::iter::once(OsString::from("--no-optional-locks")))
            .chain(args.iter().cloned())
            .collect();
        let program = capture::Program {
            argv: self.argv.clone(),
            directory: self.directory.clone(),
            seconds: self.seconds,
            output_bytes: detail_cap.map_or(remaining, |cap| cap.min(remaining)),
            input,
            editors: if let Some(file) = self.todo_editor.take() {
                capture::Editors::Todo(file)
            } else if self.accept_message {
                capture::Editors::AcceptMessage
            } else {
                capture::Editors::Disabled
            },
        };
        let progress = self.progress.clone();
        let wake = self.wake.clone();
        let result = program.run(&self.cancel.bounded(self.deadline), move |pid, cleanup| {
            let mut progress = progress.lock().expect("Git progress owner");
            progress.phase = if cleanup { Phase::Cleanup } else { phase };
            progress.child = Some(pid);
            drop(progress);
            wake();
        });
        {
            let mut progress = self.progress.lock().expect("Git progress owner");
            if result.retained.is_none() {
                progress.phase = phase;
                progress.child = None;
            }
        }
        self.captured += result.stdout.len() + result.stderr.len();
        // Record retention is one aggregate diagnostic prefix; complete raw
        // observation bytes stay only in this worker until parsed/rejected.
        let cap = self
            .policy
            .max_diagnostic_bytes
            .min(self.policy.max_process_record_bytes.saturating_sub(16384));
        let stderr_end = result.stderr.len().min(cap);
        let stdout_end = result.stdout.len().min(cap.saturating_sub(stderr_end));
        self.stdout = result.stdout[..stdout_end].to_vec();
        self.stderr = result.stderr[..stderr_end].to_vec();
        self.truncated = result.stdout.len() > stdout_end
            || result.stderr.len() > stderr_end
            || result.end == capture::End::OutputLimit;
        if self.private_output {
            if result.status.is_some_and(|status| !status.success()) {
                for (needle, diagnosis) in [
                    (
                        b"terminal prompts disabled".as_slice(),
                        "Noninteractive Git credential input is unavailable",
                    ),
                    (
                        b"Authentication failed".as_slice(),
                        "Git authentication failed",
                    ),
                    (
                        b"Permission denied (publickey".as_slice(),
                        "Git SSH authentication failed",
                    ),
                ] {
                    if result
                        .stderr
                        .windows(needle.len())
                        .any(|part| part == needle)
                    {
                        self.transport_diagnostic = Some(diagnosis);
                        break;
                    }
                }
            }
            self.stdout.clear();
            self.stderr.clear();
        }
        self.exit = result.status;
        self.end = if result.end == capture::End::Cancelled && !self.cancel.is_cancelled() {
            capture::End::TimedOut
        } else {
            result.end
        };
        self.retained = result.retained;
        if self.retained.is_some() {
            return Err(result
                .diagnostic
                .unwrap_or("Git cleanup remains owned".into()));
        }
        if self.end == capture::End::OutputLimit && detail_cap.is_some() {
            return Ok((result.stdout, true));
        }
        if self.end != capture::End::Exited {
            return Err(result
                .diagnostic
                .unwrap_or("Git observation did not complete".into()));
        }
        if result
            .status
            .is_some_and(|status| status.success() || (allow_absent && status.code() == Some(1)))
        {
            Ok((result.stdout, false))
        } else {
            Err("Git rejected the requested repository observation; inspect its retained diagnostic".into())
        }
    }

    fn finish(mut self, mut observed: Result<Candidate, String>, mut admission: Lease) -> Finished {
        if self.cancel.is_cancelled() {
            self.end = capture::End::Cancelled;
            observed = Err("Git observation cancelled before publication".into());
        } else if Instant::now() >= self.deadline {
            self.end = capture::End::TimedOut;
            observed = Err("Git observation timed out before publication".into());
        }
        let progress = *self.progress.lock().expect("Git progress owner");
        let outcome = if self.retained.is_some() {
            Outcome::CleanupPending
        } else {
            match self.end {
                capture::End::Cancelled => Outcome::Cancelled,
                capture::End::TimedOut => Outcome::TimedOut,
                capture::End::OutputLimit => Outcome::OutputLimit,
                _ if self.in_progress => Outcome::InProgress,
                _ if self.operation_failure.is_some()
                    && self.effects == Effects::Possible
                    && !self.failure_verified =>
                {
                    Outcome::Indeterminate
                }
                _ if self.operation_failure.is_some() => Outcome::Failed,
                _ if observed.is_ok() => self.completion.unwrap_or(Outcome::Verified),
                _ => Outcome::Failed,
            }
        };
        let diagnostic = self
            .operation_failure
            .or_else(|| observed.as_ref().err().cloned());
        let diagnostic = if self.private_output {
            let mut text = diagnostic.unwrap_or("Checked Git request completed".into());
            if let Some(reason) = self.transport_diagnostic {
                text.push_str("; ");
                text.push_str(reason);
            }
            text.push_str("; raw transport/configuration output withheld to protect credentials");
            Some(text)
        } else {
            diagnostic
        };
        let mut record = Record {
            id: progress.operation,
            kind: progress.kind,
            context: progress.context,
            started: progress.started,
            ended: Instant::now(),
            outcome,
            effects: self.effects,
            argv: self.argv,
            exit: self.exit,
            stdout: self.stdout,
            stderr: self.stderr,
            output_truncated: self.truncated,
            diagnostic,
            admission: admission.split(0),
        };
        record.admission = admission.split(record.bytes() + 4096);
        let publication = observed.and_then(|candidate| match candidate {
            Candidate::Stashes(stashes) => Ok(Publication::Stashes(stashes)),
            Candidate::StashDetail(detail) => Ok(Publication::StashDetail(detail)),
            Candidate::StashPlan(plan) => Ok(Publication::StashPlan(plan)),
            Candidate::StashResult { stashes, sequence } => {
                Ok(Publication::StashResult { stashes, sequence })
            }
            Candidate::BisectPlan(plan) => Ok(Publication::BisectPlan(plan)),
            Candidate::ApplicationPlan(plan) => Ok(Publication::ApplicationPlan(plan)),
            Candidate::Autosquash(plan) => Ok(Publication::Autosquash(plan)),
            Candidate::ConflictExplanation(snapshot) => {
                Ok(Publication::ConflictExplanation(snapshot))
            }
            Candidate::Resolution(plan) => Ok(Publication::Resolution(plan)),
            Candidate::Conflict(conflict) => Ok(Publication::Conflict(conflict)),
            Candidate::Sequence(sequence) => Ok(Publication::Sequence(sequence)),
            Candidate::PushIntent(intent) => Ok(Publication::PushIntent(intent)),
            Candidate::TagIntent(intent) => Ok(Publication::TagIntent(intent)),
            Candidate::Refs { refs, changed } => Ok(Publication::Refs { refs, changed }),
            Candidate::History(result) => Ok(Publication::History(result)),
            Candidate::Status(identity, status) => Ok(Publication::Status(Observation {
                context: progress.context.unwrap_or(ContextId(progress.operation.0)),
                generation: progress.operation.0,
                identity,
                status,
                admission,
            })),
            Candidate::Diff(prepared) => diff::prepare(
                progress.operation,
                prepared,
                admission,
                &self.cancel.bounded(self.deadline),
            )
            .map(Publication::Diff),
            Candidate::Composition(prepared) => {
                let mut composition = Composition {
                    observation: prepared.observation,
                    mode: prepared.mode,
                    message: prepared.message,
                    parents: prepared.parents,
                    fixup_prefix: prepared.fixup_prefix,
                    admission,
                };
                composition.admission.retain(composition.bytes());
                Ok(Publication::Composition(composition))
            }
        });
        if let Err(error) = &publication {
            if record.outcome == Outcome::Verified {
                record.outcome = if self.cancel.is_cancelled() {
                    Outcome::Cancelled
                } else if Instant::now() >= self.deadline {
                    Outcome::TimedOut
                } else {
                    Outcome::Failed
                };
            }
            if !self.private_output {
                record.diagnostic = Some(error.clone());
            }
        }
        if let Ok(Publication::Diff(detail)) = &publication
            && let DiffKind::Malformed(reason) = &detail.kind
        {
            record.outcome = Outcome::Failed;
            record.diagnostic = Some(reason.clone());
        }
        Finished {
            publication,
            record,
            retained: self.retained.map(|child| Retained {
                child,
                _driver: self.driver_admission,
            }),
        }
    }
}

mod detail;
