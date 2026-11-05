use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolutionAction {
    Use(ConflictRole),
    Delete,
    Stage,
}
impl ResolutionAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Use(ConflictRole::Base) => "replace result with Base",
            Self::Use(ConflictRole::Current) => "replace result with Current",
            Self::Use(ConflictRole::Applied) => "replace result with Applied",
            Self::Delete => "delete result and stage exact deletion",
            Self::Stage => "stage exact saved result",
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct SavedResult {
    pub bytes: Vec<u8>,
    pub mode: u32,
    pub(in crate::git) fingerprint: [u64; 8],
}
#[derive(Debug)]
pub struct Resolution {
    pub conflict: Arc<Conflict>,
    pub action: ResolutionAction,
    pub saved: Option<SavedResult>,
    pub expected_object: Option<ObjectId>,
    pub expected_bytes: Vec<u8>,
    pub(in crate::git) other_worktree: Vec<u8>,
    pub mode: u32,
    pub(in crate::git) _admission: Lease,
}
impl Conflict {
    pub fn resolution_admission(&self, action: ResolutionAction) -> Result<u32, String> {
        if self.completeness != Completeness::Complete
            || self.change.old_path.is_some()
            || self.change.submodule != *b"N..."
            || !matches!(
                (self.change.index, self.change.worktree),
                (b'U', b'U') | (b'A', b'A') | (b'U', b'D') | (b'D', b'U') | (b'D', b'D')
            )
            || matches!(
                self.sequence.family,
                Some(SequenceFamily::Unsupported | SequenceFamily::Bisect)
            )
        {
            return Err("Conflict action requires complete compatible path, stages and sequence evidence; structural or partial cases remain inspection only".into());
        }
        let mode = self
            .sides
            .iter()
            .flatten()
            .next()
            .ok_or("Conflict has no available stage")?
            .mode;
        if !matches!(mode, 0o100644 | 0o100755)
            || self
                .sides
                .iter()
                .flatten()
                .any(|s| s.mode != mode || !s.text)
        {
            return Err("Binary, non-text, type-change or submodule sides have no native text resolution action".into());
        }
        if let ResolutionAction::Use(role) = action
            && self.side(role).is_none()
        {
            return Err(
                "Selected complete side is absent; deletion requires its separate explicit review"
                    .into(),
            );
        }
        Ok(mode)
    }
}
pub(in crate::git) enum ResolutionRequest {
    Prepare {
        conflict: Arc<Conflict>,
        action: ResolutionAction,
    },
    Apply(Arc<Resolution>),
}
impl Repositories {
    pub(super) fn check_conflict(&self, conflict: &Arc<Conflict>) -> Result<(), String> {
        self.check_sequence(&conflict.sequence)?;
        if self
            .context(conflict.sequence.refs.observation.context)
            .and_then(|c| c.conflict.as_ref())
            .is_none_or(|c| !Arc::ptr_eq(c, conflict))
        {
            return Err("Conflict observation expired; inspect its actual stages again".into());
        }
        Ok(())
    }
    pub fn prepare_resolution(
        &mut self,
        conflict: Arc<Conflict>,
        action: ResolutionAction,
    ) -> Result<OperationId, String> {
        self.check_conflict(&conflict)?;
        conflict.resolution_admission(action)?;
        let root = self.mutation_identity_root(&conflict.sequence.refs.observation.identity)?;
        self.start(
            root,
            Some(conflict.sequence.refs.observation.context),
            work::Request::Resolution(ResolutionRequest::Prepare { conflict, action }),
        )
    }
    /// Native caller owns exact confirmation and visiting-buffer admission.
    /// Worker rechecks sequence, stages, disk identity/bytes and conversion facts.
    pub fn resolve_conflict(&mut self, plan: Arc<Resolution>) -> Result<OperationId, String> {
        self.check_conflict(&plan.conflict)?;
        let context = plan.conflict.sequence.refs.observation.context;
        if self
            .context(context)
            .and_then(|c| c.resolution.as_ref())
            .is_none_or(|p| !Arc::ptr_eq(p, &plan))
        {
            return Err("Resolution review expired; prepare the exact result again".into());
        }
        plan.conflict.resolution_admission(plan.action)?;
        let root =
            self.mutation_identity_root(&plan.conflict.sequence.refs.observation.identity)?;
        self.start(
            root,
            Some(context),
            work::Request::Resolution(ResolutionRequest::Apply(plan)),
        )
    }
}

pub(in crate::git) fn fingerprint(m: &std::fs::Metadata) -> [u64; 8] {
    use std::os::unix::fs::MetadataExt;
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
