mod control;
mod options;
use super::{Action, Pending, Session};
use crate::{
    command::OrgCommand as C,
    config::{AgendaBlock, AgendaKind, AgendaView, Org},
    org::{
        Cancellation,
        agenda::{CalendarQuery, CustomView, QueryKind, SourceScan, SourceSelection, SourceSet},
    },
    window::ViewId,
};
pub(super) struct Request {
    pub scan: Option<SourceScan>,
    pub sources: Option<SourceSet>,
    pub definition: AgendaView,
    pub baseline: AgendaView,
    pub context: CalendarQuery,
    pub policy: Org,
    pub selection: SourceSelection,
    pub view: ViewId,
}
impl Request {
    pub(super) fn run(self, cancel: &Cancellation) -> Result<super::agenda::Output, String> {
        let bounded = cancel.bounded(
            std::time::Instant::now()
                + std::time::Duration::from_secs(self.policy.limits.job_seconds as u64),
        );
        let sources = match self.scan {
            Some(scan) => scan.run(&bounded),
            None => self
                .sources
                .ok_or("Custom view requires retained sources")?,
        };
        let mut result = CustomView::observe(
            &sources,
            self.definition,
            self.context,
            &self.policy,
            &bounded,
        )?;
        result.baseline = Some(self.baseline);
        Ok(super::agenda::Output::Query {
            sources,
            result: Box::new(result),
            selection: self.selection,
            view: self.view,
        })
    }
}
impl Session {
    pub(super) fn agenda_presentation_command(
        &mut self,
        command: crate::command::Command,
    ) -> Option<Action> {
        match command {
            crate::command::Command::Org(c) if c.agenda_custom() => Some(self.custom_command(c)),
            _ => None,
        }
    }

    pub(super) fn custom_command(&mut self, command: C) -> Action {
        if !self.buffers.snapshot().settings().org.enabled {
            return self.message("Native Org mode is disabled");
        }
        if self.org.busy() {
            return self.message("Org observation pending; C-g cancels");
        }
        if command == C::AgendaCustom {
            return self.custom_prompt(command, "");
        }
        if !self.agenda_active() {
            return self.message("Open an agenda before changing its presentation");
        }
        match command {
            C::AgendaFilterReset => self.change_custom(command, ""),
            C::AgendaMonth => self.change_custom(C::AgendaSpan, "month"),
            C::AgendaYear => self.change_custom(C::AgendaSpan, "year"),
            _ => self.custom_prompt(command, ""),
        }
    }
    fn custom_prompt(&mut self, command: C, draft: &str) -> Action {
        let prompt = match command {
            C::AgendaCustom => "Agenda view key: ",
            C::AgendaFilter => "Agenda tag/property filter [empty: reset]: ",
            C::AgendaSort => "Agenda sort keys [comma separated]: ",
            C::AgendaColumns => "Agenda property columns [comma separated]: ",
            C::AgendaSpan => "Agenda span [day/week/month/year/1..366]: ",
            C::AgendaDate => "Agenda date [YYYY-MM-DD/today]: ",
            _ => unreachable!("custom prompt"),
        };
        if let Err(e) = self.input.argument(prompt, draft) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::AgendaCustom(command));
        if command == C::AgendaCustom {
            let choices = self
                .buffers
                .snapshot()
                .settings()
                .org
                .agenda
                .views
                .iter()
                .map(|v| format!("{}: {}", v.key, v.title))
                .collect::<Vec<_>>()
                .join("; ");
            return self.message(&format!(
                "Configured views [{choices}]; C-g keeps the current view"
            ));
        }
        self.message("Typed agenda presentation; C-g keeps the current view")
    }
    pub(super) fn agenda_custom_argument(&mut self, command: C, draft: &str) -> Action {
        if command == C::AgendaCustom {
            return self.select_custom(draft);
        }
        self.change_custom(command, draft)
    }
    fn select_custom(&mut self, key: &str) -> Action {
        let policy = &self.buffers.snapshot().settings().org;
        let Some(mut view) = policy
            .agenda
            .views
            .iter()
            .find(|v| v.key == key.trim())
            .cloned()
        else {
            self.custom_prompt(C::AgendaCustom, key);
            return self.error("Unknown agenda view key");
        };
        self.resolve_custom_paths(&mut view);
        let inherited = match SourceSelection::configured(self.buffers.snapshot()) {
            Ok(selection) => selection
                .paths
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            Err(e) => return self.error(&e),
        };
        for block in &mut view.blocks {
            if block.sources.is_empty() {
                block.sources = inherited.clone();
            }
        }
        self.agenda_custom_plan = Some(view);
        self.agenda_query_plan = None;
        self.agenda_command()
    }
    fn resolve_custom_paths(&self, view: &mut AgendaView) {
        let Some(root) = self
            .buffers
            .snapshot()
            .origin(&["org", "agenda", "views"])
            .and_then(|p| p.parent())
        else {
            return;
        };
        for block in &mut view.blocks {
            for path in &mut block.sources {
                let p = std::path::Path::new(path);
                if p.is_relative() {
                    *path = root.join(p).to_string_lossy().into_owned();
                }
            }
        }
    }
    fn custom_definition(
        &self,
    ) -> Result<(AgendaView, AgendaView, CalendarQuery, SourceSelection), String> {
        if let Some(s) = &self.query_agenda {
            let definition = s
                .result
                .view
                .clone()
                .unwrap_or_else(|| single(&s.result.query));
            let baseline = s
                .result
                .baseline
                .clone()
                .unwrap_or_else(|| definition.clone());
            return Ok((
                definition,
                baseline,
                s.result.context.clone(),
                s.selection.clone(),
            ));
        }
        let s = self
            .calendar_agenda
            .as_ref()
            .ok_or("Open an agenda first")?;
        let view = AgendaView {
            key: "calendar".into(),
            title: "Calendar".into(),
            blocks: vec![AgendaBlock {
                kind: AgendaKind::Calendar,
                ..AgendaBlock::default()
            }],
        };
        Ok((
            view.clone(),
            view,
            s.calendar.query.clone(),
            s.selection.clone(),
        ))
    }
}
fn single(query: &crate::org::agenda::AgendaQuery) -> AgendaView {
    let kind = match query.kind {
        QueryKind::Todo => AgendaKind::Todo,
        QueryKind::Search => AgendaKind::Search,
        QueryKind::Tags => AgendaKind::Tags,
    };
    AgendaView {
        key: "query".into(),
        title: query.kind.label().into(),
        blocks: vec![AgendaBlock {
            kind,
            query: query.source.clone(),
            ..AgendaBlock::default()
        }],
    }
}
