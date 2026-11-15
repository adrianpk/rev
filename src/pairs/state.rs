use std::time::Instant;

use ropey::RopeSlice;

use super::{Answer, Hint, Profile, Query, Stats, Status, Ticket, work::Work};
use crate::buffer::Stamp;

/// One operation owner per canonical buffer, independent of its view count.
#[derive(Default)]
pub(crate) struct State {
    generation: u64,
    work: Option<Box<Work>>,
}

impl State {
    pub fn retire(&mut self) {
        self.work = None;
        self.generation = self
            .generation
            .checked_add(1)
            .expect("pair generation exhausted");
    }

    pub fn start(&mut self, query: Query, source: (Stamp, Profile), hint: Option<Hint>) -> Ticket {
        let previous = self.work.take();
        self.retire();
        let ticket = Ticket {
            stamp: source.0,
            generation: self.generation,
        };
        self.work = Some(match previous.filter(|work| work.reusable(source)) {
            Some(mut work) => {
                work.begin(ticket, query, hint);
                work
            }
            None => Box::new(Work::new(ticket, query, source.1, hint)),
        });
        ticket
    }

    /// Reuse only a prefix whose complete lookahead predates the first change.
    /// Bounded recent checkpoints support nearby deletions; old tickets retire.
    pub fn source_changed(&mut self, source: (Stamp, Stamp), changed: usize) {
        let previous = self.work.take();
        self.retire();
        let ticket = Ticket {
            stamp: source.1,
            generation: self.generation,
        };
        self.work = previous
            .filter(|work| work.ticket.stamp == source.0)
            .and_then(|mut work| work.source_changed(ticket, changed).then_some(work));
    }

    pub fn poll(&mut self, ticket: Ticket, text: RopeSlice<'_>, deadline: Instant) -> Status {
        let Some(work) = self.work.as_mut().filter(|work| work.ticket == ticket) else {
            return Status::Retired;
        };
        work.poll(text, deadline)
    }

    pub fn answer(&self, ticket: Ticket) -> Option<&Answer> {
        self.work
            .as_ref()
            .filter(|work| work.ticket == ticket)?
            .answer
            .as_ref()
    }

    pub fn stats(&self, ticket: Ticket) -> Option<Stats> {
        self.work
            .as_ref()
            .filter(|work| work.ticket == ticket)
            .map(|work| work.stats())
    }

    pub fn continue_query(&mut self, ticket: Ticket, query: Query) -> bool {
        let Some(work) = self.work.as_mut().filter(|work| work.ticket == ticket) else {
            return false;
        };
        work.retarget(query);
        true
    }

    pub fn cancel(&mut self, ticket: Ticket) -> bool {
        if !self.work.as_ref().is_some_and(|work| work.ticket == ticket) {
            return false;
        }
        self.retire();
        true
    }
}
