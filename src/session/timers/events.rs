use super::*;
use crate::display::Frame;
use std::time::{Duration, Instant};
impl Session {
    pub(in crate::session) fn timer_remaining(&self, now: Instant) -> Option<Duration> {
        self.timer
            .deadline()
            .map(|due| due.saturating_duration_since(now))
    }
    pub(in crate::session) fn expire_timer(&mut self, now: Instant) -> Action {
        match self.timer.expire(now) {
            Ok(false) => Action::Wait,
            Ok(true) => {
                self.timer_notice = true;
                if self.input.focus() == Focus::Editing {
                    self.message("Org countdown finished");
                }
                Action::Redraw
            }
            Err(error) => {
                self.end_timer_session();
                self.error(&error)
            }
        }
    }
    pub(in crate::session) fn timer_frame(
        &self,
        frame: &mut Frame,
    ) -> Result<(), crate::view::Error> {
        let label = if self.timer_notice {
            Some("<Timer done>".into())
        } else {
            self.timer_label()
        };
        if let Some(label) = label {
            frame.timer_status(&format!("{} {label}", self.buffer()?.mode()))?;
        }
        Ok(())
    }
    fn timer_label(&self) -> Option<String> {
        match self.timer.reading(self.timer_now()) {
            Ok(Some(reading)) => Some(format!(
                "<T{}{}{}>",
                if reading.mode == TimerMode::Relative {
                    "+"
                } else {
                    "-"
                },
                reading.format(),
                if reading.paused { " paused" } else { "" }
            )),
            Ok(None) => None,
            Err(_) => Some("<Timer clock error>".into()),
        }
    }
    pub(in crate::session) fn end_timer_session(&mut self) {
        self.timer = crate::org::time::Timer::default();
        self.timer_notice = false;
        if matches!(self.pending, Some(super::super::Pending::Timer(_))) {
            self.pending = None;
            self.input.cancel();
        }
    }
}
