use std::time::Instant;

use ropey::RopeSlice;

use super::{
    Context, Hint, Kind, Limit, MAX_DEPTH, MAX_METADATA, Profile, Reason, SLICE_TIME, SLICE_TOKENS,
    Stats, Status, Ticket,
    lexical::{Action, Lexical},
    reader::{Pause, Reader},
};
use crate::buffer::Stamp;
use crate::coordinates::Scalar;

/// One admitted signed group motion. Construction preserves native count bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupRequest {
    pub point: Scalar,
    pub backward: bool,
    count: i32,
    movement: Movement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Movement {
    Across,
    Up,
    Down,
}

impl GroupRequest {
    pub fn new(point: Scalar, backward: bool, count: i32) -> Option<Self> {
        (count.unsigned_abs() <= crate::command::PrefixArgument::LIMIT.unsigned_abs()).then_some(
            Self {
                point,
                backward,
                count,
                movement: Movement::Across,
            },
        )
    }

    pub fn count(self) -> i32 {
        self.count
    }

    /// Leave an enclosing group backward, or enter a nested group forward.
    /// A negative count reverses that direction under the same work budget.
    pub fn nesting(point: Scalar, down: bool, count: i32) -> Option<Self> {
        Self::new(point, !down, count).map(|mut request| {
            request.movement = if down { Movement::Down } else { Movement::Up };
            request
        })
    }
}

/// A group endpoint is never a confirmed typed pair or editing authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupOutcome {
    Target(Scalar),
    Boundary(Scalar),
    NoGroup,
    Uncertain(Reason),
    Limited(Limit),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupAnswer {
    pub stamp: Stamp,
    pub request: GroupRequest,
    pub profile: Profile,
    pub policy_version: u32,
    pub outcome: GroupOutcome,
    pub stats: Stats,
}

struct Group {
    start: usize,
    end: usize,
    depth: usize,
}

/// Backward motion scans its prefix once, then visits completed groups in reverse.
/// These source facts are bounded per buffer, never a per-count destination list.
pub(super) struct Work {
    pub ticket: Ticket,
    pub answer: Option<GroupAnswer>,
    request: GroupRequest,
    backward: bool,
    remaining: u32,
    reader: Reader,
    lexical: Lexical,
    stack: Vec<usize>,
    groups: Vec<Group>,
    search: Option<usize>,
    base: Option<usize>,
    floor: Option<usize>,
    destination: usize,
    stats: Stats,
    hint: Option<Hint>,
}

impl Work {
    pub fn new(
        ticket: Ticket,
        request: GroupRequest,
        profile: Profile,
        hint: Option<Hint>,
    ) -> Self {
        Self {
            ticket,
            answer: None,
            request,
            backward: request.backward != (request.count < 0),
            remaining: request.count.unsigned_abs(),
            reader: Reader::default(),
            lexical: Lexical::new(profile),
            stack: Vec::with_capacity(MAX_DEPTH),
            groups: Vec::new(),
            search: None,
            base: None,
            floor: None,
            destination: request.point.0,
            stats: Stats {
                syntax_nodes: hint.as_ref().map_or(0, |h| h.nodes),
                ..Stats::default()
            },
            hint,
        }
    }

    pub fn stats(&self) -> Stats {
        Stats {
            inspected_bytes: self.reader.bytes,
            retained_bytes: std::mem::size_of::<Self>()
                + std::mem::size_of::<super::state::State>()
                + self.reader.retained()
                + self.stack.capacity() * std::mem::size_of::<usize>()
                + self.groups.capacity() * std::mem::size_of::<Group>(),
            ..self.stats
        }
    }

