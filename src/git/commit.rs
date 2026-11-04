//! Session-only messages and checked-index commit requests.
use super::*;

pub const MAX_COMMIT_BYTES: usize = 65536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitMode {
    Amend { head: ObjectId },
    Fixup { target: ObjectId },
}

/// Exact read-only preparation. The native caller must review this identity
/// before passing the accepted message to commit_prepared.
pub struct Composition {
    pub observation: Arc<Observation>,
    pub mode: CommitMode,
    pub message: String,
    pub(super) parents: Vec<ObjectId>,
    pub(super) fixup_prefix: Option<String>,
    pub(super) admission: Lease,
}
impl Composition {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.message.capacity()
            + self.parents.capacity() * std::mem::size_of::<ObjectId>()
            + self.fixup_prefix.as_ref().map_or(0, String::capacity)
    }
}

pub(super) struct Preparation {
    pub observation: Arc<Observation>,
    pub target: Option<String>,
    pub _admission: Lease,
}

pub(super) struct Draft {
    text: String,
    _admission: Lease,
}

pub(super) struct Request {
    pub observation: Arc<Observation>,
    pub message: String,
    pub composition: Option<Arc<Composition>>,
    pub _admission: Lease,
}

impl Repositories {
    pub fn composition(&self, context: ContextId) -> Option<&Arc<Composition>> {
        let context = self.context(context)?;
        context
            .composition
            .as_ref()
            .filter(|c| c.observation.generation == context.observation.generation)
    }

    pub fn prepare_amend(&mut self, context: ContextId) -> Result<OperationId, String> {
        let observation = self.commit_observation(context)?;
        if matches!(
            check_status(observation.status.as_ref())?.head,
            Head::Unborn { .. }
        ) {
            return Err("Amend requires an existing exact HEAD commit".into());
        }
        self.prepare_composition(observation, None)
    }

    pub fn prepare_fixup(
        &mut self,
        observation: Arc<Observation>,
        revision: &str,
    ) -> Result<OperationId, String> {
        if revision.is_empty() || revision.len() > 4086 || revision.contains('\0') {
            return Err("Fixup requires one revision within the 4096-byte argument bound".into());
        }
        self.prepare_composition(observation, Some(revision.to_owned()))
    }

    fn prepare_composition(
        &mut self,
        observation: Arc<Observation>,
        target: Option<String>,
    ) -> Result<OperationId, String> {
        let current = self.commit_observation(observation.context)?;
        if !Arc::ptr_eq(&current, &observation) {
            return Err("Commit preparation observation changed; refresh required".into());
        }
        let root = self.mutation_identity_root(&observation.identity)?;
        let admission = self.budget.reserve(
            std::mem::size_of::<Preparation>() + target.as_ref().map_or(0, String::capacity),
        )?;
        self.start(
            root,
            Some(observation.context),
            work::Request::CommitPrepare(Preparation {
                observation,
                target,
                _admission: admission,
            }),
        )
    }

    pub fn commit_prepared(
        &mut self,
        composition: Arc<Composition>,
        message: &str,
    ) -> Result<OperationId, String> {
        if let Some(prefix) = &composition.fixup_prefix
            && !message
                .strip_prefix(prefix)
                .is_some_and(|tail| tail.is_empty() || tail.starts_with('\n'))
        {
            return Err("Fixup subject must retain the exact prepared target subject".into());
        }
        self.submit_commit(composition.observation.clone(), Some(composition), message)
    }
    pub fn commit_draft(&self, context: ContextId) -> Option<&str> {
        self.context(context)?
            .draft
            .as_ref()
            .map(|d| d.text.as_str())
    }

    pub fn retain_commit_draft(&mut self, context: ContextId, text: &str) -> Result<(), String> {
        if text.len() > MAX_COMMIT_BYTES || text.contains('\0') {
            return Err("Commit draft exceeds 65536 bytes or contains NUL".into());
        }
        let admission = self
            .budget
            .reserve(text.len() + std::mem::size_of::<Draft>())?;
        let context = self
            .contexts
            .iter_mut()
            .find(|c| c.id == context)
            .ok_or("Commit repository context expired")?;
        context.draft = Some(Draft {
            text: text.to_owned(),
            _admission: admission,
        });
        Ok(())
    }

    /// Composition captures a complete index/HEAD observation. Unstaged files
    /// and native source buffers are deliberately outside this authority.
    pub fn commit_observation(&self, context: ContextId) -> Result<Arc<Observation>, String> {
        let context = self
            .context(context)
            .ok_or("Commit repository context expired")?;
        let observation = &context.observation;
        if context.failure.is_some() {
            return Err("Commit requires a fresh successful Status observation".into());
        }
        check_status(observation.status.as_ref())?;
        self.mutation_identity_root(&observation.identity)?;
        Ok(observation.clone())
    }

    pub fn commit(
        &mut self,
        observation: Arc<Observation>,
        message: &str,
    ) -> Result<OperationId, String> {
        self.submit_commit(observation, None, message)
    }

    fn submit_commit(
        &mut self,
        observation: Arc<Observation>,
        composition: Option<Arc<Composition>>,
        message: &str,
    ) -> Result<OperationId, String> {
        let current = self.commit_observation(observation.context)?;
        if !Arc::ptr_eq(&current, &observation) {
            return Err("Commit observation changed; reopen composition after refresh".into());
        }
        if message.trim().is_empty() || message.len() > MAX_COMMIT_BYTES || message.contains('\0') {
            return Err(
                "Commit requires a nonempty message of at most 65536 bytes without NUL".into(),
            );
        }
        let root = self.mutation_identity_root(&observation.identity)?;
        let admission = self
            .budget
            .reserve(message.len() + std::mem::size_of::<Request>())?;
        self.retain_commit_draft(observation.context, message)?;
        self.start(
            root,
            Some(observation.context),
            work::Request::Commit(Request {
                observation,
                message: message.to_owned(),
                composition,
                _admission: admission,
            }),
        )
    }
}

pub(super) fn check_status(status: Option<&Status>) -> Result<&Status, String> {
    let status = status.ok_or("Commit requires a working-tree repository")?;
    if status.completeness != Completeness::Complete
        || status.group(Group::Unmerged).next().is_some()
    {
        return Err("Commit requires complete Status without unmerged entries".into());
    }
    Ok(status)
}

pub(super) fn same_index(a: &Status, b: &Status) -> bool {
    if a.head != b.head {
        return false;
    }
    let mut left = a.group(Group::Staged);
    let mut right = b.group(Group::Staged);
    loop {
        let (a, b) = match (left.next(), right.next()) {
            (Some(a), Some(b)) => (a, b),
            (None, None) => return true,
            _ => return false,
        };
        if !(a.path == b.path
            && a.index == b.index
            && a.modes.get(..2) == b.modes.get(..2)
            && a.objects.get(..2) == b.objects.get(..2)
            && a.old_path.as_ref().filter(|_| a.renamed_in(Group::Staged))
                == b.old_path.as_ref().filter(|_| b.renamed_in(Group::Staged)))
        {
            return false;
        }
    }
}
