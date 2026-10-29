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
        let mut selection = match SourceSelection::configured(self.buffers.snapshot()) {
            Ok(selection) => selection,
            Err(error) => return self.error(&error),
        };
        if let Some(view) = &self.agenda_custom_plan {
            for block in &view.blocks {
                selection
                    .paths
                    .extend(block.sources.iter().map(std::path::PathBuf::from));
            }
            selection.paths.sort();
            selection.paths.dedup();
        }
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
    pub(super) fn queue_agenda_sources(&mut self, selection: SourceSelection) -> Action {
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
        let result = self
            .prepare_agenda_refresh(selection)
            .and_then(|work| self.org.start_sources(self.view().point.stamp, work));
        self.agenda_query_plan = None;
        self.agenda_custom_plan = None;
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
        if let Some(state) = &mut self.calendar_agenda {
            state.calendar.cancelled = true;
        }
        if let Some(state) = &mut self.query_agenda {
            state.result.cancelled = true;
        }
        self.agenda_query_plan = None;
        self.agenda_custom_plan = None;
        Some(self.message("Org agenda source discovery cancelled; result incomplete"))
    }
}

impl Session {
    fn prepare_agenda_refresh(
        &self,
        selection: SourceSelection,
    ) -> Result<super::agenda::Work, String> {
        let policy = &self.buffers.snapshot().settings().org;
        let query = self.agenda_context(policy)?;
        let scan = SourceScan::new(&self.buffers, selection.clone(), &policy.limits)?;
        if let Some(view) = &self.agenda_custom_plan {
            return Ok(super::agenda::Work::Custom(Box::new(
                super::agenda_custom::Request {
                    scan: Some(scan),
                    sources: None,
                    definition: view.clone(),
                    baseline: self
                        .query_agenda
                        .as_ref()
                        .and_then(|s| s.result.baseline.as_ref())
                        .filter(|base| base.key == view.key)
                        .cloned()
                        .unwrap_or_else(|| view.clone()),
                    context: query,
                    policy: policy.clone(),
                    selection,
                    view: self.windows.selected_id(),
                },
            )));
        }
        if let Some(plan) = &self.agenda_query_plan {
            return Ok(super::agenda::Work::Query {
                scan: Box::new(scan),
                query: plan.clone(),
                context: query,
                policy: Box::new(policy.clone()),
                selection,
                view: self.windows.selected_id(),
            });
        }
        Ok(super::agenda::Work::Refresh {
            scan: Box::new(scan),
            selection,
            query,
            policy: Box::new(policy.clone()),
            view: self.windows.selected_id(),
        })
    }
    fn agenda_context(
        &self,
        policy: &crate::config::Org,
    ) -> Result<crate::org::agenda::CalendarQuery, String> {
        let operation = self
            .org_clock
            .unwrap_or(super::outline::timestamps::capture)(&policy.time)?;
        let query = crate::org::agenda::CalendarQuery::new(operation, policy)?;
        let old = self
            .calendar_agenda
            .as_ref()
            .map(|s| &s.calendar.query)
            .or_else(|| {
                self.query_agenda
                    .as_ref()
                    .filter(|s| s.result.view.is_some())
                    .map(|s| &s.result.context)
            });
        let Some(old) = old else {
            return Ok(query);
        };
        let showing_today = old.start <= old.today && old.end().is_ok_and(|end| old.today <= end);
        query.at(if showing_today { query.today } else { old.date }, old.span)
    }
    pub(super) fn refresh_calendar(&mut self) -> Action {
        let Some(state) = &self.calendar_agenda else {
            return self.agenda_command();
        };
        if self.org.busy() {
            return self.message("Org observation pending; C-g cancels");
        }
        self.queue_agenda_sources(state.selection.clone())
    }
}
