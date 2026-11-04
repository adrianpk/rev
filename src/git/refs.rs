//! Exact, bounded ref observations and one-shot local branch requests.
use super::*;
mod tag;
mod transport;
pub(super) use tag::TagDraft;
pub use tag::{TagIntent, TagKind};
pub use transport::PushIntent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefKind {
    Branch,
    Tracking,
    Tag,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    pub name: Vec<u8>,
    pub kind: RefKind,
    pub object: ObjectId,
    pub object_type: Vec<u8>,
    pub symbolic: Vec<u8>,
    pub peeled: Option<ObjectId>,
    pub peeled_type: Vec<u8>,
    pub upstream: Vec<u8>,
    pub tracking: Vec<u8>,
}
impl Reference {
    pub fn commit(&self) -> Option<ObjectId> {
        if self.object_type == b"commit" {
            Some(self.object)
        } else if self.peeled_type == b"commit" {
            self.peeled
        } else {
            None
        }
    }
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + [
                &self.name,
                &self.object_type,
                &self.symbolic,
                &self.peeled_type,
                &self.upstream,
                &self.tracking,
            ]
            .iter()
            .map(|s| s.capacity())
            .sum::<usize>()
    }
}

#[derive(Debug)]
pub struct Refs {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub entries: Vec<Reference>,
    pub remotes: Vec<Vec<u8>>,
    pub head_ref: Vec<u8>,
    pub head: Option<ObjectId>,
    pub completeness: Completeness,
    pub(super) _admission: Lease,
}
impl Refs {
    pub fn reference(&self, name: &[u8]) -> Option<&Reference> {
        self.entries.iter().find(|r| r.name == name)
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entries.capacity() * std::mem::size_of::<Reference>()
            + self.entries.iter().map(Reference::bytes).sum::<usize>()
            + self.remotes.capacity() * std::mem::size_of::<Vec<u8>>()
            + self.remotes.iter().map(Vec::capacity).sum::<usize>()
            + self.head_ref.capacity()
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.remotes == other.remotes
            && self.head_ref == other.head_ref
            && self.head == other.head
            && self.completeness == other.completeness
    }
}

#[derive(Clone, Debug)]
pub enum BranchAction {
    Create { name: String, target: ObjectId },
    Switch { name: Vec<u8> },
    Rename { name: Vec<u8>, new_name: String },
    Delete { name: Vec<u8> },
}
impl BranchAction {
    pub(super) fn bytes(&self) -> usize {
        match self {
            Self::Create { name, .. } => name.capacity(),
            Self::Switch { name } | Self::Delete { name } => name.capacity(),
            Self::Rename { name, new_name } => name.capacity() + new_name.capacity(),
        }
    }
}
pub(super) enum Request {
    Observe(Arc<Observation>),
    PreparePush {
        refs: Arc<Refs>,
        source: Vec<u8>,
        remote: Option<Vec<u8>>,
        _admission: Lease,
    },
    Push(Arc<PushIntent>),
    Fetch {
        refs: Arc<Refs>,
        remote: Vec<u8>,
        _admission: Lease,
    },
    PrepareTag {
        refs: Arc<Refs>,
        name: String,
        kind: TagKind,
        target: ObjectId,
        _admission: Lease,
    },
    CreateTag {
        intent: Arc<TagIntent>,
        text: String,
        _admission: Lease,
    },
    DeleteTag {
        refs: Arc<Refs>,
        name: Vec<u8>,
        _admission: Lease,
    },
    Branch {
        refs: Arc<Refs>,
        action: BranchAction,
        _admission: Lease,
    },
}
impl Repositories {
    /// Observe exact local/tracking refs and both tag identities. No URL is
    /// retained in this public observation or used as a remote identity.
    pub fn refs(&mut self, context: ContextId) -> Result<OperationId, String> {
        let observation = self
            .context(context)
            .ok_or("Git context expired")?
            .observation
            .clone();
        if self
            .progress()
            .any(|p| p.context.is_none() || p.context == Some(context))
        {
            return Err("Repository already owns active Git work".into());
        }
        let directory = observation
            .identity
            .worktree
            .as_ref()
            .unwrap_or(&observation.identity.git_directory)
            .clone();
        self.start(
            directory,
            Some(context),
            work::Request::Refs(Request::Observe(observation)),
        )
    }
    fn local_ref_directory(&self, refs: &Refs) -> Result<PathBuf, String> {
        let identity = &refs.observation.identity;
        if self.progress().any(|p| {
            p.context.is_none()
                || p.context.and_then(|id| self.context(id)).is_some_and(|c| {
                    c.observation.identity.common_directory == identity.common_directory
                })
        }) {
            return Err("Repository common storage already owns active Git work".into());
        }
        Ok(identity
            .worktree
            .as_ref()
            .unwrap_or(&identity.git_directory)
            .clone())
    }
    pub fn check_refs(&self, refs: &Refs) -> Result<(), String> {
        let context = self
            .context(refs.observation.context)
            .ok_or("Git context expired")?;
        if !Arc::ptr_eq(&context.observation, &refs.observation)
            || context.failure.is_some()
            || context.refs.as_ref().is_none_or(|r| r.id != refs.id)
        {
            return Err("Refs require a current confirmed observation; refresh required".into());
        }
        Ok(())
    }
    /// Caller owns explicit review/confirmation and protects unsaved sources.
    /// The worker rechecks actual refs, HEAD and switch disk preconditions.
    pub fn branch(&mut self, refs: Arc<Refs>, action: BranchAction) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.completeness != Completeness::Complete {
            return Err("Branch action requires complete Refs".into());
        }
        if action.bytes() > 8192 {
            return Err("Branch parameters exceed metadata admission".into());
        }
        match &action {
            BranchAction::Create { name, target } => {
                parameter(name)?;
                if refs.entries.len() >= self.policy.max_refs {
                    return Err("New branch exceeds complete-ref admission".into());
                }
                if target.format() != refs.observation.identity.format {
                    return Err("Branch target object format differs".into());
                }
            }
            BranchAction::Rename { name, new_name } => {
                local(&refs, name)?;
                parameter(new_name)?;
            }
            BranchAction::Switch { name } | BranchAction::Delete { name } => {
                local(&refs, name)?;
            }
        }
        let identity = &refs.observation.identity;
        let directory = if matches!(action, BranchAction::Switch { .. }) {
            self.mutation_identity_root(identity)?
        } else {
            self.local_ref_directory(&refs)?
        };
        let admission = self
            .budget
            .reserve(action.bytes() + std::mem::size_of::<BranchAction>())?;
        self.start(
            directory,
            Some(refs.observation.context),
            work::Request::Refs(Request::Branch {
                refs,
                action,
                _admission: admission,
            }),
        )
    }
}
fn parameter(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 4085 || name.contains('\0') {
        Err("Branch name requires 1..=4085 non-NUL bytes".into())
    } else {
        Ok(())
    }
}
fn local(refs: &Refs, name: &[u8]) -> Result<(), String> {
    if refs
        .reference(name)
        .is_none_or(|r| r.kind != RefKind::Branch || !r.symbolic.is_empty())
    {
        Err("Branch action requires one exact nonsymbolic local branch".into())
    } else {
        Ok(())
    }
}
