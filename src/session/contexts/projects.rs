use std::collections::VecDeque;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::context::{
    self, Catalog, Context, FileEntry, Generation, NativePath, Project, ProjectId, Store,
};
use crate::file::{Loaded, Observation};
use crate::org::Job;

use super::super::{Action, Pending, Session};
use super::Replacement;
use super::checkpoint::{After, Capture};
use super::record::Record;

#[derive(Default)]
pub(in crate::session) struct Persistence {
    pub(super) path: Option<PathBuf>,
    error: Option<String>,
    canceled: bool,
    pub(super) catalog: Option<Catalog>,
    pub(super) job: Job<Result<Output, String>>,
    pub(super) operation: Option<Operation>,
}
impl Persistence {
    pub(in crate::session) fn busy(&self) -> bool {
        self.operation.is_some() || self.job.is_active()
    }
    pub(in crate::session) fn locked(&self) -> bool {
        matches!(
            self.operation,
            Some(
                Operation::Capture(_)
                    | Operation::Publish(_)
                    | Operation::ManagementPublish
                    | Operation::Confirm { .. }
            )
        )
    }
    pub(in crate::session) fn departing(&self) -> bool {
        self.locked()
    }
}

pub(super) enum Operation {
    Read {
        request: Request,
        generation: Generation,
    },
    Restore(Box<Restore>),
    ManagementPublish,
    Capture(Box<Capture>),
    Publish(After),
    Confirm {
        goal: super::super::quit::Goal,
        generation: Generation,
        stamps: Vec<crate::buffer::Stamp>,
    },
}
pub(super) enum Request {
    List,
    Manage(crate::command::Command),
    ManageSelected(crate::command::Command, Record),
    Create { path: PathBuf, promote: bool },
    Open(Project),
    WorkspaceCreate,
    WorkspaceList,
    WorkspaceOpen(context::Workspace),
    WorkspaceDirectory(PathBuf),
    WorkspaceProjects,
    WorkspaceProject(Project),
}
pub(super) enum Output {
    Read(Catalog),
    Files(Vec<(FileEntry, Result<Ingress, String>)>),
    Published(context::Publication),
    Candidate(Catalog),
}
pub(super) struct Ingress {
    path: PathBuf,
    loaded: Loaded,
    observation: Observation,
}
pub(super) struct Restore {
    replacement: Replacement,
    files: VecDeque<FileEntry>,
    active: Option<context::FileReference>,
    selected: bool,
}
pub(in crate::session) struct Creation {
    catalog: Catalog,
    path: NativePath,
    identity: (u64, u64),
    promote: bool,
    generation: Generation,
}
pub(in crate::session) struct Choice {
    pub(super) label: String,
    pub(super) project: Project,
}

