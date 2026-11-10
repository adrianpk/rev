use super::*;
use crate::git::conflicts::resolution::fingerprint;
use crate::git::repository::worktrees::Request;
use crate::git::{WorktreeAction, WorktreeEntry, WorktreePlan, Worktrees};
use std::io::Read;

impl Runner {
    pub(in crate::git::work) fn worktrees(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::List(observation) => {
                let refs = self.integration_refs(&observation, admission)?;
                self.read_worktrees(std::sync::Arc::clone(&refs.observation), admission)
                    .map(Candidate::Worktrees)
            }
            Request::Prepare {
                worktrees, action, ..
            } => self.prepare_worktree(worktrees, action, admission),
            Request::Change(plan) => self.change_worktree(&plan, admission),
            Request::Visit {
                worktrees, path, ..
            } => self.visit_worktree(&worktrees, path, admission),
        }
    }
    fn visit_worktree(
        &mut self,
        worktrees: &Worktrees,
        path: PathBuf,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_worktree_list(worktrees, admission)?;
        let entry = worktrees
            .entry(&path)
            .ok_or("Worktree visit target disappeared")?;
        let state = self.worktree_state(entry, worktrees, false, admission)?;
        self.check_worktree_list(worktrees, admission)?;
        self.directory = path;
        let (identity, status) = self.observe(admission)?;
        if identity != state.refs.observation.identity {
            return Err("Worktree identity changed before visit publication".into());
        }
        Ok(Candidate::Status(identity, status))
    }
    fn worktree_metadata(
        &mut self,
        observation: &Observation,
        admission: &mut Lease,
    ) -> Result<Vec<(PathBuf, PathBuf)>, String> {
        let mut records = Vec::new();
        let root = observation.identity.common_directory.join("worktrees");
        let entries = match std::fs::read_dir(&root) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(records),
            Err(e) => return Err(format!("Read worktree registry: {e}")),
        };
        for entry in entries {
            self.admit_worktree_registry(records.len())?;
            let directory = entry
                .map_err(|e| format!("Read worktree registry entry: {e}"))?
                .path();
            let path = worktree_backlink(&directory)?;
            admission.grow(path.capacity() + directory.capacity() + 256)?;
            records.push((path, directory));
        }
        Ok(records)
    }
    fn admit_worktree_registry(&self, count: usize) -> Result<(), String> {
        if self.cancel.is_cancelled() || Instant::now() >= self.deadline {
            return Err("Worktree registry observation cancelled or timed out".into());
        }
        if count == self.policy.max_refs {
            return Err("Worktree registry exceeds complete count admission".into());
        }
        Ok(())
    }
    fn read_worktrees(
        &mut self,
        observation: Arc<Observation>,
        admission: &mut Lease,
    ) -> Result<Worktrees, String> {
        let bytes = self.command(
            &["worktree", "list", "--porcelain", "-z", "--expire=now"],
            Phase::Refreshing,
            false,
        )?;
        admission.grow(bytes.len() * 4 + 4096)?;
        let metadata = self.worktree_metadata(&observation, admission)?;
        let mut catalog = Catalog {
            entries: Vec::new(),
            completeness: Completeness::Complete,
            max_refs: self.policy.max_refs,
            format: observation.identity.format,
        };
        for field in bytes.split(|b| *b == 0) {
            if !catalog.field(field, admission)? {
                break;
            }
        }
        let Catalog {
            mut entries,
            completeness,
            ..
        } = catalog;
        for entry in &mut entries {
            self.admit_worktree(entry, &observation, &metadata)?;
        }
        let size = std::mem::size_of::<Worktrees>()
            + entries.capacity() * std::mem::size_of::<WorktreeEntry>()
            + entries
                .iter()
                .map(|e| {
                    e.path.capacity()
                        + e.git_directory.capacity()
                        + e.branch.capacity()
                        + e.locked.as_ref().map_or(0, Vec::capacity)
                        + e.prunable.as_ref().map_or(0, Vec::capacity)
                })
                .sum::<usize>();
        admission.grow(size)?;
        let id = self.progress.lock().expect("Git progress owner").operation;
        Ok(Worktrees {
            id,
            observation,
            entries,
            completeness,
            _admission: admission.split(size),
        })
    }
    fn admit_worktree(
        &mut self,
        entry: &mut WorktreeEntry,
        observation: &Observation,
        metadata: &[(PathBuf, PathBuf)],
    ) -> Result<(), String> {
        entry.root_stamp = worktree_root_stamp(&entry.path)?;
        entry.git_directory = metadata.iter().find(|(p, _)| *p == entry.path).map_or_else(
            || observation.identity.common_directory.clone(),
            |(_, d)| d.clone(),
        );
        entry.directory_stamp = fingerprint(
            &std::fs::symlink_metadata(&entry.git_directory)
                .map_err(|e| format!("Observe per-worktree directory: {e}"))?,
        );
        if entry.root_stamp.is_some() {
            let canonical = std::fs::canonicalize(&entry.path)
                .map_err(|e| format!("Resolve worktree root: {e}"))?;
            if canonical != entry.path {
                return Err("Worktree root has changed or crosses a symbolic alias".into());
            }
            if entry.bare {
                entry.available = canonical == observation.identity.common_directory;
            } else if std::fs::symlink_metadata(entry.path.join(".git")).is_ok() {
                let directory = self.worktree_git_directory(entry)?;
                entry.available = directory == entry.git_directory;
            }
        }
        Ok(())
    }
    fn worktree_git_directory(&mut self, entry: &WorktreeEntry) -> Result<PathBuf, String> {
        let bytes = self
            .command_os(
                &[
                    "rev-parse".into(),
                    "--resolve-git-dir".into(),
                    entry.path.join(".git").into_os_string(),
                ],
                Phase::Checking,
                false,
                None,
            )?
            .0;
        let bytes = bytes
            .strip_suffix(b"\n")
            .ok_or("Worktree Git directory response has no final newline")?;
        let directory = std::fs::canonicalize(PathBuf::from(OsString::from_vec(bytes.into())))
            .map_err(|e| format!("Resolve per-worktree Git identity: {e}"))?;
        Ok(directory)
    }
    fn check_worktree_list(
        &mut self,
        expected: &Worktrees,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let refs = self.integration_refs(&expected.observation, admission)?;
        let actual = self.read_worktrees(std::sync::Arc::clone(&refs.observation), admission)?;
        if !expected.same(&actual) {
            return Err("Worktree paths, identities, refs, lock or prune state changed while review was open".into());
        }
        Ok(())
    }
    fn normalized_destination(path: &Path) -> Result<PathBuf, String> {
        let mut result = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::RootDir => result.push("/"),
                std::path::Component::Normal(c) => result.push(c),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    if !result.pop() {
                        return Err("Worktree destination escapes its absolute root".into());
                    }
                }
                _ => return Err("Worktree destination must be absolute".into()),
            }
        }
        if result.file_name().is_none() {
            return Err("Worktree destination cannot be a filesystem root".into());
        }
        Ok(result)
    }
    fn destination_stamp(path: &Path) -> Result<[u64; 8], String> {
        match std::fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("Worktree destination already exists or cannot be checked".into()),
        }
        let parent = path.parent().ok_or("Worktree destination has no parent")?;
        if std::fs::canonicalize(parent)
            .map_err(|e| format!("Resolve worktree destination parent: {e}"))?
            != parent
        {
            return Err("Worktree destination parent crosses a symbolic alias".into());
        }
        let metadata = std::fs::symlink_metadata(parent)
            .map_err(|e| format!("Observe worktree destination parent: {e}"))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("Worktree destination parent is not an ordinary directory".into());
        }
        Ok(fingerprint(&metadata))
    }
    fn worktree_state(
        &mut self,
        entry: &WorktreeEntry,
        list: &Worktrees,
        clean: bool,
        admission: &mut Lease,
    ) -> Result<Sequence, String> {
        if !entry.available || entry.bare {
            return Err("Worktree has no available editable working tree".into());
        }
        let original = std::mem::replace(&mut self.directory, entry.path.clone());
        let result = State { entry, list, clean }.capture(self, admission);
        self.directory = original;
        result
    }
    fn worktree_source(
        &mut self,
        worktrees: &Worktrees,
        admission: &mut Lease,
    ) -> Result<Sequence, String> {
        self.check_worktree_list(worktrees, admission)?;
        let refs = self.integration_refs(&worktrees.observation, admission)?;
        if refs.completeness != Completeness::Complete {
            return Err("Worktree actions require complete source refs".into());
        }
        self.read_sequence(&refs, admission)
    }
    fn prepare_worktree(
        &mut self,
        worktrees: Arc<Worktrees>,
        mut action: WorktreeAction,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let source = self.worktree_source(&worktrees, admission)?;
        let mut preparation = WorktreePreparation {
            worktrees: &worktrees,
            source: &source,
            target_state: None,
            parent_stamp: None,
        };
        let paths = preparation.paths(self, &mut action, admission)?;
        let WorktreePreparation {
            target_state,
            parent_stamp,
            ..
        } = preparation;
        let evidence = if source.refs.observation.status.is_some() {
            self.stash_evidence(&source, admission)?
        } else {
            [Vec::new(), Vec::new(), Vec::new()]
        };
        let size = std::mem::size_of::<WorktreePlan>()
            + paths.capacity() * std::mem::size_of::<PathBuf>()
            + paths.iter().map(PathBuf::capacity).sum::<usize>()
            + evidence.iter().map(Vec::capacity).sum::<usize>()
            + 16384;
        admission.grow(size)?;
        self.checked_sequence(&source, admission)?;
        self.check_worktree_list(&worktrees, admission)?;
        Ok(Candidate::WorktreePlan(WorktreePlan {
            worktrees,
            source: Arc::new(source),
            action,
            paths,
            target_state,
            parent_stamp,
            evidence,
            _admission: admission.split(size),
        }))
    }
    fn change_worktree(
        &mut self,
        plan: &Arc<WorktreePlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        Change { plan }.run(self, admission)
    }
}

