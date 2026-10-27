use std::io;
use std::time::Instant;

use crossterm::event::{self, Event};

use super::{Action, Session};

impl Session {
    pub(super) fn read_input(&mut self) -> io::Result<Action> {
        if let Some(wait) = self.input.remaining(Instant::now())
            && !event::poll(wait)?
        {
            return Ok(self.expire_input(Instant::now()));
        }
        Ok(self.handle(event::read()?))
    }

    pub fn handle(&mut self, event: Event) -> Action {
        self.handle_at(event, Instant::now())
    }

    /// Process timers before an event without consuming or delaying that event.
    pub fn handle_at(&mut self, event: Event, now: Instant) -> Action {
        self.input.set_tree_focus(self.tree.focused());
        let expired = self.expire_input(now);
        if let Event::Resize(width, height) = event {
            return self.resize(width, height);
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