    pub fn poll(&mut self, text: RopeSlice<'_>, deadline: Instant) -> Status {
        if self.answer.is_some() {
            return Status::Ready;
        }
        self.stats.slices += 1;
        self.reader.slice_reads = 0;
        let deadline = deadline.min(Instant::now() + SLICE_TIME);
        let mut steps = 0;
        while steps < SLICE_TOKENS && Instant::now() < deadline && self.answer.is_none() {
            if self.advance(text).is_err() {
                break;
            }
            steps += 1;
        }
        self.stats.maximum_slice_tokens = self.stats.maximum_slice_tokens.max(steps);
        let stats = self.stats();
        if let Some(answer) = &mut self.answer {
            answer.stats = stats;
        }
        if self.answer.is_some() {
            Status::Ready
        } else {
            Status::Pending
        }
    }

    fn advance(&mut self, text: RopeSlice<'_>) -> Result<(), ()> {
        match self.step(text) {
            Ok(()) => Ok(()),
            Err(Pause::Yield) => Err(()),
            Err(Pause::Limit(limit)) => {
                self.finish(GroupOutcome::Limited(limit));
                Err(())
            }
        }
    }

    fn step(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        if self.remaining == 0 {
            self.finish(GroupOutcome::Target(Scalar(self.destination)));
            return Ok(());
        }
        if self.search.is_some() {
            self.search_step();
            return Ok(());
        }
        self.observe_point();
        if self.answer.is_some() || self.search.is_some() {
            return Ok(());
        }
        if self.reader.at == text.len_chars() {
            self.eof();
            return Ok(());
        }
        self.scan(text)
    }

    fn scan(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        let at = self.reader.at;
        let action = self.lexical.step(&mut self.reader, text)?;
        self.observe_depth()?;
        self.action(at, &action)?;
        if self.base.is_none() && self.reader.at > self.request.point.0 {
            self.finish(GroupOutcome::NoGroup);
        }
        Ok(())
    }

    fn observe_point(&mut self) {
        if self.base.is_some() || self.reader.at != self.request.point.0 {
            return;
        }
        if let Some(hint) = self.hint.take() {
            if hint.limited {
                self.finish(GroupOutcome::Limited(Limit::Depth));
                return;
            }
            if hint.context == Context::Literal {
                self.finish(GroupOutcome::Uncertain(Reason::LiteralContext));
                return;
            }
        }
        if self.lexical.context() != Context::Code && !self.lexical.complete_at_eof() {
            self.finish(GroupOutcome::NoGroup);
            return;
        }
        self.base = Some(self.stack.len());
        if self.backward {
            self.begin_backward();
        }
    }

    fn begin_backward(&mut self) {
        if self.request.movement != Movement::Up {
            self.search = Some(self.groups.len());
            return;
        }
        let count = usize::try_from(self.remaining).expect("bounded group count fits usize");
        let outcome = self
            .stack
            .len()
            .checked_sub(count)
            .and_then(|index| self.stack.get(index))
            .map_or(GroupOutcome::NoGroup, |&start| {
                GroupOutcome::Target(Scalar(start))
            });
        self.finish(outcome);
    }

    fn observe_depth(&mut self) -> Result<(), Pause> {
        let quote = usize::from(matches!(self.lexical.context(), Context::String(_)));
        let depth = self.stack.len() + self.lexical.comment_depth() + quote;
        if depth > MAX_DEPTH {
            return Err(Pause::Limit(Limit::Depth));
        }
        self.stats.maximum_depth = self.stats.maximum_depth.max(depth);
        Ok(())
    }

