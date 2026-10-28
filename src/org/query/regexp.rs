//! Finite Thompson automata for native agenda regexps. No backtracking or code
//! evaluation; unsupported constructs reject before any source is observed.
mod parse;
use crate::org::Cancellation;
use ropey::RopeSlice;
#[derive(Debug, Clone)]
pub struct Regexp {
    states: Vec<State>,
    start: usize,
}
#[derive(Debug, Clone)]
enum State {
    Consume(Atom, usize),
    Split(usize, usize),
    Start(usize),
    End(usize),
    WordStart(usize),
    WordEnd(usize),
    Accept,
}
#[derive(Debug, Clone)]
enum Atom {
    Literal(char),
    Any,
    Space,
    Word,
    Class {
        ranges: Vec<(char, char)>,
        negative: bool,
    },
}
impl Atom {
    fn matches(&self, c: char) -> bool {
        match self {
            Self::Literal(value) => *value == c,
            Self::Any => c != '\n' && c != '\r',
            Self::Space => c.is_whitespace(),
            Self::Word => c.is_alphanumeric() || c == '_',
            Self::Class { ranges, negative } => {
                ranges.iter().any(|(a, b)| *a <= c && c <= *b) != *negative
            }
        }
    }
}
impl Regexp {
    pub fn parse(source: &str, depth: usize) -> Result<Self, String> {
        parse::Parser::new(source, depth)?.compile()
    }
    pub fn matches(&self, text: RopeSlice<'_>, cancel: &Cancellation) -> Result<bool, String> {
        self.run(text.chars(), cancel)
    }
    pub fn matches_folded(
        &self,
        text: RopeSlice<'_>,
        cancel: &Cancellation,
    ) -> Result<bool, String> {
        self.run(text.chars().flat_map(char::to_lowercase), cancel)
    }
    fn run(
        &self,
        chars: impl Iterator<Item = char>,
        cancel: &Cancellation,
    ) -> Result<bool, String> {
        let mut raw = Vec::new();
        let mut before = None;
        for after in chars.map(Some).chain(std::iter::once(None)) {
            if cancel.is_cancelled() {
                return Err("Agenda query cancelled or timed out".into());
            }
            raw.push(self.start);
            let (active, accepted) = self.closure(&raw, before, after);
            if accepted {
                return Ok(true);
            }
            raw.clear();
            if let Some(c) = after {
                raw = self.consume(active, c);
            }
            before = after;
        }
        Ok(false)
    }
    fn consume(&self, active: Vec<usize>, c: char) -> Vec<usize> {
        active
            .into_iter()
            .filter_map(|state| match &self.states[state] {
                State::Consume(atom, next) if atom.matches(c) => Some(*next),
                _ => None,
            })
            .collect()
    }
    fn closure(
        &self,
        raw: &[usize],
        before: Option<char>,
        after: Option<char>,
    ) -> (Vec<usize>, bool) {
        let mut pending = raw.to_vec();
        let mut seen = vec![false; self.states.len()];
        let mut active = Vec::new();
        while let Some(i) = pending.pop() {
            if seen[i] {
                continue;
            }
            seen[i] = true;
            if self.expand(i, before, after, &mut pending) {
                return (active, true);
            }
            if matches!(self.states[i], State::Consume(..)) {
                active.push(i);
            }
        }
        (active, false)
    }
    fn expand(
        &self,
        index: usize,
        before: Option<char>,
        after: Option<char>,
        pending: &mut Vec<usize>,
    ) -> bool {
        match self.states[index] {
            State::Split(a, b) => pending.extend([a, b]),
            State::Accept => return true,
            _ => {
                if let Some(next) = self.states[index].assertion(before, after) {
                    pending.push(next);
                }
            }
        }
        false
    }
}

fn word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl State {
    fn assertion(&self, before: Option<char>, after: Option<char>) -> Option<usize> {
        match *self {
            State::Start(next) if before.is_none_or(|c| c == '\n') => Some(next),
            State::End(next) if after.is_none_or(|c| matches!(c, '\n' | '\r')) => Some(next),
            State::WordStart(next) if edge(before, after, true) => Some(next),
            State::WordEnd(next) if edge(before, after, false) => Some(next),
            _ => None,
        }
    }
}
fn edge(before: Option<char>, after: Option<char>, start: bool) -> bool {
    let before = before.is_some_and(word);
    let after = after.is_some_and(word);
    if start {
        !before && after
    } else {
        before && !after
    }
}
