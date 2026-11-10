mod inspection;
use super::*;
pub use inspection::{Attribution, Blame, Reflog, ReflogEntry, RevisionFile};
use std::path::PathBuf;

/// Full commit metadata in one finite ancestry page. Subject text is display
/// data; it never supplies revision authority.
#[derive(Debug)]
pub struct Revision {
    pub object: ObjectId,
    pub parents: Vec<ObjectId>,
    pub author: Vec<u8>,
    pub author_email: Vec<u8>,
    pub authored: Vec<u8>,
    pub committer: Vec<u8>,
    pub committer_email: Vec<u8>,
    pub committed: Vec<u8>,
    pub subject: Vec<u8>,
}
impl Revision {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.parents.capacity() * std::mem::size_of::<ObjectId>()
            + [
                &self.author,
                &self.author_email,
                &self.authored,
                &self.committer,
                &self.committer_email,
                &self.committed,
                &self.subject,
            ]
            .iter()
            .map(|v| v.capacity())
            .sum::<usize>()
    }
}

#[derive(Debug)]
pub struct HistoryPage {
    pub revisions: Vec<Revision>,
    pub(super) _admission: Lease,
}

/// Continuation is the complete immutable root plus the admitted ancestry
/// count in Git's explicit topological order, never a moving ref or row label.
#[derive(Debug)]
pub struct History {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub root: Option<ObjectId>,
    pub pages: Vec<Arc<HistoryPage>>,
    pub more: bool,
    pub page_limit: bool,
    pub(super) _admission: Lease,
}
impl History {
    pub fn revisions(&self) -> impl Iterator<Item = &Revision> {
        self.pages.iter().flat_map(|p| &p.revisions)
    }
    pub fn revision(&self, object: ObjectId) -> Option<&Revision> {
        self.revisions().find(|r| r.object == object)
    }
}

#[derive(Debug)]
pub struct RevisionPath {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub status: Vec<u8>,
}

/// Historical detail is a typed, read-only resource, independent of current
/// source buffers and of the index/worktree diff mutation API.
#[derive(Debug)]
pub struct RevisionDetail {
    pub id: OperationId,
    pub history: Arc<History>,
    pub object: ObjectId,
    pub parent: Option<ObjectId>,
    pub range: bool,
    pub metadata: Vec<u8>,
    pub paths: Vec<RevisionPath>,
    pub patch: Vec<u8>,
    pub completeness: Completeness,
    pub lines: Vec<std::ops::Range<usize>>,
    pub(super) _admission: Lease,
}

pub(super) struct FileRequest {
    pub(super) observation: Arc<Observation>,
    pub(super) object: ObjectId,
    pub(super) path: PathBuf,
    pub(super) line: usize,
    pub(super) current_path: Option<PathBuf>,
}

pub(super) struct Ancestry {
    pub(super) observation: Arc<Observation>,
    pub(super) root: Option<String>,
    pub(super) prior: Option<Arc<History>>,
    pub(super) next: bool,
}

pub(super) enum Request {
    Reflog(Arc<Observation>),
    Blame {
        observation: Arc<Observation>,
        path: PathBuf,
    },
    ShowObject {
        observation: Arc<Observation>,
        object: ObjectId,
    },
    File(FileRequest),
    Range {
        history: Arc<History>,
        base: ObjectId,
        end: ObjectId,
    },
    Log(Ancestry),
    Show {
        history: Arc<History>,
        object: ObjectId,
        parent: Option<ObjectId>,
    },
}
pub(super) enum ResultData {
    Log(History),
    Show(RevisionDetail),
    Reflog(Reflog),
    Blame(Blame),
    File(RevisionFile),
}

impl Request {
    pub(super) fn observation(&self) -> &Arc<Observation> {
        match self {
            Self::Log(request) => &request.observation,
            Self::File(request) => &request.observation,
            Self::Reflog(observation)
            | Self::Blame { observation, .. }
            | Self::ShowObject { observation, .. } => observation,
            Self::Show { history, .. } | Self::Range { history, .. } => &history.observation,
        }
    }
    pub(super) fn ancestry_owner(&self) -> Option<Arc<History>> {
        match self {
            Self::Log(request) => request.prior.as_ref().map(Arc::clone),
            Self::Show { history, .. } | Self::Range { history, .. } => Some(Arc::clone(history)),
            _ => None,
        }
    }
}
impl ResultData {
    pub(super) fn observation(&self) -> &Arc<Observation> {
        match self {
            Self::Log(h) => &h.observation,
            Self::Show(d) => &d.history.observation,
            Self::Reflog(r) => &r.observation,
            Self::Blame(b) => &b.observation,
            Self::File(f) => &f.observation,
        }
    }
}

