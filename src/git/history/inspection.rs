use super::*;

#[derive(Debug)]
pub struct ReflogEntry {
    pub object: ObjectId,
    /// Volatile selector is display metadata, never a revision argument.
    pub selector: Vec<u8>,
    pub subject: Vec<u8>,
    pub author: Vec<u8>,
    pub email: Vec<u8>,
}
#[derive(Debug)]
pub struct Reflog {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub entries: Vec<ReflogEntry>,
    pub completeness: Completeness,
    pub(in crate::git) _admission: Lease,
}
#[derive(Debug)]
pub struct Attribution {
    pub object: ObjectId,
    pub path: PathBuf,
    pub original_line: usize,
    pub final_line: usize,
    pub author: Vec<u8>,
    pub email: Vec<u8>,
    pub time: Vec<u8>,
    pub subject: Vec<u8>,
    pub text: Vec<u8>,
}
#[derive(Debug)]
pub struct Blame {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub root: ObjectId,
    pub path: PathBuf,
    pub lines: Vec<Attribution>,
    pub completeness: Completeness,
    pub(in crate::git) _admission: Lease,
}
#[derive(Debug)]
pub struct RevisionFile {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub object: ObjectId,
    pub path: PathBuf,
    pub current_path: Option<PathBuf>,
    pub bytes: Vec<u8>,
    pub lines: Vec<std::ops::Range<usize>>,
    pub completeness: Completeness,
    pub binary: bool,
    pub original_line: usize,
    pub(in crate::git) _admission: Lease,
}

