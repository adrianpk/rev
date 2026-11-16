use std::time::Instant;

use ropey::{Rope, RopeSlice};

use super::{Active, Completed, State};
use crate::buffer::{Span, edit::Transaction};
use crate::coordinates::Scalar;
use crate::pairs::{
    Advance, MAX_METADATA, Presentation, PresentationTicket, Profile, Query, Status, Ticket,
    work::Work,
};
use crate::window::ViewId;

impl State {
    pub fn retained(&self) -> usize {
        self.work
            .as_ref()
            .map_or(std::mem::size_of::<Self>(), Active::stats_bytes)
            + self.presentation.heap()
    }

    pub fn start_presentation(&mut self, view: ViewId, span: Span) -> Option<PresentationTicket> {
        self.presentation.request(view, span, self.retained())
    }

    pub fn presentation(&self, ticket: PresentationTicket) -> Option<Presentation<'_>> {
        self.presentation.answer(ticket, self.retained())
    }

    pub fn cancel_presentation(&mut self, ticket: PresentationTicket) -> bool {
        let cancelled = self.presentation.cancel(ticket);
        if self.presentation.empty() && self.work.as_ref().is_some_and(Active::background) {
            self.restore_foreground();
        }
        cancelled
    }

    fn restore_foreground(&mut self) {
        let Some(Active::Background(mut work)) = self.work.take() else {
            return;
        };
        let (ticket, answer) = match self.completed.take() {
            Some(Completed::Pair(ticket, answer)) => (ticket, answer),
            other => {
                self.completed = other;
                return;
            }
        };
        work.resume(ticket, answer.query, answer.stats);
        work.answer = Some(answer);
        self.work = Some(Active::Pair(work));
    }

    pub fn poll_presentation(
        &mut self,
        request: PresentationTicket,
        source: (RopeSlice<'_>, Profile),
        deadline: Instant,
    ) -> Status {
        if self.presentation.ready(request) {
            return Status::Ready;
        }
        let Some(range) = self.presentation.range(request) else {
            return Status::Retired;
        };
        if self.work.as_ref().is_some_and(Active::pending_foreground) {
            return Status::Pending;
        }
        if !self.work.as_ref().is_some_and(Active::background)
            || self.presentation.needs_target(range.end)
        {
            if self.presentation.running() && self.work.as_ref().is_some_and(Active::background) {
                self.extend_background(range.end);
            } else {
                self.background(request, range.end, source.1);
            }
        }
        self.poll_background(source.0, deadline)
    }

    fn extend_background(&mut self, target: usize) {
        self.presentation.begin(target);
        let Some(Active::Background(work)) = &mut self.work else {
            unreachable!("current background request");
        };
        // Overlapping views share the in-flight request's inspection budget.
        work.retarget(Query::Presentation(Scalar(target)));
    }

    fn background(&mut self, request: PresentationTicket, target: usize, profile: Profile) {
        let previous = self.work.take();
        self.generation = self
            .generation
            .checked_add(1)
            .expect("pair generation exhausted");
        let ticket = Ticket {
            stamp: request.stamp,
            generation: self.generation,
        };
        let query = Query::Presentation(Scalar(target));
        self.presentation.begin(target);
        let work = match previous {
            Some(Active::Pair(mut work)) => {
                self.completed = work.answer.take().map(|a| Completed::Pair(work.ticket, a));
                if work.reusable((request.stamp, profile)) {
                    work.begin(ticket, query, None);
                    work
                } else {
                    drop(work);
                    Box::new(Work::new(ticket, query, profile, None))
                }
            }
            Some(Active::Groups(mut work)) => {
                self.completed = work
                    .answer
                    .take()
                    .map(|a| Completed::Groups(work.ticket, a));
                drop(work);
                Box::new(Work::new(ticket, query, profile, None))
            }
            Some(Active::Background(mut work) | Active::Prefix(mut work))
                if work.reusable((request.stamp, profile)) =>
            {
                work.begin(ticket, query, None);
                work
            }
            _ => Box::new(Work::new(ticket, query, profile, None)),
        };
        self.work = Some(Active::Background(work));
    }

    fn poll_background(&mut self, text: RopeSlice<'_>, deadline: Instant) -> Status {
        let heap = self.presentation.heap();
        let Some(Active::Background(work)) = &mut self.work else {
            return Status::Pending;
        };
        work.metadata_limit = MAX_METADATA.saturating_sub(heap);
        if work.answer.is_none() {
            work.poll(text, deadline);
            return Status::Pending;
        }
        if self
            .presentation
            .export(work, work.stats().retained_bytes, deadline)
        {
            Status::Ready
        } else {
            Status::Pending
        }
    }

    pub(super) fn foreground_budget(&mut self) {
        // A foreground poll can emit at most two facts per token. Evict visual
        // retention before its next reserve could reduce the native work budget.
        let reserve = Work::poll_retention();
        if self.retained().saturating_add(reserve) > MAX_METADATA {
            self.presentation.retire();
        }
        let limit = MAX_METADATA.saturating_sub(self.presentation.heap());
        if let Some(work) = &mut self.work {
            work.limit(limit);
        }
    }

    pub fn prepare_advance(&self, record: &Transaction, texts: (&Rope, &Rope)) -> Option<Advance> {
        self.presentation.proof(record, texts)
    }

    pub fn advance_presentation(&mut self, proof: Option<Advance>) {
        self.presentation.map_snapshot(proof);
    }
}

impl Active {
    fn stats_bytes(&self) -> usize {
        self.stats().retained_bytes
    }
}
