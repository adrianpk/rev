use super::*;
use crate::{
    command::Scroll,
    org::agenda::{Navigation, SourceVersion},
};
impl Session {
    pub(in crate::session) fn query_view_command(&mut self, command: Command) -> Option<Action> {
        if !self.query_active() {
            return None;
        }
        match command {
            Command::KeyboardQuit => {
                self.query_agenda = None;
                self.sync_input_context();
                Some(self.message("Returned to source"))
            }
            Command::Window(_) | Command::Quit => None,
            Command::Scroll(scroll) => Some(self.query_scroll(scroll)),
            Command::Org(c) => self.query_control(c),
            _ => Some(self.message("Read-only query agenda; RET source; q closes")),
        }
    }
    fn query_control(&mut self, command: C) -> Option<Action> {
        if command.timer_control() {
            return None;
        }
        if command.agenda_query() {
            return Some(self.prompt_agenda_query(command, ""));
        }
        let action = match command {
            C::AgendaRefresh | C::Agenda => self.refresh_query(),
            C::AgendaVisit => match self.visit_query() {
                Ok(()) => Action::Redraw,
                Err(e) => self.error(&e),
            },
            _ => self.message("Read-only query agenda; g refresh; RET source; q close"),
        };
        Some(action)
    }
    fn query_scroll(&mut self, scroll: Scroll) -> Action {
        let height = self.pane_size().height.saturating_sub(2).max(1) as usize;
        let s = self.query_agenda.as_mut().expect("query owner");
        let distance = if scroll.page() { height } else { 1 };
        let count = s.result.rows.len() + s.result.issues.len();
        match scroll {
            Scroll::NextLine | Scroll::PageForward => {
                s.cursor = s
                    .cursor
                    .saturating_add(distance)
                    .min(count.saturating_sub(1))
            }
            Scroll::PreviousLine | Scroll::PageBackward => {
                s.cursor = s.cursor.saturating_sub(distance)
            }
            _ => (),
        }
        if s.cursor < s.top {
            s.top = s.cursor;
        }
        if s.cursor >= s.top + height {
            s.top = s.cursor + 1 - height;
        }
        Action::Redraw
    }
    fn visit_query(&mut self) -> Result<(), String> {
        if self.org.busy() {
            return Err("Org observation pending; C-g cancels".into());
        }
        let state = self.query_agenda.as_ref().expect("query owner");
        let row = state
            .result
            .rows
            .get(state.cursor)
            .ok_or("Select a query result")?;
        let origin = &state.result.sources[row.source];
        if let SourceVersion::Buffer(stamp) = origin.version {
            return self.visit_calendar_buffer(stamp, row.byte);
        }
        let navigation = Navigation::prepare(
            origin,
            row.byte,
            self.agenda_sources
                .as_ref()
                .ok_or("Refresh sources first")?,
            &self.buffers.snapshot().settings().org.limits,
        )?;
        self.org.start_sources(
            self.view().point.stamp,
            super::super::agenda::Work::Visit {
                navigation,
                view: state.view,
            },
        )?;
        self.message("Checking query source; C-g cancels");
        Ok(())
    }
}
