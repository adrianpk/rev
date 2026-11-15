use std::time::Instant;

use ropey::RopeSlice;

use super::{
    Answer, GroupAnswer, GroupRequest, Hint, Profile, Query, Stats, Status, Ticket, groups,
    work::Work,
};
use crate::buffer::Stamp;

/// One operation owner per canonical buffer, independent of its view count.
#[derive(Default)]
pub(crate) struct State {
    generation: u64,
    work: Option<Active>,
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
        self.work = Some(match previous {
            Some(Active::Pair(mut work)) if work.reusable(source) => {
                work.begin(ticket, query, hint);
                Active::Pair(work)
            }
            _ => Active::Pair(Box::new(Work::new(ticket, query, source.1, hint))),
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
        self.work = previous.and_then(|active| match active {
            Active::Pair(mut work) if work.ticket.stamp == source.0 => work
                .source_changed(ticket, changed)
                .then_some(Active::Pair(work)),
            _ => None,
        });
    }

    pub fn poll(&mut self, ticket: Ticket, text: RopeSlice<'_>, deadline: Instant) -> Status {
        let Some(work) = self.work.as_mut().filter(|work| work.ticket() == ticket) else {
            return Status::Retired;
        };
        work.poll(text, deadline)
    }

    pub fn answer(&self, ticket: Ticket) -> Option<&Answer> {
        match self.work.as_ref().filter(|work| work.ticket() == ticket)? {
            Active::Pair(work) => work.answer.as_ref(),
            Active::Groups(_) => None,
        }
    }

    pub fn group_answer(&self, ticket: Ticket) -> Option<&GroupAnswer> {
        match self.work.as_ref().filter(|work| work.ticket() == ticket)? {
            Active::Groups(work) => work.answer.as_ref(),
            Active::Pair(_) => None,
        }
    }

    pub fn start_groups(
        &mut self,
        request: GroupRequest,
        source: (Stamp, Profile),
        hint: Option<Hint>,
    ) -> Ticket {
        self.retire();
        let ticket = Ticket {
            stamp: source.0,
            generation: self.generation,
        };
        self.work = Some(Active::Groups(Box::new(groups::Work::new(
            ticket, request, source.1, hint,
        ))));
        ticket
    }

    pub fn stats(&self, ticket: Ticket) -> Option<Stats> {
        self.work
            .as_ref()
            .filter(|work| work.ticket() == ticket)
            .map(Active::stats)
    }

    pub fn continue_query(&mut self, ticket: Ticket, query: Query) -> bool {
        let Some(Active::Pair(work)) = self.work.as_mut().filter(|work| work.ticket() == ticket)
        else {
            return false;
        };
        work.retarget(query);
        true
    }

    pub fn cancel(&mut self, ticket: Ticket) -> bool {
        if !self
            .work
            .as_ref()
            .is_some_and(|work| work.ticket() == ticket)
        {
            return false;
        }
        self.retire();
        true
    }
}

/// Exactly one request kind owns the buffer budget and retained metadata.
enum Active {
    Pair(Box<Work>),
    Groups(Box<groups::Work>),
}

impl Active {
    fn ticket(&self) -> Ticket {
        match self {
            Self::Pair(work) => work.ticket,
            Self::Groups(work) => work.ticket,
        }
    }
    fn poll(&mut self, text: RopeSlice<'_>, deadline: Instant) -> Status {
        match self {
            Self::Pair(work) => work.poll(text, deadline),
            Self::Groups(work) => work.poll(text, deadline),
        }
    }
    fn stats(&self) -> Stats {
        match self {
            Self::Pair(work) => work.stats(),
            Self::Groups(work) => work.stats(),
        }
    }
}
