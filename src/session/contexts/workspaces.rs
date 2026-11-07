use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::buffer::{BufferId, Closing};
use crate::context::{
    self, Catalog, Context, Generation, Name, NativePath, Project, Root, RootId, SessionRef,
    Workspace, WorkspaceId,
};

use super::super::{
    Action, AfterSave, Pending, Session,
    quit::{Goal, Sources},
};
use super::projects::{Choice, Request, same_directory};
use super::record::Record;

pub(in crate::session) struct Membership {
    pub(super) expected: Workspace,
    pub(super) next: Workspace,
    pub(super) generation: Generation,
    pub(super) closed: Vec<BufferId>,
    pub(super) closing: Option<Closing>,
    pub(super) observations: Vec<(PathBuf, (u64, u64))>,
}
pub(in crate::session) struct Creation {
    catalog: Catalog,
    generation: Generation,
}
pub(in crate::session) struct WorkspaceChoice {
    label: String,
    workspace: Workspace,
}
pub(in crate::session) struct RootPicker {
    workspace: Workspace,
    choices: Vec<(String, RootId)>,
}

impl Session {
    pub(in crate::session) fn workspace_create(&mut self) -> Action {
        self.read_projects(Request::WorkspaceCreate)
    }
    pub(in crate::session) fn workspace_list(&mut self) -> Action {
        self.read_projects(Request::WorkspaceList)
    }
    pub(in crate::session) fn workspace_add_directory(&mut self) -> Action {
        if let Err(e) = self.active_workspace() {
            return self.error(&e);
        }
        if self.context_storage.busy() || self.context_membership.is_some() {
            return self.error("Workspace state work is already pending");
        }
        let action = self.start_path_mode(true);
        if matches!(self.pending, Some(Pending::Path(_))) {
            self.workspace_directory_prompt = true;
        }
        action
    }
    pub(in crate::session) fn workspace_add_project(&mut self) -> Action {
        if let Err(e) = self.active_workspace() {
            return self.error(&e);
        }
        self.read_projects(Request::WorkspaceProjects)
    }
    pub(super) fn workspaces_read(&mut self, request: Request, catalog: Catalog) -> Action {
        match request {
            Request::WorkspaceCreate => self.ask_workspace_name(
                Creation {
                    catalog,
                    generation: self.context_generation,
                },
                "",
                "",
            ),
            Request::WorkspaceList => {
                let choices: Vec<_> = catalog
                    .workspaces
                    .into_iter()
                    .map(|workspace| WorkspaceChoice {
                        label: format!(
                            "{} — {} roots {}",
                            workspace.name.as_str(),
                            workspace.roots.len(),
                            workspace
                                .roots
                                .iter()
                                .map(|r| r
                                    .path
                                    .as_path()
                                    .display()
                                    .to_string()
                                    .escape_default()
                                    .to_string())
                                .collect::<Vec<_>>()
                                .join(" | ")
                        ),
                        workspace,
                    })
                    .collect();
                if choices.is_empty() {
                    return self.message("No registered Workspaces; use workspace-create");
                }
                if let Err(e) = self.input.names_argument(
                    "Workspace: ",
                    "",
                    choices.iter().map(|c| c.label.clone()).collect(),
                ) {
                    return self.error(&e.to_string());
                }
                self.pending = Some(Pending::Workspaces(choices));
                self.message("Select a Workspace by name/path; C-g retains current work")
            }
            Request::WorkspaceOpen(workspace) => {
                if !catalog.workspaces.contains(&workspace) {
                    return self.error("Selected Workspace changed; reopen the picker");
                }
                self.prepare_record(Record::Workspace(workspace), &catalog)
            }
            Request::WorkspaceDirectory(path) => self.add_workspace_root(&path, None),
            Request::WorkspaceProjects => self.offer_workspace_projects(catalog.projects),
            Request::WorkspaceProject(project) => {
                if !catalog.projects.contains(&project) {
                    return self.error("Selected Project changed; reopen the picker");
                }
                self.add_workspace_root(project.root.path.as_path(), Some(project.id.clone()))
            }
            _ => self.error("Workspace operation no longer owns its request"),
        }
    }
    fn ask_workspace_name(&mut self, creation: Creation, text: &str, message: &str) -> Action {
        if let Err(e) = self.input.argument("Workspace name: ", text) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::WorkspaceName(Box::new(creation)));
        self.message(message)
    }
    pub(in crate::session) fn workspace_name(&mut self, creation: Creation, text: &str) -> Action {
        if creation.generation != self.context_generation {
            return self.error("Workspace creation source changed; live work retained");
        }
        let name = match Name::new(text) {
            Ok(n) => n,
            Err(e) => return self.ask_workspace_name(creation, text, &e.to_string()),
        };
        if creation.catalog.workspaces.iter().any(|w| w.name == name) {
            return self.ask_workspace_name(creation, text, "Workspace name is already registered");
        }
        if creation.catalog.workspaces.len() >= context::MAX_WORKSPACES {
            return self.error("Workspace capacity reached; live work retained");
        }
        let id = match WorkspaceId::new() {
            Ok(id) => id,
            Err(e) => return self.error(&e.to_string()),
        };
        let workspace = Workspace {
            session_ref: SessionRef {
                owner: context::Owner::Workspace(id.clone()),
                revision: context::Revision::INITIAL,
            },
            id,
            name,
            revision: context::Revision::INITIAL,
            roots_revision: context::Revision::INITIAL,
            roots: Vec::new(),
        };
        self.context_storage.catalog = Some(creation.catalog);
        let mut replacement = match self.prepare_context(None) {
            Ok(r) => r,
            Err(e) => return self.error(&e.to_string()),
        };
        replacement.session.tree =
            self.tree
                .opening_workspace(workspace.id.clone(), &[], self.size);
        replacement.session.context = Context::Workspace(workspace.id.clone());
        replacement.session.context_record = Some(Record::Workspace(workspace.clone()));
        replacement.registration = Some(Record::Workspace(workspace));
        self.begin_context(replacement)
    }
    pub(in crate::session) fn workspace_selected(
        &mut self,
        choices: Vec<WorkspaceChoice>,
        label: &str,
    ) -> Action {
        let Some(choice) = choices.into_iter().find(|c| c.label == label) else {
            return self.error("Select an offered Workspace identity");
        };
        if self.context == Context::Workspace(choice.workspace.id.clone()) {
            self.leave_dashboard();
            return self.message("Workspace is already active; live work retained");
        }
        self.read_projects(Request::WorkspaceOpen(choice.workspace))
    }
    fn offer_workspace_projects(&mut self, projects: Vec<Project>) -> Action {
        let choices = super::projects::project_choices(projects);
        if choices.is_empty() {
            return self.message("No registered Projects; add a directory directly");
        }
        if let Err(e) = self.input.names_argument(
            "Add Project directory: ",
            "",
            choices.iter().map(|c| c.label.clone()).collect(),
        ) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::WorkspaceProjects(choices));
        self.message("Add the Project directory; its saved files remain independent")
    }
    pub(in crate::session) fn workspace_project_selected(
        &mut self,
        choices: Vec<Choice>,
        label: &str,
    ) -> Action {
        let Some(choice) = choices.into_iter().find(|c| c.label == label) else {
            return self.error("Select an offered Project identity");
        };
        self.read_projects(Request::WorkspaceProject(choice.project))
    }
    pub(in crate::session) fn workspace_directory(&mut self, path: PathBuf) -> Action {
        self.read_projects(Request::WorkspaceDirectory(path))
    }
    fn active_workspace(&self) -> Result<&Workspace, String> {
        match &self.context_record {
            Some(Record::Workspace(w)) => Ok(w),
            _ => Err("This command requires an active Workspace".into()),
        }
    }
    fn add_workspace_root(&mut self, path: &Path, origin: Option<context::ProjectId>) -> Action {
        let workspace = match self.active_workspace() {
            Ok(w) => w.clone(),
            Err(e) => return self.error(&e),
        };
        let path = match NativePath::directory(path) {
            Ok(p) => p,
            Err(e) => return self.error(&e.to_string()),
        };
        if workspace.roots.len() >= context::MAX_ROOTS {
            return self.error("Workspace root capacity reached; live work retained");
        }
        if workspace
            .roots
            .iter()
            .any(|r| same_directory(r.path.as_path(), path.as_path()))
        {
            return self.error("Directory is already a Workspace root");
        }
        let label = path
            .as_path()
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| Name::new(n).ok())
            .unwrap_or_else(|| Name::new("Root").expect("literal root label"));
        let id = match RootId::new() {
            Ok(id) => id,
            Err(e) => return self.error(&e.to_string()),
        };
        let mut next = workspace.clone();
        next.roots.push(Root {
            id,
            path,
            label,
            origin_project_id: origin,
        });
        self.prepare_membership(workspace, next, None)
    }
    pub(in crate::session) fn workspace_remove_root(&mut self) -> Action {
        let workspace = match self.active_workspace() {
            Ok(w) => w.clone(),
            Err(e) => return self.error(&e),
        };
        if workspace.roots.is_empty() {
            return self.message("Workspace has no roots to remove");
        }
        if self.context_storage.busy()
            || self.context_prepared.is_some()
            || self.context_membership.is_some()
        {
            return self.error("Context state work is already pending");
        }
        let choices: Vec<_> = workspace
            .roots
            .iter()
            .enumerate()
            .map(|(i, r)| {
                (
                    format!(
                        "{}: {} — {}",
                        i + 1,
                        r.label.as_str(),
                        r.path.as_path().display().to_string().escape_default()
                    ),
                    r.id.clone(),
                )
            })
            .collect();
        if let Err(e) = self.input.names_argument(
            "Remove Workspace root: ",
            "",
            choices.iter().map(|(l, _)| l.clone()).collect(),
        ) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::WorkspaceRoot(Box::new(RootPicker {
            workspace,
            choices,
        })));
        self.message("Choose a root; source directories and Project registrations are retained")
    }
    pub(in crate::session) fn workspace_root_selected(
        &mut self,
        picker: RootPicker,
        label: &str,
    ) -> Action {
        let Some((_, id)) = picker.choices.into_iter().find(|(l, _)| l == label) else {
            return self.error("Select an offered root identity");
        };
        let mut next = picker.workspace.clone();
        next.roots.retain(|r| r.id != id);
        self.prepare_membership(picker.workspace, next, Some(id))
    }
    fn prepare_membership(
        &mut self,
        expected: Workspace,
        next: Workspace,
        removed: Option<RootId>,
    ) -> Action {
        if self.context_storage.busy()
            || self.context_prepared.is_some()
            || self.context_membership.is_some()
            || self.git_shutdown_active()
            || self.terminal_shutdown_active()
        {
            return self.error("Context state work is already pending");
        }
        if self.capture.is_some()
            || self.source_edit.is_some()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
        {
            return self
                .error("A source draft or output owns this context; finish or cancel it first");
        }
        if self.context_record.as_ref() != Some(&Record::Workspace(expected.clone()))
            || !self
                .context_storage
                .catalog
                .as_ref()
                .is_some_and(|c| Record::Workspace(expected.clone()).matches(c))
        {
            return self.error("Workspace membership changed; reopen the root operation");
        }
        if let Err(e) = self.context_generation.next() {
            return self.error(&e.to_string());
        }
        let closed = if removed.is_some() {
            let result: Result<Vec<_>, String> = self
                .buffers
                .iter()
                .filter(|b| b.file().is_some() && !b.is_transient())
                .filter_map(|b| {
                    let path = b.file()?.path();
                    let old = super::checkpoint::capture_reference(path, &expected.roots);
                    let new = super::checkpoint::capture_reference(path, &next.roots);
                    match (old, new) {
                        (
                            Ok(context::FileReference::RootRelative { .. }),
                            Ok(context::FileReference::LooseAbsolute(_)),
                        ) => Some(Ok(b.id())),
                        (Err(e), _) | (_, Err(e)) => Some(Err(e)),
                        _ => None,
                    }
                })
                .collect();
            match result {
                Ok(ids) => ids,
                Err(e) => return self.error(&e),
            }
        } else {
            Vec::new()
        };
        let observations = next
            .roots
            .iter()
            .filter_map(|r| observe(r.path.as_path()).map(|i| (r.path.as_path().to_owned(), i)))
            .collect();
        self.context_membership = Some(Box::new(Membership {
            expected,
            next,
            generation: self.context_generation,
            closed,
            closing: None,
            observations,
        }));
        self.input.cancel();
        self.pending = None;
        if self.active_clock.as_ref().is_some_and(|c| {
            self.context_membership
                .as_ref()
                .is_some_and(|m| m.closed.contains(&c.buffer()))
        }) {
            return self.clock_departure(super::super::outline::clocks::Departure::Roots(
                self.context_generation,
            ));
        }
        self.begin_workspace_sources(self.context_generation)
    }
    pub(in crate::session) fn begin_workspace_sources(&mut self, generation: Generation) -> Action {
        let Some(m) = &self.context_membership else {
            return self.error("Root removal was canceled");
        };
        if m.generation != generation || generation != self.context_generation {
            return self.error("Root removal is stale; live work retained");
        }
        let targets = m.closed.clone();
        self.begin_scoped_departure(Goal::Roots(generation), targets)
    }
    pub(in crate::session) fn finish_workspace_sources(
        &mut self,
        generation: Generation,
        sources: Sources,
    ) -> Action {
        let Some(membership) = self.context_membership.take() else {
            return self.error("Workspace membership was canceled");
        };
        if membership.generation != generation
            || self.context_generation != generation
            || self.context_record.as_ref() != Some(&Record::Workspace(membership.expected.clone()))
            || !sources.permits(&self.buffers)
        {
            return self.error("Workspace sources changed; membership retained");
        }
        self.capture_membership(membership)
    }
    pub(super) fn publish_membership(&mut self, mut membership: Membership) -> Action {
        let Some(Record::Workspace(workspace)) = &self.context_record else {
            return self.error("Membership published; active Workspace is unavailable");
        };
        let workspace = workspace.clone();
        self.tree.workspace_roots(&workspace.roots);
        let closing = membership
            .closing
            .take()
            .expect("membership publication follows close admission");
        let (closed, replacement) = match self.buffers.publish_closing(closing) {
            Ok(v) => v,
            Err(e) => return self.error(&format!("Membership published; sources retained: {e}")),
        };
        for id in closed {
            let buffer = self
                .buffers
                .get(replacement)
                .expect("admitted replacement remains live");
            self.windows.replace_buffer(id, buffer);
            self.context_positions.remove(&id);
            self.lsp.closed(id);
            self.recent_clocks.retain(|p| p.stamp.buffer != id);
            self.refresh_buffer_completion(id);
        }
        let live: std::collections::BTreeSet<_> = self
            .buffers
            .iter()
            .filter_map(|b| b.file().map(|f| f.path().to_owned()))
            .collect();
        self.context_unavailable = self
            .context_storage
            .catalog
            .as_ref()
            .and_then(|c| {
                self.context_record
                    .as_ref()
                    .and_then(|r| c.session(r.session_ref()))
            })
            .and_then(|s| s.snapshot.as_ref().ok())
            .map(|s| {
                s.files
                    .iter()
                    .filter(|e| {
                        let path = match &e.file {
                            context::FileReference::LooseAbsolute(p) => {
                                Some(p.as_path().to_owned())
                            }
                            context::FileReference::RootRelative { root, path } => workspace
                                .roots
                                .iter()
                                .find(|r| r.id == *root)
                                .map(|r| r.path.as_path().join(path.as_path())),
                        };
                        path.is_none_or(|p| !live.contains(&p))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        self.context_generation = match self.context_generation.next() {
            Ok(g) => g,
            Err(e) => return self.error(&e.to_string()),
        };
        self.sync_input_context();
        self.observe_files();
        self.message("Workspace roots published; covered and loose buffers retained")
    }
    pub(in crate::session) fn retire_workspace_membership(&mut self) {
        let Some(membership) = &self.context_membership else {
            return;
        };
        let goal = Goal::Roots(membership.generation);
        let pending = match &self.pending {
            Some(Pending::Quit(q)) => q.goal() == goal,
            Some(
                Pending::Save {
                    after: AfterSave::Quit(q),
                    ..
                }
                | Pending::Write {
                    after: AfterSave::Quit(q),
                    ..
                },
            ) => q.goal() == goal,
            Some(Pending::Clock(p)) => p.root_departure() == Some(membership.generation),
            _ => false,
        };
        if !pending {
            self.context_membership = None;
        }
    }
}
pub(super) fn observe(path: &Path) -> Option<(u64, u64)> {
    std::fs::metadata(path)
        .ok()
        .filter(|m| m.is_dir())
        .map(|m| (m.dev(), m.ino()))
}
