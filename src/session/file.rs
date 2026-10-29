use std::io;
use std::path::Path;

use super::{Action, Pending, Session};

impl Session {
    /// Failed ingress changes only the error echo. Existing buffers, history,
    /// point, scroll and input context remain available without lossy replacement.
    pub fn load(&mut self, path: &Path) -> io::Result<()> {
        if self.capture_blocks_input() || self.capture_selected() {
            return Err(io::Error::other(
                "Capture owns this view; abort the draft before visiting a file",
            ));
        }
        self.cancel_pointer_capture();
        self.yank = None;
        self.kills.break_chain();
        let result = self.ingress(path);
        if let Err(error) = &result {
            self.error(&error.to_string());
        }
        result
    }

    fn ingress(&mut self, path: &Path) -> io::Result<()> {
        let visit = self.buffers.prepare_visit(path)?;
        let buffer = visit.buffer(&self.buffers).map_err(io::Error::other)?;
        let observed = buffer.file().map(|file| file.path().to_path_buf());
        let mut view = self.windows.selected().clone();
        view.rebind(buffer);
        self.stop_typing().map_err(io::Error::other)?;
        // All recoverable work is complete before registry/view publication.
        let id = self.buffers.publish_visit(visit);
        if self.windows.selected().point.stamp.buffer != id {
            *self.windows.selected_mut() = view;
        }
        self.tree.edit();
        self.input.cancel();
        self.pending = None;
        self.echo_kind = crate::display::Message::Normal;
        self.echo.clear();
        if let Some(path) = observed
            && let Err(error) = self.tree.observe_path(&path)
        {
            self.echo = format!("Opened {}; tree refresh failed: {error}", path.display());
            self.echo_kind = crate::display::Message::Warning;
        }
        Ok(())
    }

    pub(super) fn find_file(&mut self) -> Action {
        self.start_path()
    }

    pub(super) fn argument(&mut self, path: String) -> Action {
        let pending = self.pending.take();
        if pending.as_ref().is_some_and(Pending::collection_argument) {
            return self.collection_argument(pending.expect("admitted collection argument"), &path);
        }
        self.file_argument(pending, path)
    }
    fn file_argument(&mut self, pending: Option<Pending>, path: String) -> Action {
        match pending {
            Some(pending) if pending.org_argument() => self.org_argument(pending, &path),
            Some(Pending::GotoLine { view, source }) => self.goto_argument(view, source, &path),
            Some(Pending::TreeRename(rename)) => self.tree_renamed(rename, &path),
            Some(Pending::TreeCreate(creation)) => self.tree_created(creation, &path),
            Some(Pending::Replace(state)) => self.replace_argument(*state, path),
            Some(Pending::Write { buffer, after }) => {
                self.save_to(buffer, Some(Path::new(&path)), after)
            }
            Some(Pending::Buffer { command, default }) => {
                self.buffer_argument(command, default, &path)
            }
            _ => self.message("No file argument is pending"),
        }
    }
    fn org_argument(&mut self, pending: Pending, draft: &str) -> Action {
        if pending.observation_argument() {
            return self.observation_argument(pending, draft);
        }
        match pending {
            Pending::Refile => self.refile_argument(draft),
            Pending::Timer(prompt) => self.timer_argument(prompt, draft),
            Pending::Clock(prompt) => self.clock_argument(*prompt, draft),
            Pending::Timestamp(prompt) => self.timestamp_argument(*prompt, draft),
            Pending::Estimate(prompt) => self.estimate_argument(*prompt, draft),
            Pending::Task(prompt) => self.task_argument(*prompt, draft),
            Pending::Tags(prompt) => self.tag_argument(*prompt, draft),
            Pending::Property(prompt) => self.property_argument(*prompt, draft),
            _ => unreachable!("qualified Org argument"),
        }
    }
}

impl Pending {
    fn org_argument(&self) -> bool {
        matches!(
            self,
            Self::Refile
                | Self::AgendaSources
                | Self::AgendaQuery(_)
                | Self::AgendaCustom(_)
                | Self::AgendaBulk(_)
                | Self::AgendaDate
                | Self::Habit(_)
                | Self::Timer(_)
                | Self::ClockReport(_)
                | Self::Clock(_)
                | Self::Timestamp(_)
                | Self::Estimate(_)
                | Self::Task(_)
                | Self::Tags(_)
                | Self::Property(_)
        )
    }
}

impl Session {
    fn observation_argument(&mut self, pending: Pending, draft: &str) -> Action {
        match pending {
            Pending::AgendaSources => self.agenda_source_argument(draft),
            Pending::AgendaBulk(stage) => self.agenda_bulk_argument(stage, draft),
            Pending::AgendaCustom(command) => self.agenda_custom_argument(command, draft),
            Pending::AgendaQuery(kind) => self.agenda_query_argument(kind, draft),
            Pending::AgendaDate => self.agenda_date_argument(draft),
            Pending::Habit(prompt) => self.habit_argument(prompt, draft),
            Pending::ClockReport(prompt) => self.clock_report_argument(*prompt, draft),
            _ => unreachable!("qualified observation argument"),
        }
    }
}

impl Pending {
    pub(super) fn observation_argument(&self) -> bool {
        matches!(
            self,
            Self::AgendaSources
                | Self::AgendaQuery(_)
                | Self::AgendaCustom(_)
                | Self::AgendaBulk(_)
                | Self::AgendaDate
                | Self::Habit(_)
                | Self::ClockReport(_)
        )
    }
}

impl Pending {
    fn collection_argument(&self) -> bool {
        matches!(
            self,
            Self::Capture(_)
                | Self::Link(_)
                | Self::Attachment
                | Self::Document(_)
                | Self::Table(_)
                | Self::TableFile(_)
                | Self::Export(_)
        )
    }
}
impl Session {
    fn collection_argument(&mut self, pending: Pending, draft: &str) -> Action {
        match pending {
            Pending::Export(prompt) => self.export_argument(*prompt, draft),
            Pending::Table(request) => self.table_argument(*request, draft),
            Pending::TableFile(prompt) => self.table_file_argument(*prompt, draft),
            Pending::Document(prompt) => self.document_argument(*prompt, draft),
            Pending::Capture(prompt) => self.capture_argument(prompt, draft),
            Pending::Link(prompt) => self.link_prompt_argument(*prompt, draft),
            Pending::Attachment => self.attachment_argument(draft),
            _ => unreachable!("qualified collection argument"),
        }
    }
}
