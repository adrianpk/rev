mod typing;

use std::ops::Range;
use std::time::Instant;

use ropey::RopeSlice;

use super::{
    Answer, Context, Hint, Kind, Limit, MAX_DEPTH, MAX_METADATA, Outcome, Pair, Profile,
    Provenance, Query, Reason, SLICE_TIME, SLICE_TOKENS, Stats, Status, Ticket,
    lexical::{Action, Lexical},
    reader::{Pause, Reader},
};
use crate::buffer::{Span, Stamp};

#[derive(Clone, Copy)]
struct Opening {
    at: usize,
    kind: Kind,
    admitted: bool,
    depth: usize,
}

struct Checkpoint {
    at: usize,
    horizon: usize,
    previous: Option<char>,
    line_start: bool,
    lexical: Lexical,
    stack: Vec<Opening>,
    facts: usize,
    valid: bool,
}

impl Checkpoint {
    fn new(profile: Profile) -> Self {
        Self {
            at: 0,
            horizon: 0,
            previous: None,
            line_start: true,
            lexical: Lexical::new(profile),
            stack: Vec::with_capacity(MAX_DEPTH),
            facts: 0,
            valid: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Record {
    opening: usize,
    closing: usize,
    kind: Kind,
    depth: usize,
}

impl Record {
    fn qualified(self, stamp: Stamp) -> Pair {
        Pair {
            opening: Span {
                stamp,
                range: self.opening..self.opening + 1,
            },
            closing: Span {
                stamp,
                range: self.closing..self.closing + 1,
            },
            kind: self.kind,
            depth: Some(self.depth),
        }
    }
}

enum Fact {
    Pair(Record),
    Excluded(Range<usize>, Context),
}

#[derive(Default)]
struct Search {
    at: usize,
    pair: Option<Record>,
    context: Option<Context>,
}

impl Search {
    fn observe(&mut self, fact: &Fact, query: Query) {
        let point = query.point().0;
        match fact {
            Fact::Excluded(span, context) if span.contains(&point) => self.context = Some(*context),
            Fact::Pair(pair) => self.pair(*pair, query),
            _ => (),
        }
        self.at += 1;
    }

    fn pair(&mut self, pair: Record, query: Query) {
        let point = query.point().0;
        if pair.kind.quote() && pair.opening < point && point <= pair.closing {
            self.context = Some(Context::String(pair.kind));
        }
        if matches_query(&pair, query)
            && self
                .pair
                .is_none_or(|prior| pair.closing - pair.opening < prior.closing - prior.opening)
        {
            self.pair = Some(pair);
        }
    }
}

pub(super) struct Work {
    pub ticket: Ticket,
    pub answer: Option<Answer>,
    query: Query,
    reader: Reader,
    lexical: Lexical,
    stack: Vec<Opening>,
    facts: Vec<Fact>,
    stats: Stats,
    target: Option<usize>,
    point_seen: bool,
    terminal: Option<Outcome>,
    provenance: Provenance,
    hint: Option<Hint>,
    context: Context,
    mapped: bool,
    search: Option<Search>,
    checkpoints: [Checkpoint; 2],
    checkpoint_next: usize,
    checkpoint_at: usize,
    typing: Option<typing::Progress>,
}

impl Work {
    pub fn new(ticket: Ticket, query: Query, profile: Profile, hint: Option<Hint>) -> Self {
        let nodes = hint.as_ref().map_or(0, |hint| hint.nodes);
        let provenance = if hint.is_some() {
            Provenance::CurrentSyntaxAndLexical
        } else {
            Provenance::Lexical
        };
        Self {
            ticket,
            answer: None,
            query,
            reader: Reader::default(),
            lexical: Lexical::new(profile),
            stack: Vec::with_capacity(MAX_DEPTH),
            facts: Vec::new(),
            stats: Stats {
                syntax_nodes: nodes,
                ..Stats::default()
            },
            target: None,
            point_seen: false,
            terminal: None,
            provenance,
            hint,
            context: Context::Unknown,
            mapped: false,
            search: None,
            checkpoints: std::array::from_fn(|_| Checkpoint::new(profile)),
            checkpoint_next: 0,
            checkpoint_at: 0,
            typing: None,
        }
    }

    pub fn stats(&self) -> Stats {
        Stats {
            inspected_bytes: self.reader.bytes,
            retained_bytes: std::mem::size_of::<Self>()
                + std::mem::size_of::<super::state::State>()
                + self.reader.retained()
                + self.stack.capacity() * std::mem::size_of::<Opening>()
                + self.facts.capacity() * std::mem::size_of::<Fact>()
                + self
                    .checkpoints
                    .iter()
                    .map(|checkpoint| checkpoint.stack.capacity() * std::mem::size_of::<Opening>())
                    .sum::<usize>(),
            ..self.stats
        }
    }

    pub fn reusable(&self, source: (Stamp, Profile)) -> bool {
        self.ticket.stamp == source.0 && self.lexical.profile == source.1 && self.stable_prefix()
    }

    fn stable_prefix(&self) -> bool {
        self.terminal
            .as_ref()
            .is_none_or(|outcome| *outcome == Outcome::Limited(Limit::Bytes))
    }

    pub fn begin(&mut self, ticket: Ticket, query: Query, hint: Option<Hint>) {
        self.ticket = ticket;
        self.reader.new_budget();
        self.stats = Stats {
            syntax_nodes: hint.as_ref().map_or(0, |hint| hint.nodes),
            reused_chars: self.reader.at,
            maximum_depth: self.stats.maximum_depth,
            ..Stats::default()
        };
        self.terminal = None;
        self.retarget(query);
        self.hint = hint;
        self.provenance = if self.mapped {
            Provenance::MappedLexical
        } else {
            Provenance::RetainedLexical
        };
    }

    pub fn source_changed(&mut self, ticket: Ticket, changed: usize) -> bool {
        if (self.reader.horizon() > changed || !self.stable_prefix())
            && !self.restore_checkpoint(changed)
        {
            return false;
        }
        if self.reader.at == 0 {
            return false;
        }
        self.ticket = ticket;
        self.answer = None;
        self.reader.changed_suffix();
        self.mapped = true;
        true
    }

    fn restore_checkpoint(&mut self, changed: usize) -> bool {
        let selected = self
            .checkpoints
            .iter()
            .enumerate()
            .filter(|(_, checkpoint)| checkpoint.valid && checkpoint.horizon <= changed)
            .max_by_key(|(_, checkpoint)| checkpoint.at)
            .map(|(index, _)| index);
        let Some(index) = selected else {
            return false;
        };
        let checkpoint = &self.checkpoints[index];
        self.reader
            .rewind((checkpoint.at, checkpoint.previous, checkpoint.line_start));
        self.lexical = checkpoint.lexical;
        self.stack.clone_from(&checkpoint.stack);
        self.facts.truncate(checkpoint.facts);
        self.checkpoint_at = checkpoint.at;
        self.checkpoint_next = 1 - index;
        self.checkpoints[1 - index].valid &= self.checkpoints[1 - index].at <= self.checkpoint_at;
        self.terminal = None;
        true
    }

    fn checkpoint(&mut self) {
        if matches!(self.query, Query::Typing(_)) && self.point_seen {
            return;
        }
        if self.reader.at.saturating_sub(self.checkpoint_at) < SLICE_TOKENS {
            return;
        }
        let checkpoint = &mut self.checkpoints[self.checkpoint_next];
        checkpoint.at = self.reader.at;
        checkpoint.horizon = self.reader.horizon();
        checkpoint.previous = self.reader.previous;
        checkpoint.line_start = self.reader.line_start;
        checkpoint.lexical = self.lexical;
        checkpoint.stack.clear();
        checkpoint.stack.extend_from_slice(&self.stack);
        checkpoint.facts = self.facts.len();
        checkpoint.valid = true;
        self.checkpoint_at = self.reader.at;
        self.checkpoint_next = 1 - self.checkpoint_next;
    }

    pub fn poll(&mut self, text: RopeSlice<'_>, deadline: Instant) -> Status {
        if self.answer.is_some() {
            return Status::Ready;
        }
        self.reader.slice_reads = 0;
        self.stats.slices += 1;
        let deadline = deadline.min(Instant::now() + SLICE_TIME);
        let mut tokens = 0;
        while tokens < SLICE_TOKENS && Instant::now() < deadline && self.answer.is_none() {
            if self.advance(text).is_err() {
                break;
            }
            tokens += 1;
        }
        self.stats.maximum_slice_tokens = self.stats.maximum_slice_tokens.max(tokens);
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
                self.stop(Outcome::Limited(limit));
                Err(())
            }
        }
    }

    fn step(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        if self.search.is_some() {
            self.search_step();
            return Ok(());
        }
        self.observe_point(text)?;
        if self.answer.is_some() {
            return Ok(());
        }
        if self.reader.at == text.len_chars() {
            self.eof();
            return Ok(());
        }
        self.scan_step(text)
    }

    fn observe_point(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        self.syntax_hint(text)?;
        if self.answer.is_some() {
            return Ok(());
        }
        if !self.point_seen && self.reader.at == self.query.point().0 {
            self.capture_point(text)?;
        }
        if self.answer.is_none() {
            self.typing_boundary(text)?;
        }
        Ok(())
    }

    fn scan_step(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        let at = self.reader.at;
        if self.typing_replacement(text)? {
            return Ok(());
        }
        let action = self.lexical.step(&mut self.reader, text)?;
        let depth = self.stack.len() + self.lexical.comment_depth() + self.typing_extra_depth();
        if depth > MAX_DEPTH {
            return Err(Pause::Limit(Limit::Depth));
        }
        self.stats.maximum_depth = self.stats.maximum_depth.max(depth);
        self.stats.maximum_depth = self.stats.maximum_depth.max(self.lexical.maximum_depth);
        self.action(at, action)?;
        self.checkpoint();
        self.crossed_point(at);
        Ok(())
    }

    fn syntax_hint(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        let Some(hint) = self.hint.take() else {
            return Ok(());
        };
        if hint.limited {
            self.stop(Outcome::Limited(Limit::Depth));
            return Ok(());
        }
        let context = hint.context;
        if matches!(self.query, Query::Context(_)) && context != Context::Code {
            self.context = context;
            self.provenance = Provenance::CurrentSyntax;
            self.finish(Outcome::Context(context));
            return Ok(());
        }
        if matches!(context, Context::Comment | Context::Literal) {
            self.context = context;
            self.provenance = Provenance::CurrentSyntax;
            self.stop(Outcome::Uncertain(Reason::LiteralContext));
            return Ok(());
        }
        if matches!(context, Context::String(_)) && matches!(self.query, Query::At(_)) {
            let c = text.get_char(self.query.point().0);
            self.reader.bytes += c.map_or(0, char::len_utf8);
            self.reader.slice_reads += usize::from(c.is_some());
            if c.is_some_and(|c| !matches!(c, '\'' | '"' | '`')) {
                self.context = context;
                self.provenance = Provenance::CurrentSyntax;
                self.stop(Outcome::Uncertain(Reason::LiteralContext));
            }
        }
        Ok(())
    }

    fn capture_point(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        if matches!(self.query, Query::Typing(_)) {
            return self.capture_typing(text);
        }
        self.capture_lookup();
        Ok(())
    }

    fn capture_lookup(&mut self) {
        self.point_seen = true;
        self.context = self.lexical.context();
        if matches!(self.query, Query::Context(_)) {
            self.finish(Outcome::Context(self.context));
            return;
        }
        if matches!(self.context, Context::Comment | Context::Literal) {
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
            return;
        }
        if matches!(self.query, Query::Enclosing(_)) {
            self.target = self.stack.last().map(|opening| opening.at);
            if self.stack.last().is_some_and(|opening| !opening.admitted) {
                self.finish(Outcome::Uncertain(Reason::LiteralContext));
            }
        }
    }

    fn crossed_point(&mut self, before: usize) {
        if self.answer.is_some() {
            return;
        }
        let point = self.query.point().0;
        if !self.point_seen && self.reader.at > point {
            self.point_seen = true;
            self.context = Context::Literal;
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
        } else if before == point && self.target.is_none() && self.typing.is_none() {
            let reason = if matches!(self.query, Query::Enclosing(_)) {
                Reason::NoEnclosingPair
            } else {
                Reason::NotDelimiter
            };
            self.finish(Outcome::Uncertain(reason));
        }
    }

    fn action(&mut self, at: usize, action: Action) -> Result<(), Pause> {
        match action {
            Action::Ignore => Ok(()),
            Action::Open(kind, admitted) => self.open(at, (kind, admitted)),
            Action::Close(kind) => self.close(at, kind),
            Action::Excluded(span, context) => self.retain(Fact::Excluded(span, context)),
            Action::Refuse(reason) => {
                self.stop(Outcome::Uncertain(reason));
                Ok(())
            }
        }
    }

    fn open(&mut self, at: usize, policy: (Kind, bool)) -> Result<(), Pause> {
        if self.stack.len() + self.typing_extra_depth() >= MAX_DEPTH {
            return Err(Pause::Limit(Limit::Depth));
        }
        let (kind, admitted) = policy;
        self.stack.push(Opening {
            at,
            kind,
            admitted,
            depth: self.stack.len(),
        });
        self.stats.maximum_depth = self.stats.maximum_depth.max(self.stack.len());
        if at == self.query.point().0 && !matches!(self.query, Query::Typing(_)) {
            if admitted {
                self.target = Some(at);
            } else {
                self.finish(Outcome::Uncertain(Reason::AmbiguousQuote));
            }
        }
        Ok(())
    }

    fn close(&mut self, at: usize, kind: Kind) -> Result<(), Pause> {
        let Some(opening) = self.stack.pop() else {
            if self.typing_unmatched(kind) {
                return Ok(());
            }
            self.stop(Outcome::Unmatched);
            return Ok(());
        };
        if opening.kind != kind {
            self.stop(Outcome::Mismatched {
                expected: opening.kind,
                found: kind,
            });
            return Ok(());
        }
        if !opening.admitted {
            return self.retain(Fact::Excluded(opening.at..at + 1, Context::String(kind)));
        }
        let pair = Record {
            opening: opening.at,
            closing: at,
            kind,
            depth: opening.depth,
        };
        self.retain(Fact::Pair(pair))?;
        if self.typing_closed(at, (kind, opening.admitted)) {
            return Ok(());
        }
        let at_query = matches!(self.query, Query::At(_)) && self.query.point().0 == at;
        if self.target == Some(opening.at) || at_query {
            self.finish(Outcome::Confirmed(pair.qualified(self.ticket.stamp)));
        }
        Ok(())
    }

    fn retain(&mut self, fact: Fact) -> Result<(), Pause> {
        if self.facts.len() == self.facts.capacity() {
            let available = MAX_METADATA.saturating_sub(self.stats().retained_bytes);
            let extra = (available / std::mem::size_of::<Fact>()).min(256);
            if extra == 0 {
                return Err(Pause::Limit(Limit::Retention));
            }
            self.facts
                .try_reserve_exact(extra)
                .map_err(|_| Pause::Limit(Limit::Retention))?;
            if self.stats().retained_bytes > MAX_METADATA {
                self.facts = Vec::new();
                return Err(Pause::Limit(Limit::Retention));
            }
        }
        self.facts.push(fact);
        Ok(())
    }

    fn eof(&mut self) {
        if self.finish_typing() {
            return;
        }
        if matches!(self.query, Query::At(_)) && self.query.point().0 == self.reader.at {
            self.finish(Outcome::Uncertain(Reason::NotDelimiter));
        } else {
            self.finish(Outcome::Unmatched);
        }
    }

    fn finish(&mut self, outcome: Outcome) {
        self.answer = Some(Answer {
            stamp: self.ticket.stamp,
            query: self.query,
            profile: self.lexical.profile,
            policy_version: super::POLICY_VERSION,
            context: self.context,
            provenance: self.provenance,
            outcome,
            stats: self.stats(),
        });
    }

    fn stop(&mut self, outcome: Outcome) {
        self.terminal = Some(outcome.clone());
        self.finish(outcome);
    }

    pub fn retarget(&mut self, query: Query) {
        self.query = query;
        self.context = Context::Unknown;
        self.answer = None;
        self.hint = None;
        self.provenance = if self.mapped {
            Provenance::MappedLexical
        } else {
            Provenance::RetainedLexical
        };
        self.point_seen = query.point().0 < self.reader.at;
        self.target = None;
        self.search = None;
        self.typing = None;
        if matches!(query, Query::Typing(_)) {
            self.rewind_typing();
        }
        if self.point_seen {
            self.search = Some(Search::default());
        } else if let Some(outcome) = self.terminal.clone() {
            self.finish(outcome);
        }
    }

    fn search_step(&mut self) {
        let mut search = self.search.take().expect("active pair fact search");
        if let Some(fact) = self.facts.get(search.at) {
            search.observe(fact, self.query);
            self.search = Some(search);
            return;
        }
        self.context = search.context.unwrap_or_else(|| {
            if self
                .lexical
                .start()
                .is_some_and(|start| start < self.query.point().0)
            {
                self.lexical.context()
            } else {
                Context::Code
            }
        });
        self.cached_result(search.pair);
    }

    fn cached_result(&mut self, pair: Option<Record>) {
        if matches!(self.query, Query::Context(_)) {
            self.finish(Outcome::Context(self.context));
            return;
        }
        if matches!(self.context, Context::Comment | Context::Literal) {
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
            return;
        }
        if let Some(pair) =
            pair.filter(|pair| !matches!(self.context, Context::String(_)) || pair.kind.quote())
        {
            self.finish(Outcome::Confirmed(pair.qualified(self.ticket.stamp)));
            return;
        }
        if matches!(self.context, Context::String(_)) {
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
            return;
        }
        let point = self.query.point().0;
        self.target = self
            .stack
            .iter()
            .rev()
            .find(|opening| match self.query {
                Query::At(_) => opening.at == point,
                Query::Enclosing(_) => opening.at <= point,
                Query::Context(_) | Query::Typing(_) => false,
            })
            .map(|opening| opening.at);
        if self.target.is_none() {
            self.finish(Outcome::Uncertain(Reason::NoEnclosingPair));
        }
    }
}

fn matches_query(pair: &Record, query: Query) -> bool {
    let point = query.point().0;
    match query {
        Query::At(_) => pair.opening == point || pair.closing == point,
        Query::Enclosing(_) => pair.opening <= point && point <= pair.closing,
        Query::Context(_) | Query::Typing(_) => false,
    }
}
