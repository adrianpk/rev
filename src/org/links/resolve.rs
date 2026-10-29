use super::*;
use crate::{
    buffer::Buffers,
    org::{
        agenda::{SourceScan, SourceSelection},
        io::{CaptureRead, DestinationVersion},
    },
};
use std::path::{Component, Path};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Follow {
    Internal(String),
    Local {
        path: PathBuf,
        search: String,
        mode: FileMode,
        root: Option<PathBuf>,
    },
    Id {
        id: String,
        search: String,
    },
    External(String),
}
pub(crate) enum Resolve {
    Selected(crate::org::attachment::Selected, Box<Self>),
    Sources {
        scan: Box<SourceScan>,
        search: String,
        id: Option<String>,
        policy: crate::config::Org,
    },
    File {
        read: Box<CaptureRead>,
        search: String,
        root: Option<PathBuf>,
        policy: crate::config::Org,
    },
}
impl Document {
    pub(super) fn follow(
        &self,
        target: Target,
        point: usize,
        directory: &Path,
        policy: &crate::config::Org,
    ) -> Result<Follow, String> {
        match target {
            Target::Internal(value) => Ok(Follow::Internal(value)),
            Target::Id { id, search } => Ok(Follow::Id { id, search }),
            Target::External(value) => Ok(Follow::External(value)),
            Target::File { path, search, mode } => Ok(Follow::Local {
                path: local(directory, path)?,
                search,
                mode,
                root: None,
            }),
            Target::Attachment { path, search } => {
                if path.is_absolute()
                    || path
                        .components()
                        .any(|p| !matches!(p, Component::Normal(_)))
                {
                    return Err("Attachment link must stay inside its selected directory".into());
                }
                let root = self.attachment(point, directory, policy)?;
                Ok(Follow::Local {
                    path: root.join(path),
                    search,
                    mode: FileMode::Automatic,
                    root: Some(root),
                })
            }
        }
    }
    fn attachment(
        &self,
        point: usize,
        directory: &Path,
        policy: &crate::config::Org,
    ) -> Result<PathBuf, String> {
        let dir =
            self.properties
                .query_value(&self.text, point, "DIR", &policy.properties.inherit)?;
        let id =
            self.properties
                .query_value(&self.text, point, "ID", &policy.properties.inherit)?;
        crate::org::attachment::location(dir.as_deref(), id.as_deref(), directory, policy)
    }
}
fn local(directory: &Path, path: PathBuf) -> Result<PathBuf, String> {
    let value = path.to_str().ok_or("Link path is not UTF-8")?;
    crate::config::capture::path(value)?;
    if value.starts_with("//") || value.starts_with("/ssh:") || value.starts_with('~') {
        return Err("Remote and home-expanded link paths are not admitted".into());
    }
    Ok(if path.is_relative() {
        directory.join(path)
    } else {
        path
    })
}
impl Resolve {
    pub(crate) fn new(
        buffers: &Buffers,
        origin: Stamp,
        target: Follow,
        selection: SourceSelection,
    ) -> Result<Self, String> {
        let policy = buffers.snapshot().settings().org.clone();
        match target {
            Follow::Internal(search) => Ok(Self::Sources {
                scan: Box::new(SourceScan::new(
                    buffers,
                    SourceSelection {
                        paths: Vec::new(),
                        buffers: vec![origin.buffer],
                    },
                    &policy.limits,
                )?),
                search,
                id: None,
                policy,
            }),
            Follow::Id { id, search } => Ok(Self::Sources {
                scan: Box::new(SourceScan::new(buffers, selection, &policy.limits)?),
                search,
                id: Some(id),
                policy,
            }),
            Follow::Local {
                path,
                search,
                mode: _,
                root,
            } => Ok(Self::File {
                read: Box::new(CaptureRead::new(buffers, &path, &policy.limits)?),
                search,
                root,
                policy,
            }),
            Follow::External(uri) => Err(format!(
                "No external viewer selected for {uri}; configure org.tools.viewer"
            )),
        }
    }
    pub(crate) fn selected(self, attachment: crate::org::attachment::Selected) -> Self {
        Self::Selected(attachment, Box::new(self))
    }
    pub(crate) fn run(self, cancel: &Cancellation) -> Result<Navigation, String> {
        match self {
            Self::Selected(attachment, work) => selected_attachment(attachment, *work, cancel),
            Self::Sources {
                scan,
                search,
                id,
                policy,
            } => sources(*scan, search, id, &policy, cancel),
            Self::File {
                read,
                search,
                root,
                policy,
            } => {
                let source = read.run(cancel)?;
                if matches!(source.version, DestinationVersion::Missing) {
                    return Err("Link destination does not exist".into());
                }
                contained(&source.path, root.as_deref())?;
                let doc = Document::observe(source.snapshot(), &policy, cancel)?;
                let byte = doc.resolve(&search, 0..source.text().len_bytes())?;
                Ok(Navigation {
                    destination: crate::org::refile::Destination::File(source),
                    byte,
                })
            }
        }
    }
}
fn sources(
    scan: SourceScan,
    search: String,
    id: Option<String>,
    policy: &crate::config::Org,
    cancel: &Cancellation,
) -> Result<Navigation, String> {
    let sources = scan.run(cancel);
    if !sources.complete() {
        return Err(sources.feedback());
    }
    let mut matches = Vec::new();
    for source in sources.documents {
        if cancel.is_cancelled() {
            return Err("Link resolution cancelled".into());
        }
        let doc = Document::observe(source.snapshot(), policy, cancel)?;
        let scope = if let Some(id) = &id {
            let Some(scope) = doc.id_scope(id)? else {
                continue;
            };
            scope
        } else {
            0..source.text().len_bytes()
        };
        let byte = doc.resolve(&search, scope)?;
        matches.push(Navigation {
            destination: crate::org::refile::Destination::Agenda(source),
            byte,
        });
    }
    if matches.len() != 1 {
        return Err("Unresolved or ambiguous link destination in selected sources".into());
    }
    Ok(matches.pop().expect("one checked destination"))
}
pub(super) fn contained(path: &Path, root: Option<&Path>) -> Result<(), String> {
    if let Some(root) = root {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        if !path.starts_with(root) {
            return Err("Attachment link escapes its canonical directory".into());
        }
    }
    Ok(())
}

fn selected_attachment(
    attachment: crate::org::attachment::Selected,
    work: Resolve,
    cancel: &Cancellation,
) -> Result<Navigation, String> {
    attachment.check()?;
    let result = work.run(cancel)?;
    attachment.check()?;
    Ok(result)
}
