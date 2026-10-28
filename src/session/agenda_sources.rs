use super::{Action, Pending, Session};
use crate::{
    command::Command,
    org::agenda::{SourceScan, SourceSelection, SourceSet},
};
impl Session {
    /// Last explicit source observation; derived text is never a canonical edit
    /// target. Callers must refresh/revalidate before using retained locators.
    pub fn agenda_sources(&self) -> Option<&SourceSet> {
        self.agenda_sources.as_ref()
    }
    pub(super) fn agenda_command(&mut self) -> Action {
        if !self.buffers.snapshot().settings().org.enabled {
            return self.message("Native Org mode is disabled");
        }
        if self.org.sources_running
            || self.org.sources_pending.is_some()
            || self.org.request.is_some()
            || self.org.report.is_some()
        {
            return self.message("Org observation pending; C-g cancels");
        }
        let selection = match SourceSelection::configured(self.buffers.snapshot()) {
            Ok(selection) => selection,
            Err(error) => return self.error(&error),
        };
        if selection.paths.is_empty() {
            return self.prompt_agenda_sources("");
        }
        self.queue_agenda_sources(selection)
    }
    fn prompt_agenda_sources(&mut self, draft: &str) -> Action {
        if let Err(error) = self
            .input
            .argument("Agenda source [current/buffers/path]: ", draft)
        {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::AgendaSources);
        self.message(
            "Select current, buffers or one literal file/directory; no implicit scan; C-g cancels",
        )
    }
    pub(super) fn agenda_source_argument(&mut self, draft: &str) -> Action {
        let selection = match draft.trim() {
            "current" => Ok(SourceSelection {
                paths: Vec::new(),
                buffers: vec![self.view().point.stamp.buffer],
            }),
            "buffers" => self.select_agenda_buffers(),
            path => SourceSelection::paths(&[path.to_owned()]),
        };
        match selection {
            Ok(selection) => self.queue_agenda_sources(selection),
            Err(error) => {
                self.prompt_agenda_sources(draft);
                self.error(&error)
            }
        }
    }
    fn select_agenda_buffers(&self) -> Result<SourceSelection, String> {
        let ids: Vec<_> = self
            .buffers
            .iter()
            .filter(|b| b.is_org())
            .take(4097)
            .map(|b| b.id())
            .collect();
        if ids.is_empty() || ids.len() > 4096 {
            return Err("Select 1..4096 eligible native Org buffers".into());
        }
        Ok(SourceSelection {
            paths: Vec::new(),
            buffers: ids,
        })
    }
    fn queue_agenda_sources(&mut self, selection: SourceSelection) -> Action {
        self.org.sources_pending = Some(selection);
        self.refresh_org();
        // A worker can finish during subsequent normal editing. It never owns
        // input focus or starts another worker while the current slot is busy.
        if self.org.sources_pending.is_some() || self.org.sources_running {
            self.message("Org agenda sources pending; C-g cancels; editing remains available")
        } else {
            Action::Redraw
        }
    }
    pub(super) fn launch_agenda_sources(&mut self) -> bool {
        let Some(selection) = self.org.sources_pending.take() else {
            return false;
        };
        let result = SourceScan::new(
            &self.buffers,
            selection,
            &self.buffers.snapshot().settings().org.limits,
        )
        .and_then(|scan| self.org.start_sources(self.view().point.stamp, scan));
        if let Err(error) = result {
            self.error(&error);
        }
        true
    }
    pub(super) fn cancel_agenda_discovery(&mut self, command: Command) -> Option<Action> {
        if command != Command::KeyboardQuit
            || !(self.org.sources_running || self.org.sources_pending.is_some())
        {
            return None;
        }
        self.org.cancel();
        self.input.cancel();
        self.pending = None;
        if let Some(result) = &mut self.agenda_sources {
            result.cancelled = true;
        }
        Some(self.message("Org agenda source discovery cancelled; result incomplete"))
    }
}
