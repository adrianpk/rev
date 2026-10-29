mod control;
mod frame;
use super::{Action, Pending, Session};
use crate::{
    command::{Command, OrgCommand as C},
    org::agenda::{AgendaQuery, QueryKind, QueryResult, SourceSelection},
    window::ViewId,
};
pub(super) struct State {
    pub result: QueryResult,
    pub selection: SourceSelection,
    pub view: ViewId,
    pub cursor: usize,
    pub top: usize,
}
impl Session {
    pub fn query_agenda(&self) -> Option<&QueryResult> {
        self.query_agenda.as_ref().map(|s| &s.result)
    }
    pub(super) fn query_active(&self) -> bool {
        self.query_agenda
            .as_ref()
            .is_some_and(|s| s.view == self.windows.selected_id())
    }
    pub(super) fn agenda_active(&self) -> bool {
        self.calendar_active() || self.query_active()
    }
    pub(super) fn install_query(
        &mut self,
        result: QueryResult,
        selection: SourceSelection,
        view: ViewId,
    ) {
        if self.windows.get(view).is_err() {
            self.message("Agenda view closed; query observations retained");
            return;
        }
        self.query_agenda = Some(State {
            result,
            selection,
            view,
            cursor: 0,
            top: 0,
        });
        self.calendar_agenda = None;
        self.habit_agenda = None;
        self.sync_input_context();
        self.message("Query agenda: g refresh; RET source; t TODO; s search; m match; q close");
    }
    pub(super) fn prompt_agenda_query(&mut self, command: C, draft: &str) -> Action {
        if !self.buffers.snapshot().settings().org.enabled {
            return self.message("Native Org mode is disabled");
        }
        if self.org.busy() {
            return self.message("Org observation pending; C-g cancels");
        }
        let kind = match command {
            C::TodoList => QueryKind::Todo,
            C::SearchView => QueryKind::Search,
            C::TagsView => QueryKind::Tags,
            _ => unreachable!("query command"),
        };
        self.query_prompt(kind, draft)
    }
    fn query_prompt(&mut self, kind: QueryKind, draft: &str) -> Action {
        let prompt = match kind {
            QueryKind::Todo => "TODO keywords [empty: unfinished; | separates]: ",
            QueryKind::Search => "Search [phrase or +Boolean terms]: ",
            QueryKind::Tags => "Tag/property match: ",
        };
        if let Err(e) = self.input.argument(prompt, draft) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::AgendaQuery(kind));
        self.message("Enter a typed query; C-g keeps the source or agenda")
    }
    pub(super) fn agenda_query_argument(&mut self, kind: QueryKind, draft: &str) -> Action {
        let query =
            match AgendaQuery::parse(kind, draft, &self.buffers.snapshot().settings().org.limits) {
                Ok(query) => query,
                Err(e) => {
                    self.query_prompt(kind, draft);
                    return self.error(&e);
                }
            };
        self.agenda_query_plan = Some(query);
        self.agenda_custom_plan = None;
        let selection = self
            .query_agenda
            .as_ref()
            .map(|s| s.selection.clone())
            .or_else(|| self.calendar_agenda.as_ref().map(|s| s.selection.clone()));
        if let Some(selection) = selection {
            return self.queue_agenda_sources(selection);
        }
        self.agenda_command()
    }
    fn refresh_query(&mut self) -> Action {
        let state = self.query_agenda.as_ref().expect("query owner");
        if self.org.busy() {
            return self.message("Org observation pending; C-g cancels");
        }
        if let Some(view) = &state.result.view {
            self.agenda_custom_plan = Some(view.clone());
        } else {
            self.agenda_query_plan = Some(state.result.query.clone());
        }
        self.queue_agenda_sources(state.selection.clone())
    }
}