impl Repositories {
    /// Open ancestry from checked HEAD or resolve one explicitly supplied root
    /// once. Detached callers must explicitly select their root in the UI.
    pub fn log(&mut self, context: ContextId, root: Option<&str>) -> Result<OperationId, String> {
        if root.is_some_and(|s| s.is_empty() || s.len() > 4086 || s.contains('\0')) {
            return Err("History root exceeds its nonempty revision argument bound".into());
        }
        let observation = std::sync::Arc::clone(
            &self
                .context(context)
                .ok_or("Git context no longer exists")?
                .observation,
        );
        let directory = self.history_directory(&observation)?;
        self.start(
            directory,
            Some(context),
            work::Request::History(Request::Log(Ancestry {
                observation,
                root: root.map(str::to_owned),
                prior: None,
                next: false,
            })),
        )
    }
    pub fn continue_history(
        &mut self,
        history: Arc<History>,
        next: bool,
    ) -> Result<OperationId, String> {
        self.check_history(&history)?;
        if next && (!history.more || history.page_limit) {
            return Err(if history.page_limit {
                "History retained-page limit reached"
            } else {
                "History ancestry is complete"
            }
            .into());
        }
        let directory = self.history_directory(&history.observation)?;
        self.start(
            directory,
            Some(history.observation.context),
            work::Request::History(Request::Log(Ancestry {
                observation: std::sync::Arc::clone(&history.observation),
                root: None,
                prior: Some(history),
                next,
            })),
        )
    }
    pub fn show_revision(
        &mut self,
        history: Arc<History>,
        object: ObjectId,
        parent: Option<ObjectId>,
    ) -> Result<OperationId, String> {
        self.check_history(&history)?;
        let revision = history
            .revision(object)
            .ok_or("Revision is not admitted in this History")?;
        if parent.is_some_and(|p| !revision.parents.contains(&p))
            || (parent.is_none() && !revision.parents.is_empty())
        {
            return Err("Comparison requires one exact parent of the selected revision".into());
        }
        let directory = self.history_directory(&history.observation)?;
        self.start(
            directory,
            Some(history.observation.context),
            work::Request::History(Request::Show {
                history,
                object,
                parent,
            }),
        )
    }
    pub fn show_range(
        &mut self,
        history: Arc<History>,
        base: ObjectId,
        end: ObjectId,
    ) -> Result<OperationId, String> {
        self.check_history(&history)?;
        if history.revision(base).is_none() || history.revision(end).is_none() {
            return Err("Range requires two exact admitted endpoints in base/end order".into());
        }
        let directory = self.history_directory(&history.observation)?;
        self.start(
            directory,
            Some(history.observation.context),
            work::Request::History(Request::Range { history, base, end }),
        )
    }
    pub(crate) fn refresh_history_at(
        &mut self,
        history: Arc<History>,
        observation: Arc<Observation>,
        preserve_root: bool,
    ) -> Result<OperationId, String> {
        self.check_history(&history)?;
        if history.observation.context != observation.context
            || history.observation.identity != observation.identity
        {
            return Err("Retained history belongs to another repository".into());
        }
        let directory = self.history_directory(&observation)?;
        let object = |o: &Observation| {
            o.status.as_ref().and_then(|s| match s.head {
                Head::Branch { object, .. } | Head::Detached { object } => Some(object),
                Head::Unborn { .. } => None,
            })
        };
        let prior = (preserve_root || object(&history.observation) == object(&observation))
            .then_some(history);
        self.start(
            directory,
            Some(observation.context),
            work::Request::History(Request::Log(Ancestry {
                observation,
                root: None,
                prior,
                next: false,
            })),
        )
    }
    pub(super) fn check_history(&self, history: &Arc<History>) -> Result<(), String> {
        if self
            .context(history.observation.context)
            .is_none_or(|c| c.history.as_ref().is_none_or(|h| !Arc::ptr_eq(h, history)))
        {
            return Err("History requires its current native observation owner".into());
        }
        Ok(())
    }
    pub(super) fn history_directory(
        &self,
        observation: &Arc<Observation>,
    ) -> Result<PathBuf, String> {
        let context = self
            .context(observation.context)
            .ok_or("Git context no longer exists")?;
        if context.failure.is_some() || !Arc::ptr_eq(&context.observation, observation) {
            return Err("History requires a current confirmed repository observation".into());
        }
        if self
            .progress()
            .any(|p| p.context.is_none() || p.context == Some(context.id))
        {
            return Err("Git repository already owns an active operation".into());
        }
        Ok(observation
            .identity
            .worktree
            .as_ref()
            .unwrap_or(&observation.identity.git_directory)
            .clone())
    }
}

pub(super) struct PageResult {
    pub revisions: Vec<Revision>,
    pub more: bool,
}

impl History {
    /// An unpublished ancestry result owns its exact root and admission.
    pub(in crate::git) fn new(
        id: OperationId,
        observation: Arc<Observation>,
        root: Option<ObjectId>,
        admission: Lease,
    ) -> Self {
        Self {
            id,
            observation,
            root,
            pages: Vec::new(),
            more: false,
            page_limit: false,
            _admission: admission,
        }
    }
    pub(in crate::git) fn with_page(
        mut self,
        prior: Option<&Self>,
        page: PageResult,
        max_pages: usize,
    ) -> Self {
        let PageResult { revisions, more } = page;
        self.pages = prior.map_or_else(Vec::new, |h| h.pages.clone());
        if !revisions.is_empty() {
            let bytes = std::mem::size_of::<HistoryPage>()
                + revisions.capacity() * std::mem::size_of::<Revision>()
                + revisions
                    .iter()
                    .map(|r| r.bytes() - std::mem::size_of::<Revision>())
                    .sum::<usize>();
            let page = HistoryPage {
                revisions,
                _admission: self._admission.split(bytes),
            };
            self.pages.push(Arc::new(page));
        }
        self.more = more;
        self.page_limit = more && self.pages.len() >= max_pages;
        self._admission.retain(
            std::mem::size_of::<History>()
                + self.pages.capacity() * std::mem::size_of::<Arc<HistoryPage>>(),
        );
        self
    }
}