impl Session {
    /// Opt an embedded session into one host-owned state file. Pure constructors
    /// never discover or read the user's catalog. Configure before Project use.
    pub fn set_context_storage(&mut self, path: PathBuf) -> std::io::Result<()> {
        if self.context_storage.busy() || self.context_record.is_some() {
            return Err(std::io::Error::other("Project state is already owned"));
        }
        NativePath::absolute(&path).map_err(std::io::Error::other)?;
        if path.file_name().is_none_or(|name| name != "contexts.toml") {
            return Err(std::io::Error::other(
                "Project state requires contexts.toml",
            ));
        }
        self.context_storage = Persistence {
            path: Some(path),
            ..Default::default()
        };
        Ok(())
    }
    pub(in crate::session) fn startup_contexts(&mut self) {
        match crate::config::Roots::from_env() {
            Ok(roots) => self.context_storage.path = Some(roots.state.join("contexts.toml")),
            Err(error) => self.context_storage.error = Some(error.to_string()),
        }
    }
    pub(in crate::session) fn cancel_context_operation(&mut self) -> bool {
        if matches!(self.context_storage.operation, Some(Operation::Capture(_))) {
            self.context_storage.operation = None;
            self.context_prepared = None;
            self.context_management = None;
            self.pending = None;
            self.input.cancel();
            return true;
        }
        if !self.context_storage.busy()
            || matches!(
                self.context_storage.operation,
                Some(Operation::Publish(_) | Operation::ManagementPublish)
            )
        {
            return false;
        }
        self.context_prepared = None;
        self.context_management = None;
        self.context_storage.canceled = true;
        self.context_storage.job.cancel();
        self.pending = None;
        self.input.cancel();
        true
    }
    pub(in crate::session) fn project_create(&mut self) -> Action {
        if self.context_storage.busy() || self.context_prepared.is_some() {
            return self.error("Project preparation is already pending");
        }
        let action = self.start_path_mode(true);
        if matches!(self.pending, Some(Pending::Path(_))) {
            self.project_directory_prompt = true;
        }
        action
    }
    pub(in crate::session) fn project_promote(&mut self) -> Action {
        let Context::Directory(path) = &self.context else {
            return self.error("Project promotion requires an active temporary directory");
        };
        self.project_directory(path.as_path().to_owned(), true)
    }
    pub(in crate::session) fn project_directory(&mut self, path: PathBuf, promote: bool) -> Action {
        self.read_projects(Request::Create { path, promote })
    }
    pub(in crate::session) fn project_list(&mut self) -> Action {
        self.read_projects(Request::List)
    }
    pub(super) fn read_projects(&mut self, request: Request) -> Action {
        if self.context_departure_busy() || self.context_management.is_some() {
            return self.error("Project preparation or context departure is already pending");
        }
        if self.context_draft_busy() {
            return self
                .error("A source draft or output owns this context; finish or cancel it first");
        }
        if let Some(error) = self.context_storage.error.clone() {
            return self.error(&error);
        }
        let Some(path) = self.context_storage.path.clone() else {
            return self.error("Context storage is not configured by this session's host");
        };
        self.context_storage.canceled = false;
        let generation = self.context_generation;
        let stamp = self.view().point.stamp;
        if let Err(error) = self.context_storage.job.start(stamp, move |cancel| {
            if cancel.is_cancelled() {
                return Err("Project preparation canceled".into());
            }
            Store::open(&path)
                .map(|(_, catalog)| Output::Read(catalog))
                .map_err(|e| e.to_string())
        }) {
            return self.error(&error.to_string());
        }
        self.context_storage.operation = Some(Operation::Read {
            request,
            generation,
        });
        self.message("Reading contexts; live sources retained")
    }
    pub(in crate::session) fn confirm_project_context(
        &mut self,
        goal: super::super::quit::Goal,
    ) -> Action {
        let Some(path) = self.context_storage.path.clone() else {
            self.context_prepared = None;
            return self.error("Project storage is unavailable; live sources retained");
        };
        let stamps = self.buffers.iter().map(|b| b.stamp()).collect();
        let generation = self.context_generation;
        if let Err(error) = self
            .context_storage
            .job
            .start(self.view().point.stamp, move |cancel| {
                if cancel.is_cancelled() {
                    return Err("Project confirmation canceled".into());
                }
                Store::open(&path)
                    .map(|(_, catalog)| Output::Read(catalog))
                    .map_err(|e| e.to_string())
            })
        {
            self.context_prepared = None;
            return self.error(&error.to_string());
        }
        self.context_storage.operation = Some(Operation::Confirm {
            goal,
            generation,
            stamps,
        });
        self.message("Checking selected context before activation; live sources retained")
    }
    pub(in crate::session) fn context_remaining(&self) -> Option<Duration> {
        if self.context_storage.job.is_active() {
            Some(Duration::from_millis(10))
        } else if matches!(self.context_storage.operation, Some(Operation::Capture(_))) {
            Some(Duration::ZERO)
        } else {
            None
        }
    }
    pub(in crate::session) fn expire_contexts(&mut self) -> Action {
        if matches!(self.context_storage.operation, Some(Operation::Capture(_))) {
            return self.advance_context_capture();
        }
        let Some(result) = self.context_storage.job.poll_effect() else {
            return Action::Wait;
        };
        let operation = self.context_storage.operation.take();
        if std::mem::take(&mut self.context_storage.canceled) {
            return self.message("Project preparation canceled; live sources retained");
        }
        let result = result
            .map_err(|e| e.to_string())
            .and_then(|(_, value)| value);
        self.context_operation_result(operation, result)
    }

