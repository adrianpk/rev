use super::*;
use std::{fs, io::Read, os::unix::fs::OpenOptionsExt};

impl Runner {
    pub(super) fn family_file(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        if self.cancel.is_cancelled() || Instant::now() >= self.deadline {
            return Err("Prepared file observation cancelled or timed out".into());
        }
        let parent = path.parent().ok_or("Prepared file has no parent")?;
        match fs::symlink_metadata(parent) {
            Ok(m)
                if !m.is_dir()
                    || m.file_type().is_symlink()
                    || fs::canonicalize(parent).map_err(|e| e.to_string())? != parent =>
            {
                return Err("Prepared file parent is not an ordinary canonical directory".into());
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(format!("Observe prepared file parent: {e}"));
            }
            _ => {}
        }
        let metadata = match fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("Observe prepared file: {e}")),
            Ok(m) => m,
        };
        let limit = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit as u64
        {
            return Err("Prepared file requires ordinary bounded bytes; symlinks and partial input are refused".into());
        }
        if fs::canonicalize(parent).map_err(|e| e.to_string())? != parent {
            return Err("Prepared file parent is not canonical".into());
        }
        let flags = rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK;
        let mut file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(i32::try_from(flags.bits()).expect("bounded file flags"))
            .open(path)
            .map_err(|e| format!("Open prepared file: {e}"))?;
        if fingerprint(&file.metadata().map_err(|e| e.to_string())?) != fingerprint(&metadata) {
            return Err("Prepared file changed before read".into());
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.by_ref()
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Read prepared file: {e}"))?;
        if bytes.len() > limit
            || fingerprint(&file.metadata().map_err(|e| e.to_string())?) != fingerprint(&metadata)
        {
            return Err("Prepared file changed or exceeded complete admission during read".into());
        }
        Ok(Some(bytes))
    }
    pub(super) fn family_current(
        &mut self,
        plan: &FamilyPlan,
        allowed_ref: Option<&std::ffi::OsStr>,
        admission: &mut Lease,
    ) -> Result<Arc<Sequence>, String> {
        let expected = plan
            .source
            .as_ref()
            .ok_or("Prepared repository source unavailable")?;
        let refs = self.integration_refs(&expected.refs.observation, admission)?;
        if refs.completeness != Completeness::Complete
            || refs.head != expected.refs.head
            || refs.head_ref != expected.refs.head_ref
        {
            return Err("Prepared command changed HEAD or lost complete ref evidence".into());
        }
        let allowed = allowed_ref.map(std::ffi::OsStr::as_encoded_bytes);
        if expected
            .refs
            .entries
            .iter()
            .filter(|r| Some(r.name.as_slice()) != allowed)
            .any(|r| {
                refs.reference(&r.name)
                    .is_none_or(|a| a.object != r.object || a.symbolic != r.symbolic)
            })
            || refs
                .entries
                .iter()
                .filter(|r| Some(r.name.as_slice()) != allowed)
                .any(|r| {
                    expected
                        .refs
                        .reference(&r.name)
                        .is_none_or(|a| a.object != r.object || a.symbolic != r.symbolic)
                })
        {
            return Err("Prepared command changed another repository ref".into());
        }
        if expected.refs.observation.status.is_some()
            && self.stash_evidence(expected, admission)? != plan.evidence
        {
            return Err(
                "Prepared command changed unrelated saved index, working or untracked bytes".into(),
            );
        }
        let sequence = self.read_sequence(refs, admission)?;
        if sequence.family.is_some() {
            return Err("Prepared command left an active Git sequence".into());
        }
        Ok(Arc::new(sequence))
    }
    /// Refresh actual compatible facts even when the caller retained an
    /// unverified postcondition. Publication alone never declares verification.
    pub(super) fn family_observed_result(
        &mut self,
        plan: Arc<FamilyPlan>,
        object: Option<ObjectId>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let expected = plan
            .observation
            .as_ref()
            .ok_or("Prepared source unavailable")?;
        let (identity, status) = self.observe(admission)?;
        if identity != expected.identity {
            return Err("Prepared repository identity changed after execution".into());
        }
        let mut source = Observation {
            context: expected.context,
            generation: self.progress.lock().expect("Git progress").operation.0,
            identity,
            status,
            admission: admission.split(0),
        };
        let size = source.bytes() + std::mem::size_of::<FamilyResult>() + 4096;
        admission.grow(size)?;
        source.admission = admission.split(source.bytes());
        Ok(Candidate::FamilyResult(FamilyResult {
            plan,
            created: None,
            object,
            source: Some(source),
            _admission: admission.split(4096),
        }))
    }
}
