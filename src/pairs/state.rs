use std::time::Instant;

use ropey::RopeSlice;

use super::{
    Answer, GroupAnswer, GroupRequest, Hint, Profile, Query, Stats, Status, Ticket, groups,
    work::Work,
};
use crate::buffer::Stamp;

mod presentation;

/// Exactly one mutable lexical operation per canonical buffer. Publications and
/// per-view interests share its budget; foreground tickets retain priority.
#[derive(Default)]
pub(crate) struct State {
    generation: u64,
    work: Option<Active>,
    completed: Option<Completed>,
    presentation: super::presentation::Store,
}

impl State {
    pub fn retire_work(&mut self) {
        self.work = None;
        self.completed = None;
        self.generation = self
            .generation
            .checked_add(1)
            .expect("pair generation exhausted");
    }

    pub fn retire(&mut self) {
        self.retire_work();
        self.presentation.retire();
    }

    pub fn start(&mut self, query: Query, source: (Stamp, Profile), hint: Option<Hint>) -> Ticket {
        let previous = self.work.take();
        self.retire_work();
        self.presentation.stop_export();
        let ticket = Ticket {
            stamp: source.0,
            generation: self.generation,
        };
        self.work = Some(match previous {
            Some(
                Active::Pair(mut work) | Active::Background(mut work) | Active::Prefix(mut work),
            ) if work.reusable(source) => {
                work.begin(ticket, query, hint);
                Active::Pair(work)
            }
            _ => Active::Pair(Box::new(Work::new(ticket, query, source.1, hint))),
        });
        self.foreground_budget();
        ticket
    }

    /// Reuse only a prefix whose complete lookahead predates the first change.
    /// Native publication separately advances or invalidates immutable spans.
    pub fn source_changed(&mut self, source: (Stamp, Stamp), changed: usize) {
        let previous = self.work.take();
        self.retire_work();
        self.presentation.stop_export();
        let ticket = Ticket {
            stamp: source.1,
            generation: self.generation,
        };
        self.work = previous.and_then(|active| match active {
            Active::Pair(mut work) | Active::Prefix(mut work) if work.ticket.stamp == source.0 => {
                work.source_changed(ticket, changed)
                    .then_some(Active::Prefix(work))
            }
            _ => None,
        });
    }

    pub fn poll(&mut self, ticket: Ticket, text: RopeSlice<'_>, deadline: Instant) -> Status {
        if self
            .completed
            .as_ref()
            .is_some_and(|c| c.ticket() == ticket)
        {
            return Status::Ready;
        }
        if !self
            .work
            .as_ref()
            .is_some_and(|w| w.ticket() == ticket && w.foreground())
        {
            return Status::Retired;
        }
        self.foreground_budget();
        self.work
            .as_mut()
            .expect("current foreground work")
            .poll(text, deadline)
    }

    pub fn answer(&self, ticket: Ticket) -> Option<&Answer> {
        if let Some(Completed::Pair(saved, answer)) = &self.completed
            && *saved == ticket
        {
            return Some(answer);
        }
        match self.work.as_ref().filter(|work| work.ticket() == ticket)? {
            Active::Pair(work) => work.answer.as_ref(),
            _ => None,
        }
    }

    pub fn group_answer(&self, ticket: Ticket) -> Option<&GroupAnswer> {
        if let Some(Completed::Groups(saved, answer)) = &self.completed
            && *saved == ticket
        {
            return Some(answer);
        }
        match self.work.as_ref().filter(|work| work.ticket() == ticket)? {
            Active::Groups(work) => work.answer.as_ref(),
            _ => None,
        }
    }

    pub fn start_groups(
        &mut self,
        request: GroupRequest,
        source: (Stamp, Profile),
        hint: Option<Hint>,
    ) -> Ticket {
        self.retire_work();
        self.presentation.stop_export();
        let ticket = Ticket {
            stamp: source.0,
            generation: self.generation,
        };
        self.work = Some(Active::Groups(Box::new(groups::Work::new(
            ticket, request, source.1, hint,
        ))));
        self.foreground_budget();
        ticket
    }