    fn context_operation_result(
        &mut self,
        operation: Option<Operation>,
        result: Result<Output, String>,
    ) -> Action {
        match (operation, result) {
            (
                Some(Operation::Read {
                    request,
                    generation,
                }),
                Ok(Output::Read(catalog)),
            ) => {
                if generation != self.context_generation {
                    return self.error("Project preparation is stale; live sources retained");
                }
                self.context_storage.catalog = Some(catalog.clone());
                self.projects_read(request, catalog)
            }
            (Some(Operation::Restore(restore)), Ok(Output::Files(files))) => {
                self.admit_project_files(*restore, files)
            }
            (
                Some(Operation::Confirm {
                    goal,
                    generation,
                    stamps,
                }),
                Ok(Output::Read(catalog)),
            ) => self.confirm_context_read(goal, generation, stamps, &catalog),
            (Some(Operation::ManagementPublish), Ok(Output::Published(publication))) => {
                self.management_published(publication)
            }
            (Some(Operation::Publish(after)), Ok(Output::Candidate(candidate))) => {
                self.management_captured(after, &candidate)
            }
            (Some(Operation::Publish(after)), Ok(Output::Published(publication))) => {
                self.context_published(after, publication)
            }
            (_, Err(error)) => {
                self.context_prepared = None;
                self.context_management = None;
                self.error(&format!("Contexts: {error}; live sources retained"))
            }
            _ => self.error("Project worker result did not match its owner; live sources retained"),
        }
    }
    fn confirm_context_read(
        &mut self,
        goal: super::super::quit::Goal,
        generation: Generation,
        stamps: Vec<crate::buffer::Stamp>,
        catalog: &Catalog,
    ) -> Action {
        if generation != self.context_generation
            || !self.buffers.iter().map(|b| b.stamp()).eq(stamps)
            || self.context_storage.catalog.as_ref() != Some(catalog)
        {
            self.context_prepared = None;
            return self.error("Selected context changed before activation; live sources retained");
        }
        self.finish_context_departure(goal)
    }

