mod events;
mod prompt;
mod source;
use super::{Action, Session};
use crate::{
    command::OrgCommand,
    input::Focus,
    org::time::{TimerMode, TimerReading},
};
pub(super) use prompt::Prompt;
impl Session {
    /// Inject a monotonic observation for an owned native host or fixture.
    pub fn with_timer_clock(mut self, clock: fn() -> std::time::Instant) -> Self {
        self.timer_clock = Some(clock);
        self
    }
    fn timer_now(&self) -> std::time::Instant {
        self.timer_clock.unwrap_or(std::time::Instant::now)()
    }
    pub(super) fn timer_command(&mut self, command: OrgCommand) -> Action {
        if self.input.focus() != Focus::Editing || self.tree.focused() {
            return self.message("Editing owns timer commands");
        }
        self.timer_notice = false;
        if command == OrgCommand::TimerSet {
            return self.open_countdown();
        }
        let now = self.timer_now();
        let result = match command {
            OrgCommand::TimerStart => self
                .timer
                .start(now)
                .map(|_| "Org relative timer started at 0:00:00".into()),
            OrgCommand::TimerStop => self.timer.stop().map(|_| "Org timer stopped".into()),
            OrgCommand::TimerPause => self.timer.pause_or_continue(now).map(feedback),
            OrgCommand::TimerShow => self
                .timer
                .reading(now)
                .map(|v| v.map(feedback).unwrap_or("No Org timer is running".into())),
            _ => unreachable!("qualified timer control"),
        };
        match result {
            Ok(text) => self.message(&text),
            Err(error) => self.error(&error),
        }
    }
}
fn feedback(reading: TimerReading) -> String {
    let mode = match reading.mode {
        TimerMode::Relative => "relative",
        TimerMode::Countdown => "countdown",
    };
    format!(
        "Org {mode} timer {}: {}",
        if reading.paused { "paused" } else { "running" },
        reading.format()
    )
}