    pub fn stats(&self, ticket: Ticket) -> Option<Stats> {
        let stats = self.saved_stats(ticket).or_else(|| {
            self.work
                .as_ref()
                .filter(|w| w.ticket() == ticket && w.foreground())
                .map(Active::stats)
        })?;
        Some(Stats {
            retained_bytes: self.retained(),
            ..stats
        })
    }

    fn saved_stats(&self, ticket: Ticket) -> Option<Stats> {
        match self.completed.as_ref().filter(|c| c.ticket() == ticket)? {
            Completed::Pair(_, a) => Some(a.stats),
            Completed::Groups(_, a) => Some(a.stats),
        }
    }

    pub fn continue_query(&mut self, ticket: Ticket, query: Query) -> bool {
        if let Some(Completed::Pair(saved, _)) = &self.completed
            && *saved == ticket
        {
            return self.resume_pair(ticket, query);
        }
        let Some(Active::Pair(work)) = self.work.as_mut().filter(|work| work.ticket() == ticket)
        else {
            return false;
        };
        work.retarget(query);
        true
    }

    fn resume_pair(&mut self, ticket: Ticket, query: Query) -> bool {
        let Some(Active::Background(mut work)) = self.work.take() else {
            return false;
        };
        self.presentation.stop_export();
        let Some(Completed::Pair(_, answer)) = self.completed.take() else {
            unreachable!("saved foreground pair");
        };
        work.resume(ticket, query, answer.stats);
        self.work = Some(Active::Pair(work));
        true
    }

    pub fn cancel(&mut self, ticket: Ticket) -> bool {
        if self
            .completed
            .as_ref()
            .is_some_and(|c| c.ticket() == ticket)
        {
            self.completed = None;
            return true;
        }
        if !self
            .work
            .as_ref()
            .is_some_and(|w| w.ticket() == ticket && w.foreground())
        {
            return false;
        }
        self.retire_work();
        true
    }
}

enum Active {
    Pair(Box<Work>),
    Groups(Box<groups::Work>),
    Background(Box<Work>),
    Prefix(Box<Work>),
}

enum Completed {
    Pair(Ticket, Answer),
    Groups(Ticket, GroupAnswer),
}

impl Completed {
    fn ticket(&self) -> Ticket {
        match self {
            Self::Pair(ticket, _) | Self::Groups(ticket, _) => *ticket,
        }
    }
}

impl Active {
    fn foreground(&self) -> bool {
        matches!(self, Self::Pair(_) | Self::Groups(_))
    }
    fn background(&self) -> bool {
        matches!(self, Self::Background(_))
    }
    fn ticket(&self) -> Ticket {
        match self {
            Self::Pair(w) | Self::Background(w) | Self::Prefix(w) => w.ticket,
            Self::Groups(w) => w.ticket,
        }
    }
    fn poll(&mut self, text: RopeSlice<'_>, deadline: Instant) -> Status {
        match self {
            Self::Pair(w) | Self::Background(w) | Self::Prefix(w) => w.poll(text, deadline),
            Self::Groups(w) => w.poll(text, deadline),
        }
    }
    fn stats(&self) -> Stats {
        match self {
            Self::Pair(w) | Self::Background(w) | Self::Prefix(w) => w.stats(),
            Self::Groups(w) => w.stats(),
        }
    }
    fn pending_foreground(&self) -> bool {
        match self {
            Self::Pair(w) => w.answer.is_none(),
            Self::Groups(w) => w.answer.is_none(),
            Self::Background(_) | Self::Prefix(_) => false,
        }
    }
    fn limit(&mut self, limit: usize) {
        match self {
            Self::Pair(w) | Self::Background(w) | Self::Prefix(w) => w.metadata_limit = limit,
            Self::Groups(w) => w.metadata_limit = limit,
        }
    }
}