    fn projects_read(&mut self, request: Request, catalog: Catalog) -> Action {
        match request {
            Request::List => self.offer_projects(catalog.projects),
            Request::Manage(command) => self.management_read(command, catalog),
            Request::ManageSelected(command, record) => {
                self.management_selected(command, record, catalog)
            }
            Request::Create { path, promote } => {
                let path = match NativePath::directory(&path) {
                    Ok(path) => path,
                    Err(error) => return self.error(&error.to_string()),
                };
                if let Some(project) = catalog
                    .projects
                    .iter()
                    .find(|p| same_directory(p.root.path.as_path(), path.as_path()))
                    .cloned()
                {
                    return self.offer_projects(vec![project]);
                }
                let metadata = match std::fs::metadata(path.as_path()) {
                    Ok(m) => m,
                    Err(e) => return self.error(&e.to_string()),
                };
                let initial = path
                    .as_path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .filter(|s| !s.chars().any(char::is_control))
                    .unwrap_or("Project")
                    .to_owned();
                self.ask_project_name(
                    Creation {
                        catalog,
                        path,
                        identity: (metadata.dev(), metadata.ino()),
                        promote,
                        generation: self.context_generation,
                    },
                    &initial,
                    "",
                )
            }
            Request::Open(expected) => {
                if !catalog.projects.iter().any(|p| p == &expected) {
                    return self.error("Selected Project changed; reopen the Project picker");
                }
                self.prepare_project(expected, &catalog)
            }
            request => self.workspaces_read(request, catalog),
        }
    }
    fn offer_projects(&mut self, projects: Vec<Project>) -> Action {
        if projects.is_empty() {
            return self.message("No registered Projects; use project-create or project-promote");
        }
        let choices = project_choices(projects);
        let labels = choices.iter().map(|c| c.label.clone()).collect();
        if let Err(error) = self.input.names_argument("Project: ", "", labels) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Projects(choices));
        self.message("Select a Project by name/path; C-g retains the current context")
    }
    pub(in crate::session) fn project_selected(
        &mut self,
        choices: Vec<Choice>,
        label: &str,
    ) -> Action {
        let Some(choice) = choices.into_iter().find(|c| c.label == label) else {
            return self.error("Select an offered Project identity");
        };
        if self.context == Context::Project(choice.project.id.clone()) {
            self.leave_dashboard();
            return self.message("Project is already active; live work retained");
        }
        self.read_projects(Request::Open(choice.project))
    }
    fn ask_project_name(&mut self, creation: Creation, initial: &str, message: &str) -> Action {
        if let Err(error) = self.input.argument("Project name: ", initial) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::ProjectName(Box::new(creation)));
        self.message(message)
    }
    fn register_project(&mut self, creation: Creation, project: Project) -> Action {
        self.context_storage.catalog = Some(creation.catalog);
        if creation.promote {
            return self.capture_project_promotion(project, creation.identity);
        }
        let mut replacement = match self.prepare_context(Some(creation.path.as_path())) {
            Ok(r) => r,
            Err(e) => return self.error(&e.to_string()),
        };
        if replacement.root_identity != Some(creation.identity) {
            return self.error("Project directory changed; live sources retained");
        }
        replacement.session.context = Context::Project(project.id.clone());
        replacement.session.context_record = Some(Record::Project(project.clone()));
        replacement.registration = Some(Record::Project(project));
        self.begin_context(replacement)
    }

    pub(in crate::session) fn project_name(&mut self, creation: Creation, text: &str) -> Action {
        if creation.generation != self.context_generation {
            return self.error("Project creation source changed; live sources retained");
        }
        let name = match context::Name::new(text) {
            Ok(name) => name,
            Err(error) => return self.ask_project_name(creation, text, &error.to_string()),
        };
        if creation.catalog.projects.iter().any(|p| p.name == name) {
            return self.ask_project_name(creation, text, "Project name is already registered");
        }
        if creation.catalog.projects.len() >= context::MAX_PROJECTS {
            return self.error("Project capacity reached; live sources retained");
        }
        if !identity_matches(creation.path.as_path(), creation.identity) {
            return self.error("Project directory changed; live sources retained");
        }
        if creation.promote && self.context != Context::Directory(creation.path.clone()) {
            return self.error("Promotion source changed; live sources retained");
        }
        let project = match new_project(creation.path.clone(), name) {
            Ok(p) => p,
            Err(e) => return self.error(&e.to_string()),
        };
        self.register_project(creation, project)
    }
    fn prepare_project(&mut self, project: Project, catalog: &Catalog) -> Action {
        self.prepare_record(Record::Project(project), catalog)
    }
    pub(super) fn prepare_record(&mut self, record: Record, catalog: &Catalog) -> Action {
        let snapshot = match catalog.session(record.session_ref()).map(|s| &s.snapshot) {
            Some(Ok(snapshot)) => snapshot.clone(),
            Some(Err(error)) => {
                return self.error(&format!(
                    "{} {} has a damaged session: {error}; saved bytes retained",
                    record.kind(),
                    record.name()
                ));
            }
            None => return self.error("Selected Project session is unavailable"),
        };
        let mut replacement = match &record {
            Record::Project(project) => {
                match self.prepare_context(Some(project.root.path.as_path())) {
                    Ok(r) => r,
                    Err(error) => {
                        return self.error(&format!(
                            "Project root unavailable: {error}; registration retained"
                        ));
                    }
                }
            }
            Record::Workspace(workspace) => {
                let mut replacement = match self.prepare_context(None) {
                    Ok(r) => r,
                    Err(e) => return self.error(&e.to_string()),
                };
                replacement.session.tree =
                    self.tree
                        .opening_workspace(workspace.id.clone(), &workspace.roots, self.size);
                replacement.workspace_observations = workspace
                    .roots
                    .iter()
                    .map(|root| {
                        (
                            root.path.clone(),
                            super::workspaces::observe(root.path.as_path()),
                        )
                    })
                    .collect();
                replacement
            }
        };
        replacement.session.context = record.context();
        replacement.session.context_record = Some(record);
        self.load_project_batch(Restore {
            replacement,
            files: snapshot.files.into(),
            active: snapshot.active_file,
            selected: false,
        })
    }
    fn load_project_batch(&mut self, mut restore: Restore) -> Action {
        if restore.files.is_empty() {
            if restore.selected {
                restore.replacement.session.tree.edit();
            }
            if self.management_staging() {
                return self.management_replacement(restore.replacement);
            }
            return self.begin_context(restore.replacement);
        }
        let entries: Vec<_> = restore
            .files
            .drain(..restore.files.len().min(context::FILE_ADMISSION_STEPS))
            .collect();
        let roots = restore
            .replacement
            .session
            .context_record
            .as_ref()
            .expect("prepared context owns roots")
            .roots()
            .to_vec();
        let stamp = self.view().point.stamp;
        if let Err(error) = self.context_storage.job.start(stamp, move |cancel| {
            let files = entries
                .into_iter()
                .map(|entry| {
                    let result = if cancel.is_cancelled() {
                        Err("Project loading canceled".into())
                    } else {
                        load_entry(&entry, &roots)
                    };
                    (entry, result)
                })
                .collect();
            Ok(Output::Files(files))
        }) {
            return self.error(&error.to_string());
        }
        self.context_storage.operation = Some(Operation::Restore(Box::new(restore)));
        self.message("Preparing Project files; live sources retained")
    }
    fn admit_project_files(
        &mut self,
        mut restore: Restore,
        files: Vec<(FileEntry, Result<Ingress, String>)>,
    ) -> Action {
        if restore.replacement.expected != self.context_generation {
            return self.error("Project preparation is stale; live sources retained");
        }
        let destination = &mut restore.replacement.session;
        for (entry, ingress) in files {
            let result = ingress.and_then(|file| {
                let visit = destination
                    .buffers
                    .prepare_observed_visit(&file.path, file.loaded, &file.observation)
                    .map_err(|e| e.to_string())?;
                let id = destination.buffers.publish_visit(visit);
                let view = destination.restore_context_position(id, &entry)?;
                if !restore.selected || restore.active.as_ref() == Some(&entry.file) {
                    *destination.windows.selected_mut() = view;
                    restore.selected = true;
                }
                Ok(())
            });
            if result.is_err() {
                destination.context_unavailable.push(entry);
            }
        }
        self.load_project_batch(restore)
    }
}
fn load_entry(entry: &FileEntry, roots: &[context::Root]) -> Result<Ingress, String> {
    let path = entry.file.resolve(roots).map_err(|e| e.to_string())?;
    let before = crate::file::resolve(&path).map_err(|e| e.to_string())?;
    let observation = before
        .observation()
        .cloned()
        .ok_or("Saved file is unavailable")?;
    let loaded = crate::file::load(&path).map_err(|e| e.to_string())?;
    let after = crate::file::resolve(&path).map_err(|e| e.to_string())?;
    if after.observation() != Some(&observation) {
        return Err("Saved file changed while loading".into());
    }
    Ok(Ingress {
        path,
        loaded,
        observation,
    })
}
pub(super) fn identity_matches(path: &Path, identity: (u64, u64)) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_dir() && (m.dev(), m.ino()) == identity)
}
pub(super) fn same_directory(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (std::fs::metadata(left), std::fs::metadata(right)) {
        (Ok(a), Ok(b)) => (a.dev(), a.ino()) == (b.dev(), b.ino()),
        _ => false,
    }
}
fn new_project(path: NativePath, name: context::Name) -> Result<Project, context::Error> {
    let id = ProjectId::new()?;
    Ok(Project {
        session_ref: context::SessionRef {
            owner: context::Owner::Project(id.clone()),
            revision: context::Revision::INITIAL,
        },
        id,
        revision: context::Revision::INITIAL,
        name: name.clone(),
        roots_revision: context::Revision::INITIAL,
        root: context::Root {
            id: context::RootId::new()?,
            path,
            label: name,
            origin_project_id: None,
        },
    })
}

pub(super) fn project_choices(projects: Vec<Project>) -> Vec<Choice> {
    projects
        .into_iter()
        .map(|project| Choice {
            label: format!(
                "{} — {}",
                project.name.as_str(),
                project
                    .root
                    .path
                    .as_path()
                    .display()
                    .to_string()
                    .escape_default()
            ),
            project,
        })
        .collect()
}