fn worktree_backlink(directory: &Path) -> Result<PathBuf, String> {
    ordinary_worktree_registry(directory)?;
    let bytes = read_worktree_backlink(directory)?;
    let bytes = bytes
        .strip_suffix(b"\n")
        .ok_or("Worktree backlink has no final newline")?;
    if bytes.len() > 4096 || bytes.contains(&0) {
        return Err("Worktree backlink exceeds exact path admission".into());
    }
    let path = PathBuf::from(OsString::from_vec(bytes.into()));
    if !path.is_absolute() || path.file_name().is_none_or(|n| n != ".git") {
        return Err("Worktree backlink is not an absolute Git-file path".into());
    }
    let path = path
        .parent()
        .ok_or("Worktree backlink has no root")?
        .to_owned();
    Ok(path)
}
fn read_worktree_backlink(directory: &Path) -> Result<Vec<u8>, String> {
    let link = directory.join("gitdir");
    let metadata =
        std::fs::symlink_metadata(&link).map_err(|e| format!("Observe worktree backlink: {e}"))?;
    if !metadata.is_file() || metadata.len() > 4097 {
        return Err("Worktree backlink is not an ordinary bounded file".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(link)
        .map_err(|e| format!("Open worktree backlink: {e}"))?
        .take(4098)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read worktree backlink: {e}"))?;
    Ok(bytes)
}
fn worktree_root_stamp(path: &Path) -> Result<Option<[u64; 8]>, String> {
    match std::fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() && !m.file_type().is_symlink() => Ok(Some(fingerprint(&m))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        _ => Err("Worktree root is unavailable or is not an ordinary directory".into()),
    }
}
struct Catalog {
    entries: Vec<WorktreeEntry>,
    completeness: Completeness,
    max_refs: usize,
    format: ObjectFormat,
}
impl Catalog {
    fn field(&mut self, field: &[u8], admission: &mut Lease) -> Result<bool, String> {
        if field.is_empty() {
            return Ok(true);
        }
        if field.len() > 8192 {
            return Err("Worktree metadata field exceeds admission".into());
        }
        if let Some(path) = field.strip_prefix(b"worktree ") {
            return self.push(path, admission);
        }
        let entry = self
            .entries
            .last_mut()
            .ok_or("Worktree field precedes its path")?;
        if worktree_identity_field(entry, field, self.format)? {
            return Ok(true);
        }
        worktree_annotation_field(entry, field)?;
        Ok(true)
    }
    fn push(&mut self, path: &[u8], admission: &mut Lease) -> Result<bool, String> {
        if self.entries.len() == self.max_refs {
            self.completeness = Completeness::Truncated {
                reason: "Worktree count bound",
            };
            return Ok(false);
        }
        if path.is_empty() || path.len() > 4096 {
            return Err("Worktree path exceeds exact path admission".into());
        }
        let path = PathBuf::from(OsString::from_vec(path.into()));
        if !path.is_absolute()
            || !path.components().all(|c| {
                matches!(
                    c,
                    std::path::Component::RootDir | std::path::Component::Normal(_)
                )
            })
            || self.entries.iter().any(|e| e.path == path)
        {
            return Err("Worktree path is noncanonical or duplicated".into());
        }
        admission.grow(std::mem::size_of::<WorktreeEntry>() + 1024)?;
        self.entries.push(WorktreeEntry {
            path,
            git_directory: PathBuf::new(),
            head: None,
            branch: Vec::new(),
            bare: false,
            detached: false,
            locked: None,
            prunable: None,
            available: false,
            directory_stamp: [0; 8],
            root_stamp: None,
        });
        Ok(true)
    }
}
fn worktree_identity_field(
    entry: &mut WorktreeEntry,
    field: &[u8],
    format: ObjectFormat,
) -> Result<bool, String> {
    if let Some(object) = field.strip_prefix(b"HEAD ") {
        entry.head = if object.iter().all(|b| *b == b'0') {
            None
        } else {
            Some(ObjectId::parse(object, format)?)
        };
        return Ok(true);
    }
    if let Some(branch) = field.strip_prefix(b"branch ") {
        entry.branch = branch.into();
        return Ok(true);
    }
    if field == b"bare" {
        entry.bare = true;
        return Ok(true);
    }
    if field == b"detached" {
        entry.detached = true;
        return Ok(true);
    }
    Ok(false)
}
fn worktree_annotation_field(entry: &mut WorktreeEntry, field: &[u8]) -> Result<(), String> {
    if field == b"locked" || field.starts_with(b"locked ") {
        entry.locked = Some(field.get(7..).unwrap_or(&[]).into());
    } else if field == b"prunable" || field.starts_with(b"prunable ") {
        entry.prunable = Some(field.get(9..).unwrap_or(&[]).into());
    } else {
        return Err("Unsupported worktree porcelain metadata field".into());
    }
    Ok(())
}
struct State<'a> {
    entry: &'a WorktreeEntry,
    list: &'a Worktrees,
    clean: bool,
}
impl State<'_> {
    fn capture(&self, runner: &mut Runner, admission: &mut Lease) -> Result<Sequence, String> {
        let entry = self.entry;
        let clean = self.clean;
        let mut observation = self.observation(runner, admission)?;
        let size = observation.bytes();
        admission.grow(size)?;
        observation.admission = admission.split(size);
        let refs = runner.read_refs(Arc::new(observation), admission)?;
        if refs.head != entry.head || refs.head_ref != entry.branch {
            return Err("Worktree HEAD or branch changed".into());
        }
        let sequence = runner.read_sequence(&Arc::new(refs), admission)?;
        if clean && (sequence.family.is_some() || !sequence.conflicts.is_empty()) {
            return Err("Worktree owns an active Git sequence or conflicts".into());
        }
        Ok(sequence)
    }
    fn observation(
        &self,
        runner: &mut Runner,
        admission: &mut Lease,
    ) -> Result<Observation, String> {
        let list = self.list;
        let (identity, status) = runner.observe(admission)?;
        if !self.identity_matches(&identity) {
            return Err("Selected worktree resolved to a different repository identity".into());
        }
        if status
            .as_ref()
            .is_none_or(|s| s.completeness != Completeness::Complete)
        {
            return Err("Worktree Status is incomplete".into());
        }
        runner.check_worktree_saved_clean(self.clean)?;
        let observation = Observation {
            context: list.observation.context,
            generation: list.id.0,
            identity,
            status,
            admission: admission.split(0),
        };
        Ok(observation)
    }
    fn identity_matches(&self, identity: &Identity) -> bool {
        let entry = self.entry;
        let list = self.list;
        !(identity.worktree.as_ref() != Some(&entry.path)
            || identity.git_directory != entry.git_directory
            || identity.common_directory != list.observation.identity.common_directory
            || identity.format != list.observation.identity.format)
    }
}
impl Runner {
    fn check_worktree_saved_clean(&mut self, clean: bool) -> Result<(), String> {
        if clean
            && !self
                .command(
                    &[
                        "status",
                        "--porcelain=v2",
                        "--untracked-files=all",
                        "--ignored=matching",
                        "-z",
                    ],
                    Phase::Checking,
                    false,
                )?
                .is_empty()
        {
            return Err("Worktree removal requires clean saved files, including untracked and ignored paths".into());
        }
        Ok(())
    }
}

