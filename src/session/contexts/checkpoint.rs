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

type RootObservations = Vec<(PathBuf, (u64, u64))>;

pub(super) struct After {
    generation: Generation,
    stamps: Vec<Stamp>,
    promotion: Option<Project>,
    goal: Option<Goal>,
    roots: RootObservations,
    membership: Option<Box<Membership>>,
    management: bool,
}
pub(super) struct Capture {
    after: After,
    catalog: Catalog,
    snapshot: CapturedSession,
    files: VecDeque<BufferId>,
    active: BufferId,
}
struct CapturedSession {
    owner: Option<Record>,
    registration: Option<Record>,
    entries: Vec<Captured>,
    unavailable: Vec<FileEntry>,
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
    pub(in crate::session) fn capture_management(&mut self) -> Action {
        self.start_context_capture(None, None, None, None)
    }
    pub(super) fn management_captured(&mut self, after: After, candidate: &Catalog) -> Action {
        if !after.management
            || after.generation != self.context_generation
            || !self.buffers.iter().map(|b| b.stamp()).eq(after.stamps)
        {
            self.context_management = None;
            return self.error("Context source changed before publication; live work retained");
        }
        self.management_candidate(candidate)
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
        let management = self.management_change();
        let owner = promotion
            .clone()
            .map(Record::Project)
            .or_else(|| self.context_record.clone())
            .filter(|r| !management.as_ref().is_some_and(|m| m.forgets(&r.context())));
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
        let mut roots = match self.capture_owner_roots(owner.as_ref(), promotion_identity) {
            Ok(roots) => roots,
            Err(error) => return self.error(&error),
        };
        if let Err(error) = self.capture_prepared_roots(&mut roots) {
            return self.error(&error);
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
        let files = self.capture_files(owner.is_some(), membership.as_deref());
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
            management: management.is_some(),
        };
        self.input.cancel();
        self.pending = None;
        self.context_storage.operation = Some(Operation::Capture(Box::new(Capture {
            after,
            catalog,
            snapshot: CapturedSession {
                owner,
                registration,
                entries: Vec::new(),
                unavailable: self.context_unavailable.clone(),
            },
            files,
            active: self.view().point.stamp.buffer,
        })));
        self.message("Capturing context session; live sources retained")
    }
    fn capture_files(
        &self,
        has_owner: bool,
        membership: Option<&Membership>,
    ) -> VecDeque<BufferId> {
        if has_owner {
            self.buffers
                .iter()
                .filter(|b| {
                    b.file().is_some()
                        && !b.is_transient()
                        && b.terminal().is_none()
                        && !membership.is_some_and(|m| m.closed.contains(&b.id()))
                })
                .map(|b| b.id())
                .collect()
        } else {
            VecDeque::new()
        }
    }

    fn capture_prepared_roots(&self, roots: &mut RootObservations) -> Result<(), String> {
        if let Some(replacement) = &self.context_prepared {
            if replacement.expected != self.context_generation {
                return Err("Prepared context is stale; live sources retained".into());
            }
            let root = replacement.root();
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
        Ok(())
    }

    fn capture_owner_roots(
        &self,
        owner: Option<&Record>,
        promotion_identity: Option<(u64, u64)>,
    ) -> Result<RootObservations, String> {
        let mut roots = Vec::new();
        let Some(owner) = owner else {
            return Ok(roots);
        };
        match owner {
            Record::Project(project) if self.management_relocating() => {
                if let Some(identity) = observe(project.root.path.as_path()) {
                    roots.push((project.root.path.as_path().to_owned(), identity));
                }
            }
            Record::Project(project) => {
                let Some(identity) = promotion_identity.or(self.context_root_identity) else {
                    return Err(
                        "Project root identity is unavailable; live sources retained".into(),
                    );
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
        Ok(roots)
    }

    fn capture_context_batch(&self, capture: &mut Capture) -> Result<(), String> {
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
            capture.snapshot.entries.push(Captured {
                path,
                cursor,
                viewport,
                active: capture.active == id,
            });
        }
        Ok(())
    }

    pub(in crate::session) fn advance_context_capture(&mut self) -> Action {
        let Some(Operation::Capture(mut capture)) = self.context_storage.operation.take() else {
            return Action::Wait;
        };
        let result = self.capture_context_batch(&mut capture);
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
            snapshot,
            ..
        } = *capture;
        let roots = after.roots.clone();
        let membership = after.membership.as_ref().map(|m| m.next.clone());
        let after_management = after.management;
        let management = self
            .management_change()
            .map(|change| (change, self.management_capture_source()));
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
                let candidate = snapshot.candidate(&catalog, membership.as_ref(), management)?;
                if after_management {
                    return Ok(Output::Candidate(candidate));
                }
                let prepared = catalog.prepare(candidate).map_err(|e| e.to_string())?;
                if cancel.is_cancelled() {
                    return Err("Context publication canceled before effects".into());
                }
                publish_capture(&path, &prepared)
            })
        {
            self.context_prepared = None;
            return self.error(&e.to_string());
        }
        self.context_storage.operation = Some(Operation::Publish(after));
        self.message("Publishing context state; live sources retained")
    }
    fn finish_context_capture(
        &mut self,
        membership: Option<Box<Membership>>,
        goal: Option<Goal>,
        roots: &[(PathBuf, (u64, u64))],
    ) -> Action {
        if let Some(membership) = membership {
            self.publish_membership(*membership)
        } else if let Some(goal) = goal {
            if !roots.iter().all(|(p, i)| identity_matches(p, *i)) {
                self.context_prepared = None;
                self.error("Context state published; root changed and departure was canceled")
            } else {
                self.finish_context_departure(goal)
            }
        } else {
            self.message("Project registered; live buffers, positions and unsaved text retained")
        }
    }

