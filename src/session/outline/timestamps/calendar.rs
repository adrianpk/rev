use super::*;
use crate::command::{Command, OrgCommand as C};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
impl Session {
    pub(in crate::session) fn calendar_command(&mut self, command: C) -> Action {
        let Some(Pending::Timestamp(prompt)) = &self.pending else {
            return self.message("Open the Org date/calendar prompt first");
        };
        if !matches!(prompt.stage, Stage::Date) {
            return self.message("The UTC offset prompt owns input focus");
        }
        if !self.org_source_current(&prompt.request) {
            self.pending = None;
            self.input.cancel();
            return self.message("Org timestamp source or view changed; request cancelled");
        }
        if command == C::CalendarSelect {
            let text = self
                .input
                .prompt()
                .expect("calendar input")
                .text()
                .to_owned();
            self.input.cancel();
            return self.argument(text);
        }
        let date = shifted(prompt, command);
        match date {
            Ok(date) => self.calendar_selected(date),
            Err(error) => self.error(&error),
        }
    }
    fn calendar_selected(&mut self, date: Date) -> Action {
        let Some(Pending::Timestamp(prompt)) = &mut self.pending else {
            return Action::Wait;
        };
        prompt.value.start.set_date(date);
        let text = prompt.value.format();
        match self.input.complete(&text) {
            Ok(()) => self.input_changed(),
            Err(error) => self.error(&error.to_string()),
        }
    }
    pub(in crate::session) fn timestamp_pointer(&mut self, event: MouseEvent) -> Option<Action> {
        let date = self.timestamp_calendar()?;
        if !self.mouse_enabled || event.kind != MouseEventKind::Down(MouseButton::Left) {
            return Some(Action::Wait);
        }
        let selected = self
            .composition()
            .and_then(|c| c.auxiliary())
            .and_then(|p| {
                let area = p.content();
                let column = event.column.checked_sub(area.column())?;
                let row = event.row.checked_sub(area.row())?;
                if row >= area.height() {
                    return None;
                }
                crate::org::time::calendar_hit(date, (column, row), area.width())
            });
        Some(selected.map_or(Action::Wait, |date| self.calendar_selected(date)))
    }
    pub(in crate::session) fn timestamp_command(&mut self, command: Command) -> Option<Action> {
        let Command::Org(command) = command else {
            return None;
        };
        command.calendar().then(|| self.calendar_command(command))
    }
}
fn shifted(prompt: &Prompt, command: C) -> Result<Date, String> {
    use crate::org::time::{CalendarInterval as Shift, CivilDateTime};
    const SHIFTS: [(C, Shift); 10] = [
        (C::CalendarForwardDay, Shift::Days(1)),
        (C::CalendarBackwardDay, Shift::Days(-1)),
        (C::CalendarForwardWeek, Shift::Weeks(1)),
        (C::CalendarBackwardWeek, Shift::Weeks(-1)),
        (C::CalendarForwardMonth, Shift::Months(1)),
        (C::CalendarBackwardMonth, Shift::Months(-1)),
        (C::CalendarForwardYear, Shift::Months(12)),
        (C::CalendarBackwardYear, Shift::Months(-12)),
        (C::CalendarScrollThreeMonthsLeft, Shift::Months(3)),
        (C::CalendarScrollThreeMonthsRight, Shift::Months(-3)),
    ];
    if command == C::CalendarGotoToday {
        return Ok(prompt.operation.now().date);
    }
    let shift = SHIFTS
        .iter()
        .find(|(c, _)| *c == command)
        .ok_or("Not an Org calendar navigation action")?
        .1;
    let civil = CivilDateTime {
        date: prompt.value.start.date(),
        time: prompt.operation.now().time,
    };
    if let Shift::Months(count) = shift {
        return crate::org::time::Month::new(civil.date, count)?.selected_day(civil.date.day());
    }
    civil.shift(shift).map(|c| c.date)
}