fn ordinary_worktree_registry(directory: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(directory)
        .map_err(|e| format!("Observe worktree metadata: {e}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Worktree registry entry is not an ordinary directory".into());
    }
    Ok(())
}

/// Derived target evidence remains borrowed from one admitted source/list pair.
struct WorktreePreparation<'a> {
    worktrees: &'a Worktrees,
    source: &'a Sequence,
    target_state: Option<Arc<Sequence>>,
    parent_stamp: Option<[u64; 8]>,
}
impl WorktreePreparation<'_> {
    fn paths(
        &mut self,
        runner: &mut Runner,
        action: &mut WorktreeAction,
        admission: &mut Lease,
    ) -> Result<Vec<PathBuf>, String> {
        match action {
            WorktreeAction::Create {
                path,
                branch,
                target,
            } => Destination {
                path,
                branch,
                target: *target,
            }
            .prepare(runner, self),
            WorktreeAction::Prune => self.prune(),
            _ => self.existing(runner, action, admission),
        }
    }
    fn capacity(&self, runner: &Runner) -> Result<(), String> {
        let source = self.source;
        let worktrees = self.worktrees;
        if source.refs.entries.len() == runner.policy.max_refs {
            return Err("New worktree branch would exceed complete ref admission".into());
        }
        if worktrees.entries.len() == runner.policy.max_refs {
            return Err("New worktree would exceed complete list admission".into());
        }
        Ok(())
    }
    fn prune(&self) -> Result<Vec<PathBuf>, String> {
        let worktrees = self.worktrees;
        let mut paths = Vec::new();
        for e in worktrees
            .entries
            .iter()
            .filter(|e| e.prunable.is_some() && e.locked.is_none())
        {
            if e.git_directory == worktrees.observation.identity.common_directory
                || std::fs::symlink_metadata(&e.path).is_ok()
            {
                return Err("Prune cannot remove an existing or main worktree resource".into());
            }
            paths.push(e.path.clone());
        }
        if paths.is_empty() {
            return Err("No missing unlocked worktree records are eligible for prune".into());
        }
        Ok(paths)
    }
    fn existing(
        &mut self,
        runner: &mut Runner,
        action: &WorktreeAction,
        admission: &mut Lease,
    ) -> Result<Vec<PathBuf>, String> {
        let worktrees = self.worktrees;
        let entry = action
            .path()
            .and_then(|p| worktrees.entry(p))
            .ok_or("Select one exact worktree record")?;
        if entry.bare || entry.git_directory == worktrees.observation.identity.common_directory {
            return Err("Main or bare repository worktree metadata is protected".into());
        }
        match &action {
            WorktreeAction::Remove(_) => {
                self.remove(runner, entry, admission)?;
            }
            WorktreeAction::Lock { .. } if entry.locked.is_some() => {
                return Err("Worktree is already locked; its reason was preserved".into());
            }
            WorktreeAction::Unlock(_) if entry.locked.is_none() => {
                return Err("Worktree is not locked".into());
            }
            _ => {}
        }
        Ok(vec![entry.path.clone()])
    }
    fn remove(
        &mut self,
        runner: &mut Runner,
        entry: &WorktreeEntry,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let worktrees = self.worktrees;
        if worktrees.observation.identity.worktree.as_ref() == Some(&entry.path)
            || entry.locked.is_some()
            || entry.prunable.is_some()
        {
            return Err("Current, locked or unavailable worktree cannot be removed".into());
        }
        self.target_state = Some(Arc::new(
            runner.worktree_state(entry, worktrees, true, admission)?,
        ));
        Ok(())
    }
}
struct Destination<'a> {
    path: &'a mut PathBuf,
    branch: &'a str,
    target: ObjectId,
}
impl Destination<'_> {
    fn prepare(
        &mut self,
        runner: &mut Runner,
        preparation: &mut WorktreePreparation<'_>,
    ) -> Result<Vec<PathBuf>, String> {
        *self.path = Runner::normalized_destination(self.path)?;
        preparation.capacity(runner)?;
        self.overlap(preparation.worktrees)?;
        self.branch(runner, preparation.source)?;
        self.target(preparation.source)?;
        preparation.parent_stamp = Some(Runner::destination_stamp(self.path)?);
        Ok(vec![self.path.clone()])
    }
    fn overlap(&self, worktrees: &Worktrees) -> Result<(), String> {
        let path = &*self.path;
        if worktrees
            .entries
            .iter()
            .any(|e| path.starts_with(&e.path) || e.path.starts_with(&*path))
            || path.starts_with(&worktrees.observation.identity.common_directory)
        {
            return Err(
                "Worktree destination overlaps an existing worktree or repository metadata".into(),
            );
        }
        Ok(())
    }
    fn branch(&self, runner: &mut Runner, source: &Sequence) -> Result<(), String> {
        let branch = self.branch;
        runner.command(
            &["check-ref-format", "--branch", branch],
            Phase::Checking,
            false,
        )?;
        if source
            .refs
            .reference(format!("refs/heads/{branch}").as_bytes())
            .is_some()
        {
            return Err("Worktree branch already exists".into());
        }
        Ok(())
    }
    fn target(&self, source: &Sequence) -> Result<(), String> {
        let target = &self.target;
        if source.refs.head != Some(*target)
            && !source
                .refs
                .entries
                .iter()
                .any(|r| r.commit() == Some(*target))
        {
            return Err("Worktree target is not an exact observed commit".into());
        }
        Ok(())
    }
}

