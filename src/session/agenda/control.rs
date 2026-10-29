use super::*;
use crate::{
    command::{Command, OrgCommand as C, Scroll},
    config::AgendaSpan,
    org::agenda::{CalendarLine, SourceVersion},
    session::Pending,
};
impl Session {
    pub(in crate::session) fn agenda_view_command(&mut self, command: Command) -> Option<Action> {
        if !self.calendar_active() {
            return None;
        }
        match command {
            Command::KeyboardQuit => {
                self.calendar_agenda = None;
                self.sync_input_context();
                Some(self.message("Returned to source"))
            }
            Command::Quit | Command::Window(_) => None,
            Command::Scroll(scroll) => Some(self.calendar_scroll(scroll)),
            Command::Org(c) => self.calendar_org(c),
            _ => Some(self.message("Read-only calendar agenda; RET visits source; q closes")),
        }
    }
    fn calendar_org(&mut self, command: C) -> Option<Action> {
        if command.agenda_query() {
            return Some(self.prompt_agenda_query(command, ""));
        }
        if command.timer_control() {
            return None;
        }
        if command.agenda_control() || command == C::Agenda {
            return Some(self.calendar_control(command));
        }
        Some(self.message("Read-only calendar agenda; RET visits source; q closes"))
    }
    fn calendar_control(&mut self, command: C) -> Action {
        if command == C::AgendaRefresh || command == C::Agenda {
            return self.refresh_calendar();
        }
        if command == C::AgendaVisit {
            return self.calendar_visit();
        }
        if command == C::AgendaDate {
            return self.prompt_agenda_date("");
        }
        let old = &self
            .calendar_agenda
            .as_ref()
            .expect("calendar owner")
            .calendar
            .query;
        let query = match command {
            C::AgendaDay => old.at(old.start, AgendaSpan::Day),
            C::AgendaWeek => old.at(old.start, AgendaSpan::Week),
            C::AgendaNext => old.shifted(1),
            C::AgendaPrevious => old.shifted(-1),
            C::AgendaToday => old.at(old.today, old.span),
            _ => return self.message("Calendar control unavailable"),
        };
        self.project_calendar(query)
    }
    fn project_calendar(&mut self, query: Result<CalendarQuery, String>) -> Action {
        if self.org.busy() {
            return self.message("Org observation pending; C-g cancels");
        }
        let query = match query {
            Ok(q) => q,
            Err(e) => return self.error(&e),
        };
        let state = self.calendar_agenda.as_ref().expect("calendar owner");
        let work = Work::Project {
            projection: state.calendar.projection(query),
            view: state.view,
        };
        match self.org.start_sources(self.view().point.stamp, work) {
            Ok(()) => self.message("Calendar projection pending; C-g cancels"),
            Err(e) => self.error(&e),
        }
    }
    fn calendar_scroll(&mut self, scroll: Scroll) -> Action {
        let height = self.pane_size().height.saturating_sub(1).max(1) as usize;
        let state = self.calendar_agenda.as_mut().expect("calendar owner");
        let distance = if scroll.page() { height } else { 1 };
        match scroll {
            Scroll::NextLine | Scroll::PageForward => {
                state.cursor = state
                    .cursor
                    .saturating_add(distance)
                    .min(state.calendar.lines.len().saturating_sub(1))
            }
            Scroll::PreviousLine | Scroll::PageBackward => {
                state.cursor = state.cursor.saturating_sub(distance)
            }
            _ => (),
        }
        if state.cursor < state.top {
            state.top = state.cursor;
        }
        if state.cursor >= state.top + height {
            state.top = state.cursor + 1 - height;
        }
        Action::Redraw
    }
    fn prompt_agenda_date(&mut self, draft: &str) -> Action {
        if let Err(e) = self.input.argument("Agenda date [YYYY-MM-DD]: ", draft) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::AgendaDate);
        self.message("Enter YYYY-MM-DD or today; C-g keeps the calendar")
    }
    pub(in crate::session) fn agenda_date_argument(&mut self, draft: &str) -> Action {
        let query = &self
            .calendar_agenda
            .as_ref()
            .expect("calendar date prompt owns a view")
            .calendar
            .query;
        let date = if draft.trim() == "today" {
            Ok(query.today)
        } else {
            crate::org::agenda::CalendarQuery::parse_date(draft)
        };
        match date.and_then(|date| query.at(date, query.span)) {
            Ok(query) => self.project_calendar(Ok(query)),
            Err(e) => {
                self.prompt_agenda_date(draft);
                self.error(&e)
            }
        }
    }
    pub(in crate::session) fn cancel_calendar_prompt(
        &mut self,
        command: Command,
    ) -> Option<Action> {
        if command != Command::KeyboardQuit || !self.agenda_active() {
            return None;
        }
        if !matches!(
            self.pending,
            Some(
                Pending::AgendaDate
                    | Pending::AgendaQuery(_)
                    | Pending::AgendaCustom(_)
                    | Pending::AgendaBulk(_)
                    | Pending::Command
                    | Pending::Timer(_)
            )
        ) {
            return None;
        }
        self.pending = None;
        self.input.cancel();
        Some(self.message("Calendar prompt cancelled"))
    }
    fn calendar_visit(&mut self) -> Action {
        match self.prepare_calendar_visit() {
            Ok(()) => Action::Redraw,
            Err(e) => self.error(&e),
        }
    }
    fn prepare_calendar_visit(&mut self) -> Result<(), String> {
        if self.org.busy() {
            return Err("Org observation pending; C-g cancels".into());
        }
        let state = self.calendar_agenda.as_ref().expect("calendar owner");
        let CalendarLine::Entry(index) = state
            .calendar
            .lines
            .get(state.cursor)
            .ok_or("Select an agenda entry")?
        else {
            return Err("Select an agenda entry".into());
        };
        let row = &state.calendar.rows[*index];
        let origin = &state.calendar.sources[row.source];
        if let SourceVersion::Buffer(stamp) = origin.version {
            return self.visit_calendar_buffer(stamp, row.byte);
        }
        let navigation = state.calendar.disk_navigation(
            *index,
            self.agenda_sources
                .as_ref()
                .ok_or("Refresh sources first")?,
            &self.buffers.snapshot().settings().org.limits,
        )?;
        self.org.start_sources(
            self.view().point.stamp,
            Work::Visit {
                navigation,
                view: state.view,
            },
        )?;
        self.message("Checking agenda source; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn visit_calendar_buffer(
        &mut self,
        stamp: crate::buffer::Stamp,
        byte: usize,
    ) -> Result<(), String> {
        let buffer = self.buffers.get(stamp.buffer).map_err(|e| e.to_string())?;
        if buffer.stamp() != stamp {
            return Err("Agenda source changed; refresh required".into());
        }
        let position = buffer
            .byte_span(byte..byte)
            .map_err(|e| e.to_string())?
            .range
            .start;
        let point = crate::buffer::edit::Point {
            stamp,
            position: crate::coordinates::Scalar(position),
        };
        self.calendar_agenda = None;
        self.query_agenda = None;
        self.activate(stamp.buffer);
        self.place_point(point).map_err(|e| e.to_string())?;
        self.sync_input_context();
        self.message("Agenda source");
        Ok(())
    }
}
