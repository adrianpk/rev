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
    File {
        observation: Arc<Observation>,
        object: ObjectId,
        path: PathBuf,
        line: usize,
        current_path: Option<PathBuf>,
    },
    Range {
        history: Arc<History>,
        base: ObjectId,
        end: ObjectId,
    },
    Log {
        observation: Arc<Observation>,
        root: Option<String>,
        prior: Option<Arc<History>>,
        next: bool,
    },
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
            Self::Log { observation, .. }
            | Self::Reflog(observation)
            | Self::Blame { observation, .. }
            | Self::ShowObject { observation, .. }
            | Self::File { observation, .. } => observation,
            Self::Show { history, .. } | Self::Range { history, .. } => &history.observation,
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
        let observation = self
            .context(context)
            .ok_or("Git context no longer exists")?
            .observation
            .clone();
        let directory = self.history_directory(&observation)?;
        self.start(
            directory,
            Some(context),
            work::Request::History(Request::Log {
                observation,
                root: root.map(str::to_owned),
                prior: None,
                next: false,
            }),
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
            work::Request::History(Request::Log {
                observation: history.observation.clone(),
                root: None,
                prior: Some(history),
                next,
            }),
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
    fn check_history(&self, history: &Arc<History>) -> Result<(), String> {
        if self
            .context(history.observation.context)
            .is_none_or(|c| c.history.as_ref().is_none_or(|h| !Arc::ptr_eq(h, history)))
        {
            return Err("History requires its current native observation owner".into());
        }
        Ok(())
    }
    fn history_directory(&self, observation: &Arc<Observation>) -> Result<PathBuf, String> {
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

pub(super) fn log_result(
    id: OperationId,
    observation: Arc<Observation>,
    root: Option<ObjectId>,
    prior: Option<Arc<History>>,
    page: PageResult,
    max_pages: usize,
    mut admission: Lease,
) -> History {
    let PageResult { revisions, more } = page;
    let mut pages = prior.as_ref().map_or_else(Vec::new, |h| h.pages.clone());
    if !revisions.is_empty() {
        let bytes = std::mem::size_of::<HistoryPage>()
            + revisions.capacity() * std::mem::size_of::<Revision>()
            + revisions
                .iter()
                .map(|r| r.bytes() - std::mem::size_of::<Revision>())
                .sum::<usize>();
        let page = HistoryPage {
            revisions,
            _admission: admission.split(bytes),
        };
        pages.push(Arc::new(page));
    }
    let page_limit = more && pages.len() >= max_pages;
    admission.retain(
        std::mem::size_of::<History>() + pages.capacity() * std::mem::size_of::<Arc<HistoryPage>>(),
    );
    History {
        id,
        observation,
        root,
        pages,
        more,
        page_limit,
        _admission: admission,
    }
}