/// One exact reviewed worktree action owns launch checks and independent resulting-state proofs.
struct Change<'a> {
    plan: &'a WorktreePlan,
}
impl Change<'_> {
    fn run(&self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        self.check_source(runner, admission)?;
        self.check_destination()?;
        self.check_removal(runner, admission)?;
        self.check_prune()?;
        let args = self.arguments();
        let (executed, facts) = runner.execute_change(&args, None);
        if runner.retained.is_some() {
            runner.restore_execution(facts, true);
            return Err(
                "Worktree cleanup remains owned; no forced cleanup or rollback is inferred".into(),
            );
        }
        let result = self.observe(runner, executed.is_ok(), admission);
        runner.restore_execution(facts, executed.is_err());
        if result.is_err() {
            runner.operation_failure = Some("Worktree effects remain unverified; inspect actual paths and Git metadata without replay".into());
        }
        result
    }
    fn check_source(&self, runner: &mut Runner, admission: &mut Lease) -> Result<(), String> {
        let plan = self.plan;
        runner.checked_sequence(&plan.source, admission)?;
        runner.check_worktree_list(&plan.worktrees, admission)?;
        if plan.source.refs.observation.status.is_some()
            && plan.evidence != runner.stash_evidence(&plan.source, admission)?
        {
            return Err("Saved source state changed while worktree review was open".into());
        }
        Ok(())
    }
    fn check_destination(&self) -> Result<(), String> {
        let plan = self.plan;
        if let WorktreeAction::Create { path, .. } = &plan.action
            && Some(Runner::destination_stamp(path)?) != plan.parent_stamp
        {
            return Err("Worktree destination parent changed while review was open".into());
        }
        Ok(())
    }
    fn check_removal(&self, runner: &mut Runner, admission: &mut Lease) -> Result<(), String> {
        let plan = self.plan;
        if let Some(expected) = &plan.target_state {
            let entry = plan
                .action
                .path()
                .and_then(|p| plan.worktrees.entry(p))
                .ok_or("Worktree removal target expired")?;
            let actual = runner.worktree_state(entry, &plan.worktrees, true, admission)?;
            if !expected.same(&actual) {
                return Err("Worktree removal state changed while review was open".into());
            }
        }
        Ok(())
    }
    fn check_prune(&self) -> Result<(), String> {
        let plan = self.plan;
        if matches!(plan.action, WorktreeAction::Prune)
            && plan
                .paths
                .iter()
                .any(|p| std::fs::symlink_metadata(p).is_ok())
        {
            return Err("A missing worktree path reappeared; prune review was invalidated".into());
        }
        Ok(())
    }
    fn arguments(&self) -> Vec<OsString> {
        let plan = self.plan;
        match &plan.action {
            WorktreeAction::Create {
                path,
                branch,
                target,
            } => vec![
                "worktree".into(),
                "add".into(),
                "-b".into(),
                branch.into(),
                "--".into(),
                path.as_os_str().into(),
                target.hex().into(),
            ],
            WorktreeAction::Lock { path, reason } => vec![
                "worktree".into(),
                "lock".into(),
                "--reason".into(),
                reason.into(),
                "--".into(),
                path.as_os_str().into(),
            ],
            WorktreeAction::Unlock(path) => vec![
                "worktree".into(),
                "unlock".into(),
                "--".into(),
                path.as_os_str().into(),
            ],
            WorktreeAction::Remove(path) => vec![
                "worktree".into(),
                "remove".into(),
                "--".into(),
                path.as_os_str().into(),
            ],
            WorktreeAction::Prune => vec!["worktree".into(), "prune".into(), "--expire=now".into()],
        }
    }
    fn observe(
        &self,
        runner: &mut Runner,
        executed: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let plan = self.plan;
        let refs = runner.integration_refs(&plan.source.refs.observation, admission)?;
        let sequence = runner.read_sequence(&refs, admission)?;
        let after =
            runner.read_worktrees(std::sync::Arc::clone(&sequence.refs.observation), admission)?;
        let refs_ok = self.refs_match(&sequence);
        let mut records_ok = self.records_match(&after);
        records_ok &= self.created_state(runner, &after, admission)?;
        let source_ok = self.source_matches(runner, &sequence, admission)?;
        if executed && records_ok && refs_ok && source_ok {
            runner.effects = Effects::Verified;
        } else {
            runner.operation_failure = Some("Worktree action failed or exact saved state, branch, identity or registry verification differs; no automatic retry or force".into());
        }
        Ok(Candidate::Worktrees(after))
    }
    fn refs_match(&self, sequence: &Sequence) -> bool {
        let plan = self.plan;
        let source_refs = &plan.source.refs;
        match &plan.action {
            WorktreeAction::Create { branch, target, .. } => {
                sequence.refs.head == source_refs.head
                    && sequence.refs.head_ref == source_refs.head_ref
                    && sequence.refs.remotes == source_refs.remotes
                    && sequence.refs.entries.len() == source_refs.entries.len() + 1
                    && source_refs
                        .entries
                        .iter()
                        .all(|r| sequence.refs.reference(&r.name) == Some(r))
                    && sequence
                        .refs
                        .reference(format!("refs/heads/{branch}").as_bytes())
                        .is_some_and(|r| r.object == *target)
            }
            _ => sequence.refs.same(source_refs),
        }
    }
    fn records_match(&self, after: &Worktrees) -> bool {
        let plan = self.plan;
        let mut records_ok = after.completeness == Completeness::Complete;
        for before in &plan.worktrees.entries {
            let selected = plan.paths.contains(&before.path);
            let actual = after.entry(&before.path);
            records_ok &= self.record_matches(before, actual, selected);
        }
        let expected_count = match plan.action {
            WorktreeAction::Create { .. } => plan.worktrees.entries.len() + 1,
            WorktreeAction::Remove(_) | WorktreeAction::Prune => {
                plan.worktrees.entries.len() - plan.paths.len()
            }
            _ => plan.worktrees.entries.len(),
        };
        records_ok &= after.entries.len() == expected_count;
        records_ok
    }
    fn record_matches(
        &self,
        before: &WorktreeEntry,
        actual: Option<&WorktreeEntry>,
        selected: bool,
    ) -> bool {
        if selected {
            self.selected_record(before, actual)
        } else {
            actual.is_some_and(|e| before.same_record(e))
        }
    }
    fn selected_record(&self, before: &WorktreeEntry, actual: Option<&WorktreeEntry>) -> bool {
        let plan = self.plan;
        match &plan.action {
            WorktreeAction::Remove(_) | WorktreeAction::Prune => {
                actual.is_none()
                    && std::fs::symlink_metadata(&before.path)
                        .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                    && !before.git_directory.exists()
            }
            WorktreeAction::Lock { reason, .. } => actual.is_some_and(|e| {
                let mut expected = before.clone();
                expected.locked = Some(reason.as_bytes().into());
                expected.prunable = None;
                expected.same_record(e)
            }),
            WorktreeAction::Unlock(_) => actual.is_some_and(|e| {
                let mut expected = before.clone();
                expected.locked = None;
                expected.prunable = e.prunable.clone();
                expected.same_record(e)
            }),
            _ => actual.is_some_and(|e| before.same_record(e)),
        }
    }
    fn created_state(
        &self,
        runner: &mut Runner,
        after: &Worktrees,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let plan = self.plan;
        if let WorktreeAction::Create {
            path,
            branch,
            target,
        } = &plan.action
        {
            if let Some(entry) = after.entry(path).filter(|e| {
                e.head == Some(*target)
                    && e.branch == format!("refs/heads/{branch}").as_bytes()
                    && e.available
                    && e.git_directory != after.observation.identity.common_directory
            }) {
                runner.worktree_state(entry, after, true, admission)?;
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn source_matches(
        &self,
        runner: &mut Runner,
        sequence: &Sequence,
        admission: &mut Lease,
    ) -> Result<bool, String> {
        let plan = self.plan;
        Ok(
            sequence.refs.observation.status == plan.source.refs.observation.status
                && sequence.files == plan.source.files
                && (plan.source.refs.observation.status.is_none()
                    || runner.stash_evidence(sequence, admission)? == plan.evidence),
        )
    }
}
