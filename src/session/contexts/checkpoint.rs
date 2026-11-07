use std::collections::VecDeque;
use std::path::PathBuf;

use crate::buffer::{BufferId, Stamp};
use crate::context::{
    self, Catalog, Context, FileEntry, FileReference, Generation, Project, Snapshot,
    SourcePosition, Store, Viewport, Workspace,
};

use super::super::{Action, Session, quit::Goal};
use super::projects::{Operation, Output, identity_matches};
use super::record::Record;
use super::workspaces::{Membership, observe};

pub(super) struct After {
    generation: Generation,
    stamps: Vec<Stamp>,
    promotion: Option<Project>,
    goal: Option<Goal>,
    roots: Vec<(PathBuf, (u64, u64))>,
    membership: Option<Box<Membership>>,
}
pub(super) struct Capture {
    after: After,
    catalog: Catalog,
    owner: Option<Record>,
    registration: Option<Record>,
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
        self.context_record.is_some()
            || self
                .context_prepared
                .as_ref()
                .is_some_and(|r| r.registration.is_some())
    }
    pub(in crate::session) fn checkpoint_context(&mut self, goal: Goal) -> Action {
        self.start_context_capture(None, Some(goal), None, None)
    }
    pub(in crate::session) fn capture_project_promotion(
        &mut self,
        project: Project,
        identity: (u64, u64),
    ) -> Action {
        self.start_context_capture(Some(project), None, Some(identity), None)
    }
    pub(super) fn capture_membership(&mut self, membership: Box<Membership>) -> Action {
        self.start_context_capture(None, None, None, Some(membership))
    }
    fn start_context_capture(
        &mut self,
        promotion: Option<Project>,
        goal: Option<Goal>,
        promotion_identity: Option<(u64, u64)>,
        mut membership: Option<Box<Membership>>,
    ) -> Action {
        if self.context_storage.busy() {
            return self.error("Context state work is already pending");
        }
        if let Err(e) = self.context_generation.next() {
            return self.error(&e.to_string());
        }
        let Some(catalog) = self.context_storage.catalog.clone() else {
            return self.error("Context catalog is unavailable; live sources retained");
        };
        let owner = promotion
            .clone()
            .map(Record::Project)
            .or_else(|| self.context_record.clone());
        if self
            .context_record
            .as_ref()
            .is_some_and(|r| !r.matches(&catalog))
        {
            self.context_prepared = None;
            return self.error("Active context changed in storage; live sources retained");
        }
        let registration = promotion.clone().map(Record::Project).or_else(|| {
            self.context_prepared
                .as_ref()
                .and_then(|r| r.registration.clone())
        });
        let mut roots = Vec::new();
        if let Some(owner) = &owner {
            match owner {
                Record::Project(project) => {
                    let Some(identity) = promotion_identity.or(self.context_root_identity) else {
                        return self
                            .error("Project root identity is unavailable; live sources retained");
                    };
                    roots.push((project.root.path.as_path().to_owned(), identity));
                }
                Record::Workspace(workspace) => {
                    for root in &workspace.roots {
                        if let Some(identity) = observe(root.path.as_path()) {
                            roots.push((root.path.as_path().to_owned(), identity));
                        }
                    }
                }
            }
        }
        if let Some(replacement) = &self.context_prepared {
            if replacement.expected != self.context_generation {
                return self.error("Prepared context is stale; live sources retained");
            }
            let root = match &replacement.session.context {
                Context::Directory(path) => Some(path),
                Context::Project(_) => replacement
                    .session
                    .context_record
                    .as_ref()
                    .and_then(|r| r.roots().first())
                    .map(|r| &r.path),
                _ => None,
            };
            if let (Some(path), Some(identity)) = (root, replacement.root_identity) {
                roots.push((path.as_path().to_owned(), identity));
            }
            roots.extend(
                replacement
                    .workspace_observations
                    .iter()
                    .filter_map(|(path, identity)| {
                        identity.map(|i| (path.as_path().to_owned(), i))
                    }),
            );
        }
        if let Some(membership) = &mut membership {
            roots.extend(membership.observations.clone());
            membership.closing = match self.buffers.prepare_closing(&membership.closed) {
                Ok(c) => Some(c),
                Err(e) => return self.error(&e.to_string()),
            };
        }
        if !roots.iter().all(|(p, i)| identity_matches(p, *i)) {
            self.context_prepared = None;
            return self.error("Prepared context root changed; live sources retained");
        }
        let files: VecDeque<_> = if owner.is_some() {
            self.buffers
                .iter()
                .filter(|b| {
                    b.file().is_some()
                        && !b.is_transient()
                        && b.terminal().is_none()
                        && !membership
                            .as_ref()
                            .is_some_and(|m| m.closed.contains(&b.id()))
                })
                .map(|b| b.id())
                .collect()
        } else {
            VecDeque::new()
        };
        if files.len() > context::MAX_FILES {
            self.context_prepared = None;
            return self.error("Context session file capacity exceeded; live sources retained");
        }
        let after = After {
            generation: self.context_generation,
            stamps: self.buffers.iter().map(|b| b.stamp()).collect(),
            promotion,
            goal,
            roots,
            membership,
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
        self.message("Capturing context session; live sources retained")
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
        if let Err(e) = result {
            self.context_prepared = None;
            return self.error(&format!("Context session: {e}; live sources retained"));
        }
        if !capture.files.is_empty() {
            self.context_storage.operation = Some(Operation::Capture(capture));
            return Action::Wait;
        }
        let Some(path) = self.context_storage.path.clone() else {
            self.context_prepared = None;
            return self.error("Context storage is unavailable; live sources retained");
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
        let membership = after.membership.as_ref().map(|m| m.next.clone());
        if let Err(e) = self
            .context_storage
            .job
            .start(self.view().point.stamp, move |cancel| {
                if cancel.is_cancelled() {
                    return Err("Context publication canceled before effects".into());
                }
                if !roots.iter().all(|(p, i)| identity_matches(p, *i)) {
                    return Err("Context root changed before publication".into());
                }
                let candidate = candidate(
                    &catalog,
                    owner,
                    registration,
                    entries,
                    unavailable,
                    membership,
                )?;
                let prepared = catalog.prepare(candidate).map_err(|e| e.to_string())?;
                if cancel.is_cancelled() {
                    return Err("Context publication canceled before effects".into());
                }
                let (store, _) = Store::open(&path).map_err(|e| e.to_string())?;
                store
                    .publish(&prepared)
                    .map(Output::Published)
                    .map_err(|e| e.to_string())
            })
        {
            self.context_prepared = None;
            return self.error(&e.to_string());
        }
        self.context_storage.operation = Some(Operation::Publish(after));
        self.message("Publishing context state; live sources retained")
    }
    pub(super) fn context_published(
        &mut self,
        after: After,
        publication: context::Publication,
    ) -> Action {
        let warning = publication.durability_warning;
        let current = after.generation == self.context_generation
            && self.buffers.iter().map(|b| b.stamp()).eq(after.stamps);
        self.context_storage.catalog = Some(publication.catalog);
        if let Some(project) = after.promotion {
            self.context = Context::Project(project.id.clone());
            self.context_root_identity = after.roots.first().map(|(_, i)| *i);
            self.context_record = Some(Record::Project(project));
            self.context_generation = match self.context_generation.next() {
                Ok(g) => g,
                Err(e) => return self.error(&e.to_string()),
            };
        }
        if let (Some(active), Some(catalog)) = (&self.context_record, &self.context_storage.catalog)
        {
            self.context_record = active.current(catalog);
        }
        if !current {
            self.context_prepared = None;
            if let Some(Record::Workspace(w)) = &self.context_record {
                self.tree.workspace_roots(&w.roots);
            }
            return self
                .error("Context state published; source changed and departure was canceled");
        }
        let action = if let Some(membership) = after.membership {
            self.publish_membership(*membership)
        } else if let Some(goal) = after.goal {
            if !after.roots.iter().all(|(p, i)| identity_matches(p, *i)) {
                self.context_prepared = None;
                self.error("Context state published; root changed and departure was canceled")
            } else {
                self.finish_context_departure(goal)
            }
        } else {
            self.message("Project registered; live buffers, positions and unsaved text retained")
        };
        if matches!(after.goal, Some(Goal::Context(_)))
            && self.context_generation == after.generation
            && !self.echo.contains("state published")
        {
            let message = format!("Context state published; {}", self.echo);
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
    owner: Option<Record>,
    registration: Option<Record>,
    entries: Vec<Captured>,
    unavailable: Vec<FileEntry>,
    membership: Option<Workspace>,
) -> Result<Catalog, String> {
    let mut candidate = catalog.clone();
    let mut snapshot = Snapshot::default();
    if let Some(owner) = &owner {
        let roots = membership
            .as_ref()
            .map_or_else(|| owner.roots(), |m| m.roots.as_slice());
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
        for mut entry in unavailable {
            if membership.is_some()
                && let FileReference::RootRelative { root, .. } = &entry.file
                && !roots.iter().any(|r| r.id == *root)
            {
                let path = entry
                    .file
                    .resolve(owner.roots())
                    .map_err(|e| e.to_string())?;
                entry.file = capture_reference(&path, roots)?;
                if matches!(entry.file, FileReference::LooseAbsolute(_)) {
                    continue;
                }
            }
            if !snapshot.files.iter().any(|e| e.file == entry.file) {
                snapshot.files.push(entry);
            }
        }
        if snapshot.files.len() > context::MAX_FILES {
            return Err("Context session file capacity exceeded".into());
        }
        if owner.current(&candidate).is_some() {
            owner.checkpoint(&mut candidate, snapshot.clone(), membership.as_ref())?;
        }
    }
    if let Some(registration) = registration {
        let initial = if owner
            .as_ref()
            .is_some_and(|o| o.context() == registration.context())
        {
            snapshot
        } else {
            Snapshot::default()
        };
        registration.insert(&mut candidate, initial)?;
    }
    Ok(candidate)
}
pub(super) fn capture_reference(
    path: &std::path::Path,
    roots: &[context::Root],
) -> Result<FileReference, String> {
    if path.exists() {
        return FileReference::capture(path, roots).map_err(|e| e.to_string());
    }
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