    fn action(&mut self, at: usize, action: &Action) -> Result<(), Pause> {
        match action {
            Action::Open(kind, _) if !kind.quote() => self.open(at),
            Action::Close(kind) if !kind.quote() => self.close(),
            Action::Open(kind, admitted) => {
                self.quote(*kind, *admitted);
                Ok(())
            }
            Action::Refuse(reason) => {
                self.finish(GroupOutcome::Uncertain(*reason));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn quote(&mut self, kind: Kind, admitted: bool) {
        let profile = self.lexical.profile;
        let known = profile.quote(kind) || profile == Profile::Rust && kind == Kind::SingleQuote;
        if !known || profile == Profile::Html && !admitted {
            self.finish(GroupOutcome::Uncertain(Reason::AmbiguousQuote));
        }
    }

    fn open(&mut self, at: usize) -> Result<(), Pause> {
        if self.stack.len() == MAX_DEPTH {
            return Err(Pause::Limit(Limit::Depth));
        }
        self.stack.push(at);
        self.stats.maximum_depth = self.stats.maximum_depth.max(self.stack.len());
        if self.base.is_some() && self.request.movement == Movement::Down && !self.backward {
            self.destination = self.reader.at;
            self.remaining -= 1;
            self.base = Some(self.stack.len());
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), Pause> {
        let Some(start) = self.stack.pop() else {
            self.finish(GroupOutcome::NoGroup);
            return Ok(());
        };
        let depth = self.stack.len();
        if self.backward && self.request.movement != Movement::Up {
            return self.retain(Group {
                start,
                end: self.reader.at,
                depth,
            });
        }
        match self.request.movement {
            Movement::Across => self.across_close(start, depth),
            Movement::Up => self.up_close(depth),
            Movement::Down => self.down_close(depth),
        }
        Ok(())
    }

    fn across_close(&mut self, start: usize, depth: usize) {
        let Some(base) = self.base else {
            return;
        };
        if depth < base {
            self.finish(GroupOutcome::NoGroup);
        } else if depth == base && start >= self.request.point.0 {
            self.destination = self.reader.at;
            self.remaining -= 1;
        }
    }

    fn up_close(&mut self, depth: usize) {
        let Some(base) = self.base else {
            return;
        };
        if depth < base {
            self.destination = self.reader.at;
            self.remaining -= 1;
            self.base = Some(depth);
        }
    }

    fn down_close(&mut self, depth: usize) {
        let Some(base) = self.base else {
            return;
        };
        if depth < base {
            self.finish(GroupOutcome::NoGroup);
        }
    }

    fn retain(&mut self, group: Group) -> Result<(), Pause> {
        if self.groups.len() == self.groups.capacity() {
            let available = MAX_METADATA.saturating_sub(self.stats().retained_bytes);
            let extra = (available / std::mem::size_of::<Group>()).min(256);
            if extra == 0 {
                return Err(Pause::Limit(Limit::Retention));
            }
            self.groups
                .try_reserve_exact(extra)
                .map_err(|_| Pause::Limit(Limit::Retention))?;
            if self.stats().retained_bytes > MAX_METADATA {
                self.groups = Vec::new();
                return Err(Pause::Limit(Limit::Retention));
            }
        }
        self.groups.push(group);
        Ok(())
    }

    fn search_step(&mut self) {
        let index = self.search.expect("owned backward group search");
        if index == 0 {
            let outcome = if self.base == Some(0) && self.request.movement == Movement::Across {
                GroupOutcome::Boundary(Scalar(0))
            } else {
                GroupOutcome::NoGroup
            };
            self.finish(outcome);
            return;
        }
        self.search = Some(index - 1);
        let group = &self.groups[index - 1];
        if Some(group.depth) == self.base
            && group.end <= self.destination
            && self.floor.is_none_or(|start| group.start > start)
        {
            if self.request.movement == Movement::Down {
                self.destination = group.end - 1;
                self.base = Some(group.depth + 1);
                self.floor = Some(group.start);
            } else {
                self.destination = group.start;
            }
            self.remaining -= 1;
        }
    }

    fn eof(&mut self) {
        let outcome = if self.request.movement == Movement::Across
            && self.stack.is_empty()
            && self.lexical.complete_at_eof()
        {
            GroupOutcome::Boundary(Scalar(self.reader.at))
        } else {
            GroupOutcome::NoGroup
        };
        self.finish(outcome);
    }

    fn finish(&mut self, outcome: GroupOutcome) {
        self.answer = Some(GroupAnswer {
            stamp: self.ticket.stamp,
            request: self.request,
            profile: self.lexical.profile,
            policy_version: super::POLICY_VERSION,
            outcome,
            stats: self.stats(),
        });
    }
}
