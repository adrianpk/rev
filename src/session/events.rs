use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};

use super::{Action, Session};

const INPUT_BATCH_EVENTS: usize = 32;
const INPUT_BATCH_TIME: Duration = Duration::from_millis(4);

impl Session {
    pub(super) fn read_input(&mut self) -> io::Result<Action> {
        if !self.input_ready(Instant::now(), event::poll)? {
            return Ok(self.expire_input(Instant::now()));
        }
        let event = event::read()?;
        let started = Instant::now();
        let action = self.handle_at(event, started);
        self.drain_input(action, started, || {
            if event::poll(Duration::ZERO)? {
                event::read().map(Some)
            } else {
                Ok(None)
            }
        })
    }

    /// The native owner blocks indefinitely without deadlines or ready input.
    /// No periodic wake is introduced by an armed click or an inside drag.
    pub fn remaining_input(&self, now: Instant) -> Option<Duration> {
        [
            self.input.remaining(now),
            self.pointer_remaining(now),
            self.org.remaining(now),
            self.timer_remaining(now),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    fn input_ready(
        &self,
        now: Instant,
        mut poll: impl FnMut(Duration) -> io::Result<bool>,
    ) -> io::Result<bool> {
        let Some(wait) = self.remaining_input(now) else {
            return Ok(true);
        };
        if poll(wait)? {
            return Ok(true);
        }
        // Recheck native input after timeout: queued release/cancellation wins
        // over due pointer work, even when it arrives at the deadline boundary.
        poll(Duration::ZERO)
    }

    // Keep every handler transition in order. Only intermediate presentation is
    // combined; finite event/time budgets guarantee progress to the next frame.
    fn drain_input(
        &mut self,
        mut action: Action,
        started: Instant,
        mut next: impl FnMut() -> io::Result<Option<Event>>,
    ) -> io::Result<Action> {
        for _ in 1..INPUT_BATCH_EVENTS {
            if action == Action::Exit || started.elapsed() >= INPUT_BATCH_TIME {
                break;
            }
            let Some(event) = next()? else {
                break;
            };
            match self.handle(event) {
                Action::Wait => {}
                outcome => action = outcome,
            }
        }
        Ok(action)
    }

    pub fn handle(&mut self, event: Event) -> Action {
        self.handle_at(event, Instant::now())
    }

    /// Expire Escape before dispatch; pointer deadlines wait until input is drained.
    pub fn handle_at(&mut self, event: Event, now: Instant) -> Action {
        self.timer_notice = false;
        if !matches!(event, Event::Mouse(_)) {
            self.cancel_pointer_capture();
        }
        self.sync_input_context();
        let expired = self.expire_escape(now);
        if let Event::Resize(width, height) = event {
            let action = self.resize(width, height);
            self.sync_input_context();
            return action;
        }
        if let Event::Mouse(mouse) = event
            && let Some(action) = self.timestamp_pointer(mouse)
        {
            return action;
        }
        let outcome = self.input.handle_at(event, now);
        let action = self.outcome(outcome, now);
        if action == Action::Wait {
            return expired;
        }
        action
    }

    /// Service native deadlines after the host has processed ready input.
    /// A delayed wake performs one pointer step, never catch-up work.
    pub fn expire_input(&mut self, now: Instant) -> Action {
        [
            self.expire_escape(now),
            self.expire_org(now),
            self.expire_pointer(now),
            self.expire_timer(now),
        ]
        .into_iter()
        .rev()
        .find(|a| *a != Action::Wait)
        .unwrap_or(Action::Wait)
    }

    fn expire_escape(&mut self, now: Instant) -> Action {
        let outcome = self.input.expire(now);
        self.outcome(outcome, now)
    }
}

#[cfg(test)]
mod tests;