impl Repositories {
    pub fn reflog(&mut self, context: ContextId) -> Result<OperationId, String> {
        let observation = self
            .context(context)
            .ok_or("Git context no longer exists")?
            .observation
            .clone();
        let directory = self.history_directory(&observation)?;
        self.start(
            directory,
            Some(context),
            work::Request::History(Request::Reflog(observation)),
        )
    }
    pub fn blame(&mut self, context: ContextId, path: &Path) -> Result<OperationId, String> {
        let observation = self
            .context(context)
            .ok_or("Git context no longer exists")?
            .observation
            .clone();
        let directory = self.history_directory(&observation)?;
        validate_path(path.as_os_str().as_encoded_bytes())?;
        if observation.identity.worktree.is_none() {
            return Err("Blame requires a working-tree source context".into());
        }
        self.start(
            directory,
            Some(context),
            work::Request::History(Request::Blame {
                observation,
                path: path.to_owned(),
            }),
        )
    }
    pub fn show_reflog(
        &mut self,
        reflog: Arc<Reflog>,
        object: ObjectId,
    ) -> Result<OperationId, String> {
        let context = self
            .context(reflog.observation.context)
            .ok_or("Git context no longer exists")?;
        if context
            .reflog
            .as_ref()
            .is_none_or(|r| !Arc::ptr_eq(r, &reflog))
            || !reflog.entries.iter().any(|e| e.object == object)
        {
            return Err("Reflog inspection requires an exact admitted entry owner".into());
        }
        let directory = self.history_directory(&reflog.observation)?;
        self.start(
            directory,
            Some(context.id),
            work::Request::History(Request::ShowObject {
                observation: reflog.observation.clone(),
                object,
            }),
        )
    }
    pub fn inspect_blame(
        &mut self,
        blame: Arc<Blame>,
        final_line: usize,
        file: bool,
    ) -> Result<OperationId, String> {
        let context = self
            .context(blame.observation.context)
            .ok_or("Git context no longer exists")?;
        if context
            .blame
            .as_ref()
            .is_none_or(|b| !Arc::ptr_eq(b, &blame))
        {
            return Err("Blame requires its exact admitted source owner".into());
        }
        let line = blame
            .lines
            .iter()
            .find(|l| l.final_line == final_line)
            .ok_or("Blame line is not admitted")?;
        let directory = self.history_directory(&blame.observation)?;
        let request = if file {
            Request::File {
                observation: blame.observation.clone(),
                object: line.object,
                path: line.path.clone(),
                line: line.original_line,
                current_path: Some(blame.path.clone()),
            }
        } else {
            Request::ShowObject {
                observation: blame.observation.clone(),
                object: line.object,
            }
        };
        self.start(directory, Some(context.id), work::Request::History(request))
    }
    pub fn compare_parent(
        &mut self,
        detail: Arc<RevisionDetail>,
        parent: ObjectId,
    ) -> Result<OperationId, String> {
        let context = self
            .context(detail.history.observation.context)
            .ok_or("Git context no longer exists")?;
        if context
            .revision
            .as_ref()
            .is_none_or(|d| !Arc::ptr_eq(d, &detail))
            || detail
                .history
                .revision(detail.object)
                .is_none_or(|r| !r.parents.contains(&parent))
        {
            return Err("Parent comparison requires an exact admitted parent/detail owner".into());
        }
        let directory = self.history_directory(&detail.history.observation)?;
        self.start(
            directory,
            Some(context.id),
            work::Request::History(Request::Show {
                history: detail.history.clone(),
                object: detail.object,
                parent: Some(parent),
            }),
        )
    }
    pub fn refresh_revision(&mut self, detail: Arc<RevisionDetail>) -> Result<OperationId, String> {
        let context = self
            .context(detail.history.observation.context)
            .ok_or("Git context no longer exists")?;
        if context
            .revision
            .as_ref()
            .is_none_or(|d| !Arc::ptr_eq(d, &detail))
        {
            return Err("Revision refresh requires its exact native detail owner".into());
        }
        let directory = self.history_directory(&detail.history.observation)?;
        let request = if detail.range {
            Request::Range {
                history: detail.history.clone(),
                base: detail.parent.ok_or("Range lacks its exact base")?,
                end: detail.object,
            }
        } else {
            Request::Show {
                history: detail.history.clone(),
                object: detail.object,
                parent: detail.parent,
            }
        };
        self.start(directory, Some(context.id), work::Request::History(request))
    }
    pub fn refresh_revision_file(
        &mut self,
        file: Arc<RevisionFile>,
    ) -> Result<OperationId, String> {
        let context = self
            .context(file.observation.context)
            .ok_or("Git context no longer exists")?;
        if context
            .revision_file
            .as_ref()
            .is_none_or(|f| !Arc::ptr_eq(f, &file))
        {
            return Err("File refresh requires its exact native detail owner".into());
        }
        let directory = self.history_directory(&file.observation)?;
        self.start(
            directory,
            Some(context.id),
            work::Request::History(Request::File {
                observation: file.observation.clone(),
                object: file.object,
                path: file.path.clone(),
                line: file.original_line,
                current_path: file.current_path.clone(),
            }),
        )
    }
    /// Historical paths come from an immutable admitted comparison, including
    /// the old-side path/object when that comparison records a deletion.
    pub fn revision_file(
        &mut self,
        detail: Arc<RevisionDetail>,
        path: &Path,
    ) -> Result<OperationId, String> {
        let context = self
            .context(detail.history.observation.context)
            .ok_or("Git context no longer exists")?;
        if context
            .revision
            .as_ref()
            .is_none_or(|d| !Arc::ptr_eq(d, &detail))
        {
            return Err("Historical file requires its exact detail owner".into());
        }
        let selected = detail
            .paths
            .iter()
            .find(|p| p.path == path)
            .ok_or("Historical path is not admitted")?;
        let object = if selected.status.first() == Some(&b'D') {
            detail.parent.ok_or("Deleted path has no old-side object")?
        } else {
            detail.object
        };
        let directory = self.history_directory(&detail.history.observation)?;
        self.start(
            directory,
            Some(context.id),
            work::Request::History(Request::File {
                observation: detail.history.observation.clone(),
                object,
                path: selected.path.clone(),
                line: 1,
                current_path: None,
            }),
        )
    }
}

pub(super) fn validate_path(path: &[u8]) -> Result<(), String> {
    if path.is_empty()
        || path.len() > 4096
        || path.contains(&0)
        || path[0] == b'/'
        || path
            .split(|b| *b == b'/')
            .any(|p| p == b".." || p == b"." || p.is_empty())
    {
        return Err("Historical path exceeds exact relative-path admission".into());
    }
    Ok(())
}
