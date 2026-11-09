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
            } => {
                self.check_worktree_list(&worktrees, admission)?;
                let entry = worktrees
                    .entry(&path)
                    .ok_or("Worktree visit target disappeared")?;
                let state = self.worktree_state(entry, &worktrees, false, admission)?;
                self.check_worktree_list(&worktrees, admission)?;
                self.directory = path;
                let (identity, status) = self.observe(admission)?;
                if identity != state.refs.observation.identity {
                    return Err("Worktree identity changed before visit publication".into());
                }
                Ok(Candidate::Status(identity, status))
            }
        }
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
            if self.cancel.is_cancelled() || Instant::now() >= self.deadline {
                return Err("Worktree registry observation cancelled or timed out".into());
            }
            if records.len() == self.policy.max_refs {
                return Err("Worktree registry exceeds complete count admission".into());
            }
            let directory = entry
                .map_err(|e| format!("Read worktree registry entry: {e}"))?
                .path();
            let metadata = std::fs::symlink_metadata(&directory)
                .map_err(|e| format!("Observe worktree metadata: {e}"))?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err("Worktree registry entry is not an ordinary directory".into());
            }
            let link = directory.join("gitdir");
            let metadata = std::fs::symlink_metadata(&link)
                .map_err(|e| format!("Observe worktree backlink: {e}"))?;
            if !metadata.is_file() || metadata.len() > 4097 {
                return Err("Worktree backlink is not an ordinary bounded file".into());
            }
            let mut bytes = Vec::new();
            std::fs::File::open(link)
                .map_err(|e| format!("Open worktree backlink: {e}"))?
                .take(4098)
                .read_to_end(&mut bytes)
                .map_err(|e| format!("Read worktree backlink: {e}"))?;
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
            admission.grow(path.capacity() + directory.capacity() + 256)?;
            records.push((path, directory));
        }
        Ok(records)
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
        let mut entries: Vec<WorktreeEntry> = Vec::new();
        let mut completeness = Completeness::Complete;
        for field in bytes.split(|b| *b == 0) {
            if field.is_empty() {
                continue;
            }
            if field.len() > 8192 {
                return Err("Worktree metadata field exceeds admission".into());
            }
            if let Some(path) = field.strip_prefix(b"worktree ") {
                if entries.len() == self.policy.max_refs {
                    completeness = Completeness::Truncated {
                        reason: "Worktree count bound",
                    };
                    break;
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
                    || entries.iter().any(|e| e.path == path)
                {
                    return Err("Worktree path is noncanonical or duplicated".into());
                }
                admission.grow(std::mem::size_of::<WorktreeEntry>() + 1024)?;
                entries.push(WorktreeEntry {
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
            } else {
                let entry = entries
                    .last_mut()
                    .ok_or("Worktree field precedes its path")?;
                if let Some(object) = field.strip_prefix(b"HEAD ") {
                    entry.head = if object.iter().all(|b| *b == b'0') {
                        None
                    } else {
                        Some(ObjectId::parse(object, observation.identity.format)?)
                    };
                } else if let Some(branch) = field.strip_prefix(b"branch ") {
                    entry.branch = branch.into();
                } else if field == b"bare" {
                    entry.bare = true;
                } else if field == b"detached" {
                    entry.detached = true;
                } else if field == b"locked" || field.starts_with(b"locked ") {
                    entry.locked = Some(field.get(7..).unwrap_or(&[]).into());
                } else if field == b"prunable" || field.starts_with(b"prunable ") {
                    entry.prunable = Some(field.get(9..).unwrap_or(&[]).into());
                } else {
                    return Err("Unsupported worktree porcelain metadata field".into());
                }
            }
        }
        for entry in &mut entries {
            let actual = std::fs::symlink_metadata(&entry.path);
            entry.root_stamp = match actual {
                Ok(m) if m.is_dir() && !m.file_type().is_symlink() => Some(fingerprint(&m)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                _ => {
                    return Err(
                        "Worktree root is unavailable or is not an ordinary directory".into(),
                    );
                }
            };
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
                    let directory =
                        std::fs::canonicalize(PathBuf::from(OsString::from_vec(bytes.into())))
                            .map_err(|e| format!("Resolve per-worktree Git identity: {e}"))?;
                    entry.available = directory == entry.git_directory;
                }
            }
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
        let result = (|| {
            let (identity, status) = self.observe(admission)?;
            if identity.worktree.as_ref() != Some(&entry.path)
                || identity.git_directory != entry.git_directory
                || identity.common_directory != list.observation.identity.common_directory
                || identity.format != list.observation.identity.format
            {
                return Err("Selected worktree resolved to a different repository identity".into());
            }
            if status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete)
            {
                return Err("Worktree Status is incomplete".into());
            }
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
            let mut observation = Observation {
                context: list.observation.context,
                generation: list.id.0,
                identity,
                status,
                admission: admission.split(0),
            };
            let size = observation.bytes();
            admission.grow(size)?;
            observation.admission = admission.split(size);
            let refs = self.read_refs(Arc::new(observation), admission)?;
            if refs.head != entry.head || refs.head_ref != entry.branch {
                return Err("Worktree HEAD or branch changed".into());
            }
            let sequence = self.read_sequence(&Arc::new(refs), admission)?;
            if clean && (sequence.family.is_some() || !sequence.conflicts.is_empty()) {
                return Err("Worktree owns an active Git sequence or conflicts".into());
            }
            Ok(sequence)
        })();
        self.directory = original;
        result
    }
    fn prepare_worktree(
        &mut self,
        worktrees: Arc<Worktrees>,
        mut action: WorktreeAction,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_worktree_list(&worktrees, admission)?;
        let refs = self.integration_refs(&worktrees.observation, admission)?;
        if refs.completeness != Completeness::Complete {
            return Err("Worktree actions require complete source refs".into());
        }
        let source = self.read_sequence(&refs, admission)?;
        let mut target_state = None;
        let mut parent_stamp = None;
        let paths = match &mut action {
            WorktreeAction::Create {
                path,
                branch,
                target,
            } => {
                *path = Self::normalized_destination(path)?;
                if source.refs.entries.len() == self.policy.max_refs {
                    return Err("New worktree branch would exceed complete ref admission".into());
                }
                if worktrees.entries.len() == self.policy.max_refs {
                    return Err("New worktree would exceed complete list admission".into());
                }
                if worktrees
                    .entries
                    .iter()
                    .any(|e| path.starts_with(&e.path) || e.path.starts_with(&*path))
                    || path.starts_with(&worktrees.observation.identity.common_directory)
                {
                    return Err(
                        "Worktree destination overlaps an existing worktree or repository metadata"
                            .into(),
                    );
                }
                self.command(
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
                if source.refs.head != Some(*target)
                    && !source
                        .refs
                        .entries
                        .iter()
                        .any(|r| r.commit() == Some(*target))
                {
                    return Err("Worktree target is not an exact observed commit".into());
                }
                parent_stamp = Some(Self::destination_stamp(path)?);
                vec![path.clone()]
            }
            WorktreeAction::Prune => {
                let mut paths = Vec::new();
                for e in worktrees
                    .entries
                    .iter()
                    .filter(|e| e.prunable.is_some() && e.locked.is_none())
                {
                    if e.git_directory == worktrees.observation.identity.common_directory
                        || std::fs::symlink_metadata(&e.path).is_ok()
                    {
                        return Err(
                            "Prune cannot remove an existing or main worktree resource".into()
                        );
                    }
                    paths.push(e.path.clone());
                }
                if paths.is_empty() {
                    return Err(
                        "No missing unlocked worktree records are eligible for prune".into(),
                    );
                }
                paths
            }
            _ => {
                let entry = action
                    .path()
                    .and_then(|p| worktrees.entry(p))
                    .ok_or("Select one exact worktree record")?;
                if entry.bare
                    || entry.git_directory == worktrees.observation.identity.common_directory
                {
                    return Err("Main or bare repository worktree metadata is protected".into());
                }
                match &action {
                    WorktreeAction::Remove(_) => {
                        if worktrees.observation.identity.worktree.as_ref() == Some(&entry.path)
                            || entry.locked.is_some()
                            || entry.prunable.is_some()
                        {
                            return Err(
                                "Current, locked or unavailable worktree cannot be removed".into(),
                            );
                        }
                        target_state = Some(Arc::new(
                            self.worktree_state(entry, &worktrees, true, admission)?,
                        ));
                    }
                    WorktreeAction::Lock { .. } if entry.locked.is_some() => {
                        return Err("Worktree is already locked; its reason was preserved".into());
                    }
                    WorktreeAction::Unlock(_) if entry.locked.is_none() => {
                        return Err("Worktree is not locked".into());
                    }
                    _ => {}
                }
                vec![entry.path.clone()]
            }
        };
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
        self.checked_sequence(&plan.source, admission)?;
        self.check_worktree_list(&plan.worktrees, admission)?;
        if plan.source.refs.observation.status.is_some()
            && plan.evidence != self.stash_evidence(&plan.source, admission)?
        {
            return Err("Saved source state changed while worktree review was open".into());
        }
        if let WorktreeAction::Create { path, .. } = &plan.action
            && Some(Self::destination_stamp(path)?) != plan.parent_stamp
        {
            return Err("Worktree destination parent changed while review was open".into());
        }
        if let Some(expected) = &plan.target_state {
            let entry = plan
                .action
                .path()
                .and_then(|p| plan.worktrees.entry(p))
                .ok_or("Worktree removal target expired")?;
            let actual = self.worktree_state(entry, &plan.worktrees, true, admission)?;
            if !expected.same(&actual) {
                return Err("Worktree removal state changed while review was open".into());
            }
        }
        if matches!(plan.action, WorktreeAction::Prune)
            && plan
                .paths
                .iter()
                .any(|p| std::fs::symlink_metadata(p).is_ok())
        {
            return Err("A missing worktree path reappeared; prune review was invalidated".into());
        }
        let args: Vec<OsString> = match &plan.action {
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
        };
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Worktree cleanup remains owned; no forced cleanup or rollback is inferred".into(),
            );
        }
        let result = (|| {
            let refs = self.integration_refs(&plan.source.refs.observation, admission)?;
            let sequence = self.read_sequence(&refs, admission)?;
            let after =
                self.read_worktrees(std::sync::Arc::clone(&sequence.refs.observation), admission)?;
            let source_refs = &plan.source.refs;
            let refs_ok = match &plan.action {
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
            };
            let mut records_ok = after.completeness == Completeness::Complete;
            for before in &plan.worktrees.entries {
                let selected = plan.paths.contains(&before.path);
                let actual = after.entry(&before.path);
                records_ok &= if selected {
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
                } else {
                    actual.is_some_and(|e| before.same_record(e))
                };
            }
            let expected_count = match plan.action {
                WorktreeAction::Create { .. } => plan.worktrees.entries.len() + 1,
                WorktreeAction::Remove(_) | WorktreeAction::Prune => {
                    plan.worktrees.entries.len() - plan.paths.len()
                }
                _ => plan.worktrees.entries.len(),
            };
            records_ok &= after.entries.len() == expected_count;
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
                    self.worktree_state(entry, &after, true, admission)?;
                } else {
                    records_ok = false;
                }
            }
            let source_ok = sequence.refs.observation.status == plan.source.refs.observation.status
                && sequence.files == plan.source.files
                && (plan.source.refs.observation.status.is_none()
                    || self.stash_evidence(&sequence, admission)? == plan.evidence);
            if executed.is_ok() && records_ok && refs_ok && source_ok {
                self.effects = Effects::Verified;
            } else {
                self.operation_failure = Some("Worktree action failed or exact saved state, branch, identity or registry verification differs; no automatic retry or force".into());
            }
            Ok(Candidate::Worktrees(after))
        })();
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure = Some("Worktree effects remain unverified; inspect actual paths and Git metadata without replay".into());
        }
        result
    }
}
