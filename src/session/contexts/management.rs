mod change;

use super::super::{Action, Pending, Session, quit::Goal};
use super::{
    Replacement,
    projects::{Operation, Output, Request},
    record::Record,
};
use crate::command::Command;
use crate::context::{
    self, Catalog, Context, Generation, Name, NativePath, RootId, Store, Workspace,
};
pub(in crate::session) use change::Change;
use change::Mutation;

pub(in crate::session) struct Plan {
    change: Change,
    expected: Catalog,
    candidate: Option<Catalog>,
    generation: Generation,
    stamps: Vec<crate::buffer::Stamp>,
    leaves: bool,
    replacement: Option<Replacement>,
    observations: Vec<(std::path::PathBuf, (u64, u64))>,
}
pub(in crate::session) enum Prompt {
    Select {
        command: Command,
        choices: Vec<(String, Record)>,
    },
    Name(Naming),
    Confirm {
        record: Record,
        catalog: Catalog,
        generation: Generation,
    },
    Root {
        record: Record,
        catalog: Catalog,
        generation: Generation,
        choices: Vec<(String, RootId)>,
    },
    Path {
        record: Record,
        catalog: Catalog,
        generation: Generation,
        root: RootId,
    },
}

/// One name prompt retains the catalog and generation that admitted its target.
pub(in crate::session) struct Naming {
    command: Command,
    record: Record,
    catalog: Catalog,
    generation: Generation,
}

