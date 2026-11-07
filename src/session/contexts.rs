mod checkpoint;
pub(super) mod management;
pub(super) mod positions;
pub(super) mod projects;
pub(super) mod record;
pub(super) mod workspaces;

use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::buffer::Buffers;
use crate::context::{Context, Generation, NativePath};
use crate::view::View;

use super::{Action, AfterSave, Pending, Session, quit::Goal};

pub(super) struct Replacement {
    session: Session,
    expected: Generation,
    root_identity: Option<(u64, u64)>,
    registration: Option<record::Record>,
    workspace_observations: Vec<(crate::context::NativePath, Option<(u64, u64)>)>,
}

impl Session {
    /// Observes context intent independently from tree visibility or file paths.
    pub fn context(&self) -> &Context {
        &self.context
    }
    pub fn context_generation(&self) -> Generation {
        self.context_generation
    }

    pub(in crate::session) fn project_confirmation_needed(&self) -> bool {
        self.context_prepared
            .as_ref()
            .is_some_and(|r| r.session.context_record.is_some())
    }

    pub(super) fn generate_new_buffer(&mut self) -> Action {
        if self.input.focus() != crate::input::Focus::Editing {
            return self.error("A prompt owns buffer creation");
        }
        match self.buffers.create("") {
            Ok(id) => {
                self.tree.edit();
                self.activate(id)
            }
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn open_directory(&mut self) -> Action {
        self.start_path_mode(true)
    }
    pub(super) fn close_context(&mut self) -> Action {
        match self.prepare_context(None) {
            Ok(replacement) => self.begin_context(replacement),
            Err(error) => self.error(&error.to_string()),
        }
    }
    pub(super) fn request_directory(&mut self, path: &Path) -> io::Result<()> {
        let replacement = self.prepare_context(Some(path))?;
        let previous = self.context_generation;
        self.input.cancel();
        self.pending = None;
        self.begin_context(replacement);
        if self.context_generation == previous && self.context_prepared.is_none() {
            return Err(io::Error::other(self.echo.clone()));
        }
        Ok(())
    }
    pub(in crate::session) fn prepare_context(
        &self,
        directory: Option<&Path>,
    ) -> io::Result<Replacement> {
        if self.context_membership.is_some()
            || self.context_storage.busy()
            || self.context_prepared.is_some()
            || self.git_shutdown_active()
            || self.terminal_shutdown_active()
        {
            return Err(io::Error::other(
                "Context departure is already pending; cancel it first",
            ));
        }
        if self.capture.is_some()
            || self.source_edit.is_some()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
        {
            return Err(io::Error::other(
                "A source draft or output owns this context; finish or cancel it first",
            ));
        }
        let buffers =
            Buffers::with_settings(self.buffers.snapshot().clone()).map_err(io::Error::other)?;
        let view = View::with_settings(
            buffers.get(buffers.initial()).map_err(io::Error::other)?,
            buffers.snapshot(),
        );
        let mut session = Self::from_buffers_with_wake(
            buffers,
            view,
            self.size,
            self.observation.wake.replacement(),
        )
        .map_err(io::Error::other)?;
        let root_identity = match directory {
            Some(path) => {
                let tree = self.tree.opening_directory(path, self.size)?;
                let root = tree
                    .root()
                    .ok_or_else(|| io::Error::other("Directory root is unavailable"))?;
                let metadata = std::fs::metadata(root)?;
                session.context =
                    Context::Directory(NativePath::absolute(root).map_err(io::Error::other)?);
                session.tree = tree;
                Some((metadata.dev(), metadata.ino()))
            }
            None => {
                session.tree.hide_context();
                None
            }
        };
        session.context_root_identity = root_identity;
        Ok(Replacement {
            session,
            expected: self.context_generation,
            root_identity,
            registration: None,
            workspace_observations: Vec::new(),
        })
    }
    pub(in crate::session) fn begin_context(&mut self, replacement: Replacement) -> Action {
        let expected = replacement.expected;
        if let Err(error) = expected.next() {
            return self.error(&error.to_string());
        }
        self.context_prepared = Some(Box::new(replacement));
        if self.active_clock.is_some() {
            self.clock_departure(super::outline::clocks::Departure::Context(expected))
        } else {
            self.begin_departure(Goal::Context(expected))
        }
    }
    pub(super) fn activate_context(&mut self, expected: Generation) -> Action {
        let Some(mut replacement) = self.context_prepared.take() else {
            return self.error("Context preparation was canceled; live sources retained");
        };
        if replacement.expected != expected || self.context_generation != expected {
            return self.error("Context preparation is stale; live sources retained");
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
        if let Some(path) = root {
            let current = std::fs::metadata(path.as_path());
            if !current.is_ok_and(|metadata| {
                metadata.is_dir()
                    && Some((metadata.dev(), metadata.ino())) == replacement.root_identity
            }) {
                return self.error("Prepared directory changed; live sources retained");
            }
        }
        if !replacement
            .workspace_observations
            .iter()
            .all(|(path, expected)| {
                expected.is_none_or(|expected| projects::identity_matches(path.as_path(), expected))
            })
        {
            return self.error("Prepared Workspace root changed; live sources retained");
        }
        let bookmarks = match self.buffers.context_bookmarks() {
            Ok(bookmarks) => bookmarks,
            Err(error) => return self.error(&error.to_string()),
        };
        let generation = match expected.next() {
            Ok(value) => value,
            Err(error) => return self.error(&error.to_string()),
        };
        // All fallible admission is complete. Existing owners retire their own
        // requests; no reply channel or source handle reaches the replacement.
        self.observation.wake.arm(false);
        let lsp_enabled = self.lsp.enabled();
        self.dismiss_completion();
        self.ai.stop();
        self.shell.stop();
        self.installation.cancel();
        self.lsp.set_enabled(false);
        self.org.cancel();
        self.end_timer_session();
        let destination = &mut replacement.session;
        destination.lsp.set_enabled(lsp_enabled);
        *self
            .buffers
            .bookmarks
            .lock()
            .expect("bookmark owner poisoned") = bookmarks;
        destination
            .buffers
            .inherit_bookmarks(self.buffers.bookmarks.clone());
        destination.input.inherit_histories(&mut self.input);
        std::mem::swap(&mut destination.kills, &mut self.kills);
        std::mem::swap(&mut destination.rectangle, &mut self.rectangle);
        std::mem::swap(
            &mut destination.bookmark_storage,
            &mut self.bookmark_storage,
        );
        std::mem::swap(&mut destination.clock_store, &mut self.clock_store);
        destination.task_clock = self.task_clock;
        destination.org_clock = self.org_clock;
        destination.timer_clock = self.timer_clock;
        destination.mouse_enabled = self.mouse_enabled;
        std::mem::swap(&mut destination.context_storage, &mut self.context_storage);
        destination.context_generation = generation;
        destination.sync_input_context();
        destination.prepare_bookmark_markers();
        let project = matches!(destination.context, Context::Project(_));
        let workspace = matches!(destination.context, Context::Workspace(_));
        let unavailable_roots = destination
            .tree
            .entries()
            .iter()
            .filter(|e| e.depth() == 0 && e.unavailable())
            .count();
        let missing = destination.context_unavailable.len();
        let directory = matches!(destination.context, Context::Directory(_));
        std::mem::swap(self, destination);
        if workspace {
            return self.message(&format!("Workspace opened; {unavailable_roots} unavailable root(s), {missing} unavailable saved file(s) retained; workspace-add-directory adds a root"));
        }
        if project {
            return self.message(&format!(
                "Project opened; {missing} unavailable saved file(s) retained"
            ));
        }
        let message = if directory {
            self.tree
                .diagnostic()
                .unwrap_or("Temporary directory context opened")
        } else {
            "Context closed; standalone editing"
        };
        self.message(message)
    }

    /// Source continuations own the departure token. Once a prompt fails or is
    /// canceled, reclaim the staged context instead of retaining a hidden job.
    pub(super) fn retire_context_preparation(&mut self) {
        self.retire_workspace_membership();
        self.retire_management();
        let Some(replacement) = &self.context_prepared else {
            return;
        };
        let goal = Goal::Context(replacement.expected);
        let pending = match &self.pending {
            Some(Pending::Quit(quit)) => quit.goal() == goal,
            Some(
                Pending::Save {
                    after: AfterSave::Quit(quit),
                    ..
                }
                | Pending::Write {
                    after: AfterSave::Quit(quit),
                    ..
                },
            ) => quit.goal() == goal,
            Some(
                Pending::BookmarkQuit(sources)
                | Pending::TerminalQuit { sources, .. }
                | Pending::GitQuit { sources, .. },
            ) => sources.goal == goal,
            Some(Pending::Clock(prompt)) => {
                prompt.context_departure() == Some(replacement.expected)
            }
            _ => false,
        };
        let terminal = self.terminal_context_goal() == Some(goal);
        let git = self
            .git
            .shutdown
            .as_ref()
            .is_some_and(|sources| sources.goal == goal);
        if !pending && !terminal && !git && !self.context_storage.departing() {
            self.context_prepared = None;
        }
    }
}