    fn context_publication_warning(&mut self, action: Action, warning: Option<&str>) {
        if let Some(warning) = warning {
            if action == Action::Exit {
                eprintln!("{warning}");
            } else {
                let message = format!("{}; {warning}", self.echo);
                self.notice(&message);
            }
        }
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
        let action = self.finish_context_capture(after.membership, after.goal, &after.roots);
        if matches!(after.goal, Some(Goal::Context(_)))
            && self.context_generation == after.generation
            && !self.echo.contains("state published")
        {
            let message = format!("Context state published; {}", self.echo);
            self.error(&message);
        }
        self.context_publication_warning(action, warning);
        action
    }
}
fn publish_capture(path: &std::path::Path, prepared: &context::Prepared) -> Result<Output, String> {
    let (store, _) = Store::open(path).map_err(|e| e.to_string())?;
    store
        .publish(prepared)
        .map(Output::Published)
        .map_err(|e| e.to_string())
}

impl CapturedSession {
    fn candidate(
        self,
        catalog: &Catalog,
        membership: Option<&Workspace>,
        management: Option<(super::management::Change, bool)>,
    ) -> Result<Catalog, String> {
        let Self {
            owner,
            registration,
            entries,
            unavailable,
        } = self;
        let mut candidate = catalog.clone();
        let mut snapshot = Snapshot::default();
        if let Some(owner) = &owner {
            snapshot = capture_snapshot(owner, entries, unavailable, membership)?;
            if !management.as_ref().is_some_and(|(_, captured)| *captured)
                && owner.current(&candidate).is_some()
            {
                owner.checkpoint(&mut candidate, snapshot.clone(), membership)?;
            }
        }
        register_capture(registration, owner.as_ref(), &snapshot, &mut candidate)?;
        if let Some((change, capture_source)) = management {
            change.apply(&mut candidate, capture_source.then_some(snapshot))?;
        }
        Ok(candidate)
    }
}

fn capture_snapshot(
    owner: &Record,
    entries: Vec<Captured>,
    unavailable: Vec<FileEntry>,
    membership: Option<&Workspace>,
) -> Result<Snapshot, String> {
    let mut snapshot = Snapshot::default();
    let roots = membership.map_or_else(|| owner.roots(), |m| m.roots.as_slice());
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
    retain_unavailable(&mut snapshot, owner, unavailable, membership)?;
    if snapshot.files.len() > context::MAX_FILES {
        return Err("Context session file capacity exceeded".into());
    }
    Ok(snapshot)
}

fn retain_unavailable(
    snapshot: &mut Snapshot,
    owner: &Record,
    unavailable: Vec<FileEntry>,
    membership: Option<&Workspace>,
) -> Result<(), String> {
    let roots = membership.map_or_else(|| owner.roots(), |m| m.roots.as_slice());
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
    Ok(())
}

fn register_capture(
    registration: Option<Record>,
    owner: Option<&Record>,
    snapshot: &Snapshot,
    candidate: &mut Catalog,
) -> Result<(), String> {
    if let Some(registration) = registration {
        let initial = if owner.is_some_and(|o| o.context() == registration.context()) {
            snapshot.clone()
        } else {
            Snapshot::default()
        };
        registration.insert(candidate, initial)?;
    }
    Ok(())
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