impl Session {
    pub(in crate::session) fn manage_context(&mut self, command: Command) -> Action {
        self.read_projects(Request::Manage(command))
    }
    pub(super) fn management_read(&mut self, command: Command, catalog: Catalog) -> Action {
        let projects = matches!(
            command,
            Command::ProjectRename
                | Command::ProjectForget
                | Command::ProjectSetRoot
                | Command::ProjectPromoteToWorkspace
                | Command::ProjectDuplicateAsWorkspace
        );
        let records: Vec<_> = if projects {
            catalog.projects.into_iter().map(Record::Project).collect()
        } else {
            catalog
                .workspaces
                .into_iter()
                .map(Record::Workspace)
                .collect()
        };
        let choices: Vec<_> = records
            .into_iter()
            .map(|r| {
                let label = format!(
                    "{} — {}",
                    r.name(),
                    r.roots()
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
                );
                (label, r)
            })
            .collect();
        if choices.is_empty() {
            return self.message("No registered contexts of the selected type");
        }
        if let Err(e) = self.input.names_argument(
            "Manage context: ",
            "",
            choices.iter().map(|(l, _)| l.clone()).collect(),
        ) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::ContextManagement(Box::new(Prompt::Select {
            command,
            choices,
        })));
        self.message("Select the artifact by name/path; C-g retains current work")
    }
    pub(in crate::session) fn management_argument(&mut self, prompt: Prompt, text: &str) -> Action {
        match prompt {
            Prompt::Select { command, choices } => {
                let Some((_, record)) = choices.into_iter().find(|(l, _)| l == text) else {
                    return self.error("Select an offered context identity");
                };
                self.read_projects(Request::ManageSelected(command, record))
            }
            Prompt::Name(naming) => self.management_named(naming, text),
            Prompt::Confirm {
                record,
                catalog,
                generation,
            } => {
                if text.trim() != "yes" {
                    return self.message("Forget canceled; registration and sources retained");
                }
                self.admit_management(
                    Change {
                        record,
                        mutation: Mutation::Forget,
                    },
                    catalog,
                    generation,
                )
            }
            Prompt::Root {
                record,
                catalog,
                generation,
                choices,
            } => {
                let Some((_, root)) = choices.into_iter().find(|(l, _)| l == text) else {
                    return self.error("Select an offered root identity");
                };
                self.management_path(record, catalog, generation, root)
            }
            Prompt::Path {
                record,
                catalog,
                generation,
                root,
            } => {
                let path = match NativePath::directory(std::path::Path::new(text)) {
                    Ok(p) => p,
                    Err(e) => return self.error(&e.to_string()),
                };
                if record
                    .roots()
                    .iter()
                    .find(|r| r.id == root)
                    .is_some_and(|r| r.path == path)
                {
                    return self.message("Root path is unchanged");
                }
                self.admit_management(
                    Change {
                        record,
                        mutation: Mutation::Relocate { root, path },
                    },
                    catalog,
                    generation,
                )
            }
        }
    }
    fn management_named(&mut self, naming: Naming, text: &str) -> Action {
        let Naming {
            command,
            record,
            catalog,
            generation,
        } = naming;
        let name = match Name::new(text) {
            Ok(n) => n,
            Err(e) => {
                return self.management_name(
                    Naming {
                        command,
                        record,
                        catalog,
                        generation,
                    },
                    text,
                    &e.to_string(),
                );
            }
        };
        let workspace = matches!(
            command,
            Command::ProjectPromoteToWorkspace | Command::ProjectDuplicateAsWorkspace
        );
        let conflict = if workspace {
            catalog.workspaces.iter().any(|w| w.name == name)
        } else {
            match &record {
                Record::Project(p) => catalog
                    .projects
                    .iter()
                    .any(|n| n.id != p.id && n.name == name),
                Record::Workspace(w) => catalog
                    .workspaces
                    .iter()
                    .any(|n| n.id != w.id && n.name == name),
            }
        };
        if conflict {
            return self.management_name(
                Naming {
                    command,
                    record,
                    catalog,
                    generation,
                },
                text,
                "Name is already registered",
            );
        }
        if !workspace && name.as_str() == record.name() {
            return self.message("Context name is unchanged");
        }
        let mutation = match named_mutation(command, &record, name, workspace) {
            Ok(mutation) => mutation,
            Err(error) => return self.error(&error),
        };
        self.admit_management(Change { record, mutation }, catalog, generation)
    }

    pub(super) fn management_selected(
        &mut self,
        command: Command,
        record: Record,
        catalog: Catalog,
    ) -> Action {
        if !record.matches(&catalog)
            || self
                .context_record
                .as_ref()
                .is_some_and(|r| r.context() == record.context() && r != &record)
        {
            return self.error("Selected context changed; reopen its picker");
        }
        let generation = self.context_generation;
        match command {
            Command::ProjectForget | Command::WorkspaceForget => {
                if let Err(e) = self.input.argument("Forget metadata? type yes: ", "") {
                    return self.error(&e.to_string());
                }
                self.pending = Some(Pending::ContextManagement(Box::new(Prompt::Confirm {
                    record,
                    catalog,
                    generation,
                })));
                self.message(
                    "Remove registration and its saved session; source files remain intact",
                )
            }
            Command::ProjectSetRoot => {
                let root = record.roots()[0].id.clone();
                self.management_path(record, catalog, generation, root)
            }
            Command::WorkspaceSetRootPath => self.management_root(record, catalog, generation),
            _ => {
                let initial = record.name().to_owned();
                self.management_name(
                    Naming {
                        command,
                        record,
                        catalog,
                        generation,
                    },
                    &initial,
                    "",
                )
            }
        }
    }
    fn management_root(
        &mut self,
        record: Record,
        catalog: Catalog,
        generation: Generation,
    ) -> Action {
        let choices: Vec<_> = record
            .roots()
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
        if choices.is_empty() {
            return self.message("Workspace has no roots to relocate");
        }
        if let Err(e) = self.input.names_argument(
            "Relocate root: ",
            "",
            choices.iter().map(|(l, _)| l.clone()).collect(),
        ) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::ContextManagement(Box::new(Prompt::Root {
            record,
            catalog,
            generation,
            choices,
        })));
        self.message("Choose a root identity; directories will not be moved")
    }

    fn management_name(&mut self, naming: Naming, text: &str, message: &str) -> Action {
        if let Err(e) = self.input.argument("New context name: ", text) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::ContextManagement(Box::new(Prompt::Name(naming))));
        self.message(message)
    }
    fn management_path(
        &mut self,
        record: Record,
        catalog: Catalog,
        generation: Generation,
        root: RootId,
    ) -> Action {
        if let Err(e) = self.input.argument("Replacement root path: ", "") {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::ContextManagement(Box::new(Prompt::Path {
            record,
            catalog,
            generation,
            root,
        })));
        self.message("Select an existing directory; metadata changes do not move source files")
    }
    fn admit_management(
        &mut self,
        change: Change,
        catalog: Catalog,
        generation: Generation,
    ) -> Action {
        if generation != self.context_generation
            || self.context_storage.busy()
            || !change.record.matches(&catalog)
        {
            return self.error("Context operation is stale; live sources retained");
        }
        if let Err(e) = generation.next() {
            return self.error(&e.to_string());
        }
        if change.relocation_conflicts(&catalog) {
            return self.error("Root conflict: directory is already registered in this scope");
        }
        let mut candidate = catalog.clone();
        if let Err(e) = change.apply(&mut candidate, None) {
            return self.error(&e);
        }
        let observations = change.observations();
        let Some(observations) = observations else {
            return self.error("Selected root is unavailable; registration retained");
        };
        let leaves = change.leaves(&self.context);
        let captures = change.captures(&self.context);
        self.context_management = Some(Box::new(Plan {
            change,
            expected: catalog,
            candidate: Some(candidate),
            generation,
            stamps: self.buffers.iter().map(|b| b.stamp()).collect(),
            leaves,
            replacement: None,
            observations,
        }));
        if leaves {
            if self.active_clock.is_some() {
                return self.clock_departure(super::super::outline::clocks::Departure::Context(
                    generation,
                ));
            }
            self.begin_departure(Goal::Context(generation))
        } else if captures {
            self.capture_management()
        } else {
            self.publish_management()
        }
    }
    pub(super) fn management_relocating(&self) -> bool {
        self.context_management
            .as_ref()
            .is_some_and(|p| matches!(p.change.mutation, Mutation::Relocate { .. }))
    }
    pub(super) fn management_change(&self) -> Option<Change> {
        self.context_management.as_ref().map(|p| p.change.clone())
    }
    pub(super) fn management_capture_source(&self) -> bool {
        self.context_management
            .as_ref()
            .is_some_and(|p| p.change.captures(&self.context))
    }
    pub(super) fn management_candidate(&mut self, candidate: &Catalog) -> Action {
        let Some(mut plan) = self.context_management.take() else {
            return self.error("Context operation was canceled");
        };
        plan.candidate = Some(candidate.clone());
        plan.stamps = self.buffers.iter().map(|b| b.stamp()).collect();
        let destination = plan.change.destination().and_then(|r| r.current(candidate));
        let leaves = plan.leaves;
        self.context_management = Some(plan);
        if !leaves {
            return self.publish_management();
        }
        if let Some(record) = destination {
            self.prepare_record(record, candidate)
        } else {
            match self.prepare_context(None) {
                Ok(mut r) => {
                    r.session.tree = self.tree.without_context();
                    r.startup = true;
                    self.management_replacement(r)
                }
                Err(e) => {
                    self.context_management = None;
                    self.error(&e.to_string())
                }
            }
        }
    }
    pub(super) fn management_staging(&self) -> bool {
        self.context_management.as_ref().is_some_and(|p| p.leaves)
    }
    pub(super) fn management_replacement(&mut self, replacement: Replacement) -> Action {
        let Some(plan) = &mut self.context_management else {
            return self.error("Context operation was canceled");
        };
        plan.replacement = Some(replacement);
        self.publish_management()
    }
    fn publish_management(&mut self) -> Action {
        let Some(plan) = &self.context_management else {
            return self.error("Context operation was canceled");
        };
        if plan.generation != self.context_generation
            || !self
                .buffers
                .iter()
                .map(|b| b.stamp())
                .eq(plan.stamps.iter().copied())
        {
            self.context_management = None;
            return self.error("Context source changed before publication; live work retained");
        }
        let observations = plan.observations.clone();
        let Some(candidate) = plan.candidate.clone() else {
            return self.error("Context candidate is unavailable");
        };
        let expected = plan.expected.clone();
        let Some(path) = self.context_storage.path.clone() else {
            return self.error("Context storage is unavailable");
        };
        if let Err(e) = self
            .context_storage
            .job
            .start(self.view().point.stamp, move |cancel| {
                if cancel.is_cancelled() {
                    return Err("Context publication canceled before effects".into());
                }
                if !observations
                    .iter()
                    .all(|(p, i)| super::projects::identity_matches(p, *i))
                {
                    return Err("Selected root changed before publication".into());
                }
                let prepared = expected.prepare(candidate).map_err(|e| e.to_string())?;
                if !observations
                    .iter()
                    .all(|(p, i)| super::projects::identity_matches(p, *i))
                {
                    return Err("Selected root changed before publication".into());
                }
                if cancel.is_cancelled() {
                    return Err("Context publication canceled before effects".into());
                }
                Store::open(&path)
                    .map_err(|e| e.to_string())?
                    .0
                    .publish(&prepared)
                    .map(Output::Published)
                    .map_err(|e| e.to_string())
            })
        {
            self.context_management = None;
            return self.error(&e.to_string());
        }
        self.context_storage.operation = Some(Operation::ManagementPublish);
        self.message("Publishing context metadata; live sources retained")
    }
    pub(super) fn management_published(&mut self, publication: context::Publication) -> Action {
        let Some(plan) = self.context_management.take() else {
            self.context_storage.catalog = Some(publication.catalog);
            return self.error("Context state published; activation owner was canceled");
        };
        let warning = publication.durability_warning;
        let current = plan.generation == self.context_generation
            && self.buffers.iter().map(|b| b.stamp()).eq(plan.stamps);
        self.context_storage.catalog = Some(publication.catalog.clone());
        self.apply_management_record(&plan.change, &publication.catalog);
        let action = if !current {
            self.error("Context state published; live sources changed and activation was canceled")
        } else if let Some(replacement) = plan.replacement {
            self.context_prepared = Some(Box::new(replacement));
            self.activate_context(plan.generation)
        } else {
            self.context_generation = self
                .context_generation
                .next()
                .expect("generation admitted before publication");
            self.sync_input_context();
            self.message("Context metadata published; source files and live work retained")
        };
        if let Some(warning) = warning {
            self.notice(&format!("{}; {warning}", self.echo));
        }
        action
    }
    fn apply_management_record(&mut self, change: &Change, catalog: &Catalog) {
        // Durable truth owns the identity after the commit point, including a
        // later activation error. Never retry conversion with another fresh ID.
        if self.context == change.record.context() {
            match &change.mutation {
                Mutation::Forget => {
                    if let Record::Project(p) = &change.record {
                        self.context = Context::Directory(p.root.path.clone());
                        self.context_record = None;
                        self.context_unavailable.clear();
                    } else {
                        self.context = Context::Standalone;
                        self.context_record = None;
                        self.context_unavailable.clear();
                        self.tree = self.tree.without_context();
                    }
                }
                Mutation::Workspace {
                    destination,
                    consume: true,
                } => {
                    self.context = Context::Workspace(destination.id.clone());
                    self.context_record = Some(Record::Workspace(destination.clone()));
                    let focused = self.tree.focused();
                    let visible = self.tree.visible();
                    self.tree = self.tree.opening_workspace(
                        destination.id.clone(),
                        &destination.roots,
                        self.size,
                    );
                    if !focused {
                        self.tree.edit();
                    }
                    if !visible {
                        self.tree.hide_context();
                    }
                    self.remap_context_unavailable(
                        &change.record.roots()[0].id,
                        &destination.roots[0].id,
                    );
                }
                _ => self.context_record = change.record.current(catalog),
            }
        }
    }

    fn remap_context_unavailable(&mut self, old: &RootId, new: &RootId) {
        for entry in &mut self.context_unavailable {
            if let context::FileReference::RootRelative { root, .. } = &mut entry.file
                && root == old
            {
                *root = new.clone();
            }
        }
    }
    pub(super) fn retire_management(&mut self) {
        let Some(plan) = &self.context_management else {
            return;
        };
        if self.context_storage.busy() {
            return;
        }
        let goal = Goal::Context(plan.generation);
        let pending = self.context_departure_pending(plan.generation);
        let terminal = self.terminal_context_goal() == Some(goal);
        let git = self.git.shutdown.as_ref().is_some_and(|s| s.goal == goal);
        if !pending && !terminal && !git {
            self.context_management = None;
        }
    }
}

fn named_mutation(
    command: Command,
    record: &Record,
    name: Name,
    workspace: bool,
) -> Result<Mutation, String> {
    let mutation = if workspace {
        let Record::Project(p) = record else {
            return Err("Conversion requires a Project".into());
        };
        let id = context::WorkspaceId::new().map_err(|e| e.to_string())?;
        let root_id = RootId::new().map_err(|e| e.to_string())?;
        let mut root = p.root.clone();
        root.id = root_id;
        root.origin_project_id = Some(p.id.clone());
        Mutation::Workspace {
            destination: Workspace {
                session_ref: context::SessionRef {
                    owner: context::Owner::Workspace(id.clone()),
                    revision: context::Revision::INITIAL,
                },
                id,
                revision: context::Revision::INITIAL,
                roots_revision: context::Revision::INITIAL,
                name,
                roots: vec![root],
            },
            consume: command == Command::ProjectPromoteToWorkspace,
        }
    } else {
        Mutation::Rename(name)
    };
    Ok(mutation)
}
