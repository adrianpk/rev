use super::*;

/// One immutable reviewed source/upstream request. Configuration and effective
/// push endpoint remain private and are never included in Debug or records.
pub struct PushIntent {
    pub(in crate::git) refs: Arc<Refs>,
    pub(in crate::git) source: Vec<u8>,
    pub(in crate::git) object: ObjectId,
    pub(in crate::git) remote: Vec<u8>,
    pub(in crate::git) destination: Vec<u8>,
    pub(in crate::git) config: Vec<u8>,
    pub(in crate::git) endpoint: Vec<u8>,
    pub(in crate::git) recurse: Vec<u8>,
    pub(in crate::git) _admission: Lease,
}
impl std::fmt::Debug for PushIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PushIntent")
            .field("source", &self.source)
            .field("object", &self.object)
            .field("remote", &self.remote)
            .field("destination", &self.destination)
            .finish_non_exhaustive()
    }
}
impl PushIntent {
    pub fn refs(&self) -> &Arc<Refs> {
        &self.refs
    }
    pub fn source(&self) -> &[u8] {
        &self.source
    }
    pub fn object(&self) -> ObjectId {
        self.object
    }
    pub fn remote(&self) -> &[u8] {
        &self.remote
    }
    pub fn destination(&self) -> &[u8] {
        &self.destination
    }
    pub(in crate::git) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + [
                &self.source,
                &self.remote,
                &self.destination,
                &self.config,
                &self.endpoint,
                &self.recurse,
            ]
            .iter()
            .map(|v| v.capacity())
            .sum::<usize>()
    }
}

impl Repositories {
    pub fn push_intent(&self, context: ContextId) -> Option<&Arc<PushIntent>> {
        self.context(context)?.push_intent.as_ref()
    }
    /// Resolve one source branch's configured upstream without contacting a
    /// remote. A caller-selected remote is a constraint, never a replacement.
    pub fn prepare_push(
        &mut self,
        refs: Arc<Refs>,
        source: &[u8],
        remote: Option<&[u8]>,
    ) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.completeness != Completeness::Complete {
            return Err("Push requires complete current Refs".into());
        }
        local(&refs, source)?;
        let source_ref = refs.reference(source).ok_or("Push source expired")?;
        if source_ref.object_type != b"commit" {
            return Err("Push requires one exact local branch commit".into());
        }
        if remote.is_some_and(|n| !refs.remotes.iter().any(|r| r == n)) {
            return Err("Push remote constraint is not an exact configured name".into());
        }
        let directory = self.local_ref_directory(&refs)?;
        let admission = self
            .budget
            .reserve(source.len() + remote.map_or(0, <[u8]>::len) + 4096)?;
        self.start(
            directory,
            Some(refs.observation.context),
            work::Request::Refs(Request::PreparePush {
                refs,
                source: source.into(),
                remote: remote.map(<[u8]>::to_vec),
                _admission: admission,
            }),
        )
    }
    /// Launch only the current explicitly accepted review, once. No retry or
    /// default/matching destination is inferred from publication failure.
    pub fn push(&mut self, intent: Arc<PushIntent>) -> Result<OperationId, String> {
        self.check_refs(&intent.refs)?;
        let context = intent.refs.observation.context;
        if !self
            .push_intent(context)
            .is_some_and(|current| Arc::ptr_eq(current, &intent))
        {
            return Err("Push preparation expired; prepare and review again".into());
        }
        let directory = self.local_ref_directory(&intent.refs)?;
        let id = self.start(
            directory,
            Some(context),
            work::Request::Refs(Request::Push(intent)),
        )?;
        self.contexts
            .iter_mut()
            .find(|c| c.id == context)
            .expect("checked push context")
            .push_intent = None;
        Ok(id)
    }

    /// Fetch one explicitly selected configured remote. The worker captures and
    /// rechecks private transport/refspec configuration; no integration is run.
    pub fn fetch(&mut self, refs: Arc<Refs>, remote: &[u8]) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.completeness != Completeness::Complete {
            return Err("Fetch requires complete current Refs".into());
        }
        if remote.is_empty() || remote.len() > 4096 || !refs.remotes.iter().any(|r| r == remote) {
            return Err("Fetch requires one exact selected configured remote".into());
        }
        let directory = self.local_ref_directory(&refs)?;
        let admission = self.budget.reserve(remote.len() + 4096)?;
        self.start(
            directory,
            Some(refs.observation.context),
            work::Request::Refs(Request::Fetch {
                refs,
                remote: remote.into(),
                _admission: admission,
            }),
        )
    }
}
