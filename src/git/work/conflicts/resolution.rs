use super::*;
use crate::git::conflicts::ResolutionRequest;
use crate::git::{Resolution, ResolutionAction, SavedResult};
use std::{fs, io::Read, os::unix::fs::MetadataExt, path::Component};

impl Runner {
    pub(in crate::git::work) fn resolution(
        &mut self,
        request: ResolutionRequest,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            ResolutionRequest::Prepare { conflict, action } => self
                .prepare_resolution(conflict, action, admission)
                .map(Candidate::Resolution),
            ResolutionRequest::Apply(plan) => self.resolve_conflict(plan, admission),
        }
    }
    fn saved_result(
        &self,
        conflict: &Conflict,
        admission: &mut Lease,
    ) -> Result<Option<SavedResult>, String> {
        let root = conflict
            .sequence
            .refs
            .observation
            .identity
            .worktree
            .as_ref()
            .ok_or("Conflict result requires a working tree")?;
        let mut parent = root.clone();
        for component in conflict.change.path.components() {
            if !matches!(component, Component::Normal(_)) {
                return Err("Conflict path is not an exact relative worktree path".into());
            }
            let m = fs::symlink_metadata(&parent).map_err(|e| format!("Conflict parent: {e}"))?;
            if !m.is_dir() || m.file_type().is_symlink() {
                return Err("Conflict parent is not an ordinary canonical directory".into());
            }
            parent.push(component);
        }
        let path = parent;
        let metadata = match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("Conflict saved result: {e}")),
            Ok(m) => m,
        };
        let cap = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > cap as u64 {
            return Err("Saved conflict result is not an admitted ordinary complete file".into());
        }
        admission.grow(metadata.len() as usize + 4096)?;
        let descriptor = rustix::fs::open(
            &path,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::NONBLOCK,
            rustix::fs::Mode::empty(),
        )
        .map_err(|e| e.to_string())?;
        let file = fs::File::from(descriptor);
        let opened = file.metadata().map_err(|e| e.to_string())?;
        if fingerprint(&metadata) != fingerprint(&opened) {
            return Err("Saved conflict file identity changed during open".into());
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take(cap as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > cap || bytes.contains(&0) || std::str::from_utf8(&bytes).is_err() {
            return Err("Saved resolution is partial, binary or non-text".into());
        }
        let after = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if bytes.len() as u64 != metadata.len() || fingerprint(&metadata) != fingerprint(&after) {
            return Err("Saved conflict result changed during capture".into());
        }
        Ok(Some(SavedResult {
            bytes,
            mode: if metadata.mode() & 0o111 == 0 {
                0o100644
            } else {
                0o100755
            },
            fingerprint: fingerprint(&metadata),
        }))
    }
    fn resolution_object(
        &mut self,
        conflict: &Conflict,
        bytes: &[u8],
        admission: &mut Lease,
    ) -> Result<ObjectId, String> {
        let mut path = OsString::from("--path=");
        path.push(&conflict.change.path);
        admission.grow(bytes.len() + 4096)?;
        self.driver_admission = Some(admission.split(bytes.len() + 4096));
        let input = crate::process::input::stage(Some(bytes))?;
        let result = self.command_input(
            &[
                "--no-replace-objects".into(),
                "hash-object".into(),
                path,
                "--stdin".into(),
            ],
            Phase::Checking,
            false,
            Some(128),
            input,
        );
        if self.retained.is_none() {
            self.driver_admission = None;
        }
        let (id, limited) = result?;
        if limited {
            return Err("Saved resolution object identity was truncated".into());
        }
        ObjectId::parse(
            id.strip_suffix(b"\n").unwrap_or(&id),
            conflict.sequence.refs.observation.identity.format,
        )
    }
    fn resolution_checkout(
        &mut self,
        conflict: &Conflict,
        role: ConflictRole,
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        let side = conflict
            .side(role)
            .ok_or("Selected conflict side is missing")?;
        let mut path = OsString::from("--path=");
        path.push(&conflict.change.path);
        let (bytes, limited) = self.command_os(
            &[
                "--no-replace-objects".into(),
                "cat-file".into(),
                "--filters".into(),
                path,
                side.object.hex().into(),
            ],
            Phase::Checking,
            false,
            Some(self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES)),
        )?;
        if limited || bytes.contains(&0) || std::str::from_utf8(&bytes).is_err() {
            return Err("Selected side checkout conversion is partial, binary or non-text".into());
        }
        admission.grow(bytes.capacity() + 4096)?;
        Ok(bytes)
    }
    fn other_conflict_worktree(
        &mut self,
        conflict: &Conflict,
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        let mut excluded = OsString::from(":(top,literal,exclude)");
        excluded.push(&conflict.change.path);
        let (bytes, limited) = self.command_os(
            &[
                "diff".into(),
                "--no-ext-diff".into(),
                "--no-textconv".into(),
                "--no-color".into(),
                "--binary".into(),
                "--".into(),
                ".".into(),
                excluded,
            ],
            Phase::Checking,
            false,
            Some(self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES)),
        )?;
        if limited {
            return Err("Other working-tree evidence exceeds complete resolution admission".into());
        }
        admission.grow(bytes.capacity() + 4096)?;
        Ok(bytes)
    }
    fn prepare_resolution(
        &mut self,
        conflict: Arc<Conflict>,
        action: ResolutionAction,
        admission: &mut Lease,
    ) -> Result<Resolution, String> {
        let mode = conflict.resolution_admission(action)?;
        self.checked_sequence(&conflict.sequence, admission)?;
        let saved = self.saved_result(&conflict, admission)?;
        let other_worktree = self.other_conflict_worktree(&conflict, admission)?;
        if saved.as_ref().is_some_and(|s| s.mode != mode) {
            return Err("Saved result mode differs from the compatible checked stages".into());
        }
        let mut expected_object = None;
        let expected_bytes = match action {
            ResolutionAction::Use(role) => self.resolution_checkout(&conflict, role, admission)?,
            ResolutionAction::Stage => {
                if let Some(saved) = &saved {
                    expected_object =
                        Some(self.resolution_object(&conflict, &saved.bytes, admission)?);
                }
                Vec::new()
            }
            ResolutionAction::Delete => Vec::new(),
        };
        self.checked_sequence(&conflict.sequence, admission)?;
        if self.saved_result(&conflict, admission)? != saved {
            return Err("Saved resolution changed during preparation".into());
        }
        let size = std::mem::size_of::<Resolution>()
            + saved.as_ref().map_or(0, |s| s.bytes.capacity())
            + expected_bytes.capacity()
            + other_worktree.capacity();
        admission.grow(size)?;
        Ok(Resolution {
            conflict,
            action,
            saved,
            mode,
            expected_object,
            expected_bytes,
            other_worktree,
            _admission: admission.split(size),
        })
    }
    fn resolve_conflict(
        &mut self,
        plan: Arc<Resolution>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let fresh = self.prepare_resolution(plan.conflict.clone(), plan.action, admission)?;
        if fresh.saved != plan.saved
            || fresh.expected_object != plan.expected_object
            || fresh.expected_bytes != plan.expected_bytes
            || fresh.mode != plan.mode
            || fresh.other_worktree != plan.other_worktree
        {
            return Err(
                "Saved result, file identity or Git conversion policy changed after confirmation"
                    .into(),
            );
        }
        let sequence = &plan.conflict.sequence;
        let path = &plan.conflict.change.path;
        admission.grow(sequence.index.len() + 4096)?;
        let expected_index = resolution_index(
            &sequence.index,
            path,
            plan.action,
            plan.mode,
            plan.expected_object,
        )?;
        let args: Vec<OsString> = match plan.action {
            ResolutionAction::Use(role) => vec![
                "--literal-pathspecs".into(),
                "checkout-index".into(),
                "--force".into(),
                format!("--stage={}", role.stage()).into(),
                "--".into(),
                path.as_os_str().to_owned(),
            ],
            ResolutionAction::Delete => vec![
                "--literal-pathspecs".into(),
                "rm".into(),
                "--force".into(),
                "--".into(),
                path.as_os_str().to_owned(),
            ],
            ResolutionAction::Stage => vec![
                "--literal-pathspecs".into(),
                "add".into(),
                "--".into(),
                path.as_os_str().to_owned(),
            ],
        };
        self.checked_sequence(sequence, admission)?;
        if self.saved_result(&plan.conflict, admission)? != plan.saved {
            return Err("Saved result changed before the checked Git child".into());
        }
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Conflict resolution child cleanup remains owned; effects require inspection"
                    .into(),
            );
        }
        let result = (|| {
            let refs = self.integration_refs(&sequence.refs.observation, admission)?;
            let mut after = self.read_sequence(refs, admission)?;
            if let Some(progress) = &sequence.application {
                self.bind_application(&mut after, progress.clone())?;
            }
            let saved = self.saved_result(&plan.conflict, admission)?;
            let disk_ok = match plan.action {
                ResolutionAction::Use(_) => saved
                    .as_ref()
                    .is_some_and(|s| s.bytes == plan.expected_bytes && s.mode == plan.mode),
                ResolutionAction::Delete => saved.is_none(),
                ResolutionAction::Stage => saved == plan.saved,
            };
            let observed_index = if after.family.is_none() {
                let bytes =
                    self.command(&["ls-files", "--stage", "-z"], Phase::Verifying, false)?;
                admission.grow(bytes.capacity())?;
                Some(bytes)
            } else {
                None
            };
            let actual_index = observed_index.as_deref().unwrap_or(&after.index);
            let other_disk = self.other_conflict_worktree(&plan.conflict, admission)?;
            let other_status_same = match (
                &after.refs.observation.status,
                &sequence.refs.observation.status,
            ) {
                (Some(a), Some(b)) => a
                    .paths
                    .iter()
                    .filter(|p| p.path != *path)
                    .eq(b.paths.iter().filter(|p| p.path != *path)),
                (None, None) => true,
                _ => false,
            };
            let verified = actual_index == expected_index
                && other_disk == plan.other_worktree
                && other_status_same
                && disk_ok
                && after.family == sequence.family
                && after.files == sequence.files
                && after.refs.same(&sequence.refs);
            Ok((Candidate::Sequence(after), verified))
        })();
        self.restore_execution(facts, executed.is_err());
        match result {
            Ok((candidate, true)) if executed.is_ok() => {
                self.effects = Effects::Verified;
                Ok(candidate)
            }
            Ok((candidate, _)) => {
                self.operation_failure = Some("Conflict child failed or exact index/disk/sequence postconditions differ; no rollback or retry is inferred".into());
                Ok(candidate)
            }
            Err(e) => {
                self.operation_failure = Some(format!("Conflict effects remain unverified: {e}"));
                Err(e)
            }
        }
    }
}
fn fingerprint(m: &fs::Metadata) -> [u64; 8] {
    [
        m.dev(),
        m.ino(),
        m.len(),
        u64::from(m.mode()),
        m.mtime() as u64,
        m.mtime_nsec() as u64,
        m.ctime() as u64,
        m.ctime_nsec() as u64,
    ]
}
fn resolution_index(
    index: &[u8],
    path: &Path,
    action: ResolutionAction,
    mode: u32,
    object: Option<ObjectId>,
) -> Result<Vec<u8>, String> {
    if matches!(action, ResolutionAction::Use(_)) {
        return Ok(index.to_vec());
    }
    let mut expected = Vec::with_capacity(index.len());
    let mut inserted = false;
    for entry in index.split_inclusive(|b| *b == 0) {
        let at = entry
            .iter()
            .position(|b| *b == b'\t')
            .ok_or("Malformed captured index entry")?;
        if &entry[at + 1..entry.len() - 1] == path.as_os_str().as_bytes() {
            if !inserted && let Some(object) = object {
                expected.extend_from_slice(format!("{mode:06o} {} 0\t", object.hex()).as_bytes());
                expected.extend_from_slice(path.as_os_str().as_bytes());
                expected.push(0);
            }
            inserted = true;
        } else {
            expected.extend_from_slice(entry);
        }
    }
    if !inserted {
        return Err("Captured index lacks the exact unmerged path".into());
    }
    Ok(expected)
}
