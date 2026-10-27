use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};

use super::{Action, Session};

const INPUT_BATCH_EVENTS: usize = 32;
const INPUT_BATCH_TIME: Duration = Duration::from_millis(4);

impl Session {
    pub(super) fn read_input(&mut self) -> io::Result<Action> {
        if let Some(wait) = self.input.remaining(Instant::now())
            && !event::poll(wait)?
        {
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

    /// Process timers before an event without consuming or delaying that event.
    pub fn handle_at(&mut self, event: Event, now: Instant) -> Action {
        self.input.set_tree_focus(self.tree.focused());
        let expired = self.expire_input(now);
        if let Event::Resize(width, height) = event {
            let action = self.resize(width, height);
            self.input.set_tree_focus(self.tree.focused());
            return action;
        }
        let outcome = self.input.handle_at(event, now);
        let action = self.outcome(outcome);
        if action == Action::Wait {
            return expired;
        }
        action
    }

    /// Present an expired Escape once when a native host's deadline becomes due.
    pub fn expire_input(&mut self, now: Instant) -> Action {
        let outcome = self.input.expire(now);
        self.outcome(outcome)
    }
}
