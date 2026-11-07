use std::collections::VecDeque;
use std::path::PathBuf;

use crate::buffer::{BufferId, Stamp};
use crate::context::{
    self, Catalog, Context, FileEntry, FileReference, Generation, Project, SavedSession, Snapshot,
    SourcePosition, Store, Viewport,
};

use super::super::{Action, Session, quit::Goal};
use super::projects::{Operation, Output, identity_matches};

pub(super) struct After {
    generation: Generation,
    stamps: Vec<Stamp>,
    promotion: Option<Project>,
    goal: Option<Goal>,
    roots: Vec<(PathBuf, (u64, u64))>,
}
pub(super) struct Capture {
    after: After,
    catalog: Catalog,
    owner: Option<Project>,
    registration: Option<Project>,
    files: VecDeque<BufferId>,
    entries: Vec<Captured>,
    unavailable: Vec<FileEntry>,
    active: BufferId,
}
struct Captured {
    path: PathBuf,
    cursor: SourcePosition,
    viewport: Viewport,
    active: bool,
}

impl Session {
    pub(in crate::session) fn context_checkpoint_needed(&self) -> bool {
        self.context_project.is_some()
            || self
                .context_prepared
                .as_ref()
                .is_some_and(|r| r.registration.is_some())
    }
    pub(in crate::session) fn checkpoint_context(&mut self, goal: Goal) -> Action {
        self.start_context_capture(None, Some(goal), None)
    }
    pub(in crate::session) fn capture_project_promotion(
        &mut self,
        project: Project,
        identity: (u64, u64),
    ) -> Action {
        self.start_context_capture(Some(project), None, Some(identity))
    }
    fn start_context_capture(
        &mut self,
        promotion: Option<Project>,
        goal: Option<Goal>,
        promotion_identity: Option<(u64, u64)>,
    ) -> Action {
        if self.context_storage.busy() {
            return self.error("Project state work is already pending");
        }
        if let Err(error) = self.context_generation.next() {
            return self.error(&error.to_string());
        }
        let Some(catalog) = self.context_storage.catalog.clone() else {
            return self.error("Project catalog is unavailable; live sources retained");
        };
        let owner = promotion.clone().or_else(|| self.context_project.clone());
        if let Some(project) = &self.context_project
            && !catalog.projects.iter().any(|p| p == project)
        {
            self.context_prepared = None;
            return self.error("Active Project changed in storage; live sources retained");
        }
        let registration = promotion.clone().or_else(|| {
            self.context_prepared
                .as_ref()
                .and_then(|r| r.registration.clone())
        });
        let mut roots = Vec::new();
        if let Some(project) = &owner {
            let identity = promotion_identity.or(self.context_root_identity);
            let Some(identity) = identity else {
                return self.error("Project root identity is unavailable; live sources retained");
            };
            roots.push((project.root.path.as_path().to_owned(), identity));
        }
        if let Some(replacement) = &self.context_prepared {
            if replacement.expected != self.context_generation {
                return self.error("Prepared context is stale; live sources retained");
            }
            let root = match &replacement.session.context {
                Context::Directory(path) => Some(path),
                Context::Project(_) => replacement
                    .session
                    .context_project
                    .as_ref()
                    .map(|p| &p.root.path),
                _ => None,
            };
            if let (Some(path), Some(identity)) = (root, replacement.root_identity) {
                roots.push((path.as_path().to_owned(), identity));
            }
        }
        if !roots.iter().all(|(p, i)| identity_matches(p, *i)) {
            self.context_prepared = None;
            return self.error("Prepared Project root changed; live sources retained");
        }
        let files: VecDeque<_> = if owner.is_some() {
            self.buffers
                .iter()
                .filter(|b| b.file().is_some() && !b.is_transient() && b.terminal().is_none())
                .map(|b| b.id())
                .collect()
        } else {
            VecDeque::new()
        };
        if files.len() > context::MAX_FILES {
            self.context_prepared = None;
            return self.error("Project session file capacity exceeded; live sources retained");
        }
        let after = After {
            generation: self.context_generation,
            stamps: self.buffers.iter().map(|b| b.stamp()).collect(),
            promotion,
            goal,
            roots,
        };
        self.input.cancel();
        self.pending = None;
        self.context_storage.operation = Some(Operation::Capture(Box::new(Capture {
            after,
            catalog,
            owner,
            registration,
            files,
            entries: Vec::new(),
            unavailable: self.context_unavailable.clone(),
            active: self.view().point.stamp.buffer,
        })));
        self.message("Capturing Project session; live sources retained")
    }
    pub(in crate::session) fn advance_context_capture(&mut self) -> Action {
        let Some(Operation::Capture(mut capture)) = self.context_storage.operation.take() else {
            return Action::Wait;
        };
        let result = (|| -> Result<(), String> {
            for _ in 0..context::FILE_ADMISSION_STEPS {
                let Some(id) = capture.files.pop_front() else {
                    break;
                };
                let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
                let path = buffer
                    .file()
                    .ok_or("Source file association changed")?
                    .path()
                    .to_owned();
                let (cursor, viewport) = self.capture_context_position(id)?;
                capture.entries.push(Captured {
                    path,
                    cursor,
                    viewport,
                    active: capture.active == id,
                });
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.context_prepared = None;
            return self.error(&format!("Project session: {error}; live sources retained"));
        }
        if !capture.files.is_empty() {
            self.context_storage.operation = Some(Operation::Capture(capture));
            return Action::Wait;
        }
        let Some(path) = self.context_storage.path.clone() else {
            self.context_prepared = None;
            return self.error("Project storage is unavailable; live sources retained");
        };
        let Capture {
            after,
            catalog,
            owner,
            registration,
            entries,
            unavailable,
            ..
        } = *capture;
        let roots = after.roots.clone();
        let stamp = self.view().point.stamp;
        if let Err(error) = self.context_storage.job.start(stamp, move |cancel| {
            if cancel.is_cancelled() {
                return Err("Project publication canceled before effects".into());
            }
            if !roots.iter().all(|(p, i)| identity_matches(p, *i)) {
                return Err("Project root changed before publication".into());
            }
            let candidate = candidate(&catalog, owner, registration, entries, unavailable)?;
            let prepared = catalog.prepare(candidate).map_err(|e| e.to_string())?;
            if cancel.is_cancelled() {
                return Err("Project publication canceled before effects".into());
            }
            let (store, _) = Store::open(&path).map_err(|e| e.to_string())?;
            store
                .publish(&prepared)
                .map(Output::Published)
                .map_err(|e| e.to_string())
        }) {
            self.context_prepared = None;
            return self.error(&error.to_string());
        }
        self.context_storage.operation = Some(Operation::Publish(after));
        self.message("Publishing Project state; live sources retained")
    }
    pub(super) fn context_published(
        &mut self,
        after: After,
        publication: context::Publication,
    ) -> Action {
        let warning = publication.durability_warning;
        let source_current = after.generation == self.context_generation
            && self.buffers.iter().map(|b| b.stamp()).eq(after.stamps);
        self.context_storage.catalog = Some(publication.catalog);
        // Publication is the commit point. Adopt its record even when a later
        // source/root check prevents departure; never retry under a fresh ID.
        if let Some(project) = after.promotion {
            self.context = Context::Project(project.id.clone());
            self.context_root_identity = after.roots.first().map(|(_, i)| *i);
            self.context_project = Some(project);
            self.context_generation = match self.context_generation.next() {
                Ok(g) => g,
                Err(e) => return self.error(&e.to_string()),
            };
        }
        if let Some(active) = &self.context_project {
            self.context_project = self
                .context_storage
                .catalog
                .as_ref()
                .and_then(|c| c.projects.iter().find(|p| p.id == active.id))
                .cloned();
        }
        if !source_current {
            self.context_prepared = None;
            return self
                .error("Project state published; source changed and departure was canceled");
        }
        let action = if let Some(goal) = after.goal {
            if !after.roots.iter().all(|(p, i)| identity_matches(p, *i)) {
                self.context_prepared = None;
                self.error("Project state published; root changed and departure was canceled")
            } else {
                self.finish_context_departure(goal)
            }
        } else {
            self.message("Project registered; live buffers, positions and unsaved text retained")
        };
        if matches!(after.goal, Some(Goal::Context(_)))
            && self.context_generation == after.generation
            && !self.echo.starts_with("Project state published")
        {
            let message = format!("Project state published; {}", self.echo);
            self.error(&message);
        }
        if let Some(warning) = warning {
            if action == Action::Exit {
                eprintln!("{warning}");
            } else {
                let message = format!("{}; {warning}", self.echo);
                self.notice(&message);
            }
        }
        action
    }
}
fn candidate(
    catalog: &Catalog,
    owner: Option<Project>,
    registration: Option<Project>,
    entries: Vec<Captured>,
    unavailable: Vec<FileEntry>,
) -> Result<Catalog, String> {
    let mut candidate = catalog.clone();
    let mut snapshot = Snapshot::default();
    if let Some(owner) = &owner {
        let roots = std::slice::from_ref(&owner.root);
        for entry in entries {
            let reference = capture_reference(&entry.path, roots)?;
            if snapshot.files.iter().any(|e| e.file == reference) {
                continue;
            }
            if entry.active {
                snapshot.active_file = Some(reference.clone());
            }
            snapshot.files.push(FileEntry {
                file: reference,
                cursor: entry.cursor,
                viewport: Some(entry.viewport),
            });
        }
        for entry in unavailable {
            if !snapshot.files.iter().any(|e| e.file == entry.file) {
                snapshot.files.push(entry);
            }
        }
        if snapshot.files.len() > context::MAX_FILES {
            return Err("Project session file capacity exceeded".into());
        }
        if let Some(index) = candidate.projects.iter().position(|p| p.id == owner.id) {
            if candidate.projects[index] != *owner {
                return Err("Active Project revision changed".into());
            }
            let mut next = owner.clone();
            next.revision = next.revision.next().map_err(|e| e.to_string())?;
            next.session_ref.revision = next
                .session_ref
                .revision
                .next()
                .map_err(|e| e.to_string())?;
            let index_session = candidate
                .sessions
                .iter()
                .position(|s| s.reference == owner.session_ref)
                .ok_or("Active Project session disappeared")?;
            candidate.sessions[index_session] = SavedSession::new(
                next.session_ref.clone(),
                next.roots_revision,
                snapshot.clone(),
            );
            candidate.projects[index] = next;
        }
    }
    if let Some(project) = registration {
        if candidate.projects.iter().any(|p| p.id == project.id) {
            return Err("Project identity is already registered".into());
        }
        let initial = if owner.as_ref().is_some_and(|o| o.id == project.id) {
            snapshot
        } else {
            Snapshot::default()
        };
        candidate.sessions.push(SavedSession::new(
            project.session_ref.clone(),
            project.roots_revision,
            initial,
        ));
        candidate.projects.push(project);
    }
    Ok(candidate)
}
fn capture_reference(
    path: &std::path::Path,
    roots: &[context::Root],
) -> Result<FileReference, String> {
    if path.exists() {
        return FileReference::capture(path, roots).map_err(|e| e.to_string());
    }
    // A formerly loaded file may disappear. Preserve its reference without
    // manufacturing a source buffer or weakening symlink-prefix validation.
    let reference = match roots
        .iter()
        .filter_map(|root| {
            path.strip_prefix(root.path.as_path())
                .ok()
                .map(|p| (root, p))
        })
        .max_by_key(|(root, _)| root.path.as_path().components().count())
    {
        Some((root, relative)) => FileReference::RootRelative {
            root: root.id.clone(),
            path: context::NativePath::relative(relative).map_err(|e| e.to_string())?,
        },
        None => FileReference::LooseAbsolute(
            context::NativePath::absolute(path).map_err(|e| e.to_string())?,
        ),
    };
    reference.resolve(roots).map_err(|e| e.to_string())?;
    Ok(reference)
}
