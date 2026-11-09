use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TagKind {
    #[default]
    Lightweight,
    Annotated,
}
impl TagKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lightweight => "lightweight",
            Self::Annotated => "annotated",
        }
    }
}

/// Prepared local intent. Git validates names, target type and signing policy;
/// public fields cannot construct authority without the private admission.
#[derive(Debug)]
pub struct TagIntent {
    pub refs: Arc<Refs>,
    pub name: String,
    pub kind: TagKind,
    pub target: ObjectId,
    pub target_type: Vec<u8>,
    pub signing: bool,
    pub(in crate::git) policy: Vec<u8>,
    pub(in crate::git) _admission: Lease,
}
impl TagIntent {
    /// Exact message shown by native review and sent to Git. Signing needs a
    /// final LF to keep its signature delimiter on a separate line; unsigned
    /// verbatim annotations retain the entered bytes.
    pub fn annotation(&self, text: &str) -> Result<String, String> {
        if text.len() > MAX_COMMIT_BYTES || text.contains('\0') {
            return Err("Tag annotation exceeds 65536 bytes or contains NUL".into());
        }
        if self.kind == TagKind::Lightweight && !text.is_empty() {
            return Err("Lightweight tags have no annotation".into());
        }
        if self.kind == TagKind::Annotated && text.trim().is_empty() {
            return Err("Annotated tag requires a message".into());
        }
        if self.signing && !text.ends_with('\n') {
            if text.len() == MAX_COMMIT_BYTES {
                return Err(
                    "Signed annotation requires room for its final newline within 65536 bytes"
                        .into(),
                );
            }
            Ok(format!("{text}\n"))
        } else {
            Ok(text.into())
        }
    }
    pub(in crate::git) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.name.capacity()
            + self.target_type.capacity()
            + self.policy.capacity()
    }
}
pub(in crate::git) struct TagDraft {
    name: String,
    target: ObjectId,
    text: String,
    _admission: Lease,
}
impl Repositories {
    pub fn tag_intent(&self, context: ContextId) -> Option<&Arc<TagIntent>> {
        self.context(context)?.tag_intent.as_ref()
    }
    pub fn tag_draft(&self, intent: &TagIntent) -> Option<&str> {
        let draft = self
            .context(intent.refs.observation.context)?
            .tag_draft
            .as_ref()?;
        (draft.name == intent.name && draft.target == intent.target).then_some(draft.text.as_str())
    }
    pub fn retain_tag_draft(&mut self, intent: &TagIntent, text: &str) -> Result<(), String> {
        if text.len() > MAX_COMMIT_BYTES || text.contains('\0') {
            return Err("Tag annotation exceeds 65536 bytes or contains NUL".into());
        }
        let admission = self
            .budget
            .reserve(text.len() + intent.name.len() + std::mem::size_of::<TagDraft>())?;
        self.contexts
            .iter_mut()
            .find(|c| c.id == intent.refs.observation.context)
            .ok_or("Tag context expired")?
            .tag_draft = Some(TagDraft {
            name: intent.name.clone(),
            target: intent.target,
            text: text.into(),
            _admission: admission,
        });
        Ok(())
    }
    pub fn prepare_tag(
        &mut self,
        refs: Arc<Refs>,
        name: &str,
        kind: TagKind,
        target: ObjectId,
    ) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.completeness != Completeness::Complete {
            return Err("Tag preparation requires complete Refs".into());
        }
        if name.is_empty()
            || name.len() > 4086
            || name.contains('\0')
            || target.format() != refs.observation.identity.format
        {
            return Err("Tag name or target exceeds its exact metadata/format bound".into());
        }
        let directory = self.local_ref_directory(&refs)?;
        let admission = self.budget.reserve(name.len() + 4096)?;
        self.start(
            directory,
            Some(refs.observation.context),
            work::Request::Refs(Request::PrepareTag {
                refs,
                name: name.into(),
                kind,
                target,
                _admission: admission,
            }),
        )
    }
    /// Explicit accepted review only. The worker revalidates policy and the
    /// complete ref snapshot, retaining the annotation after any failure.
    pub fn create_tag(
        &mut self,
        intent: Arc<TagIntent>,
        text: &str,
    ) -> Result<OperationId, String> {
        self.check_refs(&intent.refs)?;
        let current = self
            .tag_intent(intent.refs.observation.context)
            .ok_or("Tag preparation expired")?;
        if !Arc::ptr_eq(current, &intent) {
            return Err("Tag preparation changed; review again".into());
        }
        intent.kind.check_text(text)?;
        let text = intent.annotation(text)?;
        self.retain_tag_draft(&intent, &text)?;
        let directory = self.local_ref_directory(&intent.refs)?;
        let admission = self.budget.reserve(text.len() + 4096)?;
        self.start(
            directory,
            Some(intent.refs.observation.context),
            work::Request::Refs(Request::CreateTag {
                intent,
                text,
                _admission: admission,
            }),
        )
    }
    pub fn delete_tag(&mut self, refs: Arc<Refs>, name: &[u8]) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.completeness != Completeness::Complete
            || refs
                .reference(name)
                .is_none_or(|r| r.kind != RefKind::Tag || !r.symbolic.is_empty())
        {
            return Err("Local tag deletion requires one exact complete nonsymbolic tag".into());
        }
        let directory = self.local_ref_directory(&refs)?;
        let admission = self.budget.reserve(name.len() + 4096)?;
        self.start(
            directory,
            Some(refs.observation.context),
            work::Request::Refs(Request::DeleteTag {
                refs,
                name: name.into(),
                _admission: admission,
            }),
        )
    }
}

impl TagKind {
    fn check_text(self, text: &str) -> Result<(), String> {
        match self {
            TagKind::Lightweight if !text.is_empty() => {
                return Err("Lightweight tags have no annotation".into());
            }
            TagKind::Annotated if text.trim().is_empty() => {
                return Err("Annotated tag requires a message".into());
            }
            _ => {}
        }
        Ok(())
    }
}
