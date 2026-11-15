//! Bounded native typed-pair qualification, independent of modal grouping.
//! One canonical buffer owns each request, its budget and retained facts.

mod groups;
mod lexical;
mod profile;
mod reader;
mod state;
mod work;

pub use groups::{GroupAnswer, GroupOutcome, GroupRequest};
pub use profile::Profile;
pub(crate) use state::State;

use crate::buffer::{Span, Stamp};
use crate::coordinates::Scalar;

pub const MAX_BYTES: usize = 262_144;
pub const POLICY_VERSION: u32 = 1;
pub const MAX_DEPTH: usize = 256;
pub const MAX_METADATA: usize = 2_097_152;
pub const SLICE_TOKENS: usize = 256;
pub const SLICE_TIME: std::time::Duration = std::time::Duration::from_millis(4);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Parenthesis,
    Bracket,
    Brace,
    DoubleQuote,
    SingleQuote,
    Backtick,
}

impl Kind {
    pub fn characters(self) -> (char, char) {
        match self {
            Self::Parenthesis => ('(', ')'),
            Self::Bracket => ('[', ']'),
            Self::Brace => ('{', '}'),
            Self::DoubleQuote => ('"', '"'),
            Self::SingleQuote => ('\'', '\''),
            Self::Backtick => ('`', '`'),
        }
    }

    pub fn quote(self) -> bool {
        matches!(self, Self::DoubleQuote | Self::SingleQuote | Self::Backtick)
    }

    pub(crate) fn structural(c: char) -> Option<(Self, bool)> {
        match c {
            '(' => Some((Self::Parenthesis, true)),
            ')' => Some((Self::Parenthesis, false)),
            '[' => Some((Self::Bracket, true)),
            ']' => Some((Self::Bracket, false)),
            '{' => Some((Self::Brace, true)),
            '}' => Some((Self::Brace, false)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    Unknown,
    Code,
    String(Kind),
    Comment,
    Literal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    Lexical,
    CurrentSyntaxAndLexical,
    CurrentSyntax,
    RetainedLexical,
    MappedLexical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Query {
    Typing(Typed),
    Wrap(Wrapping),
    Unwrap(Scalar),
    Context(Scalar),
    At(Scalar),
    Enclosing(Scalar),
}

impl Query {
    pub fn point(self) -> Scalar {
        match self {
            Self::Typing(input) => input.point,
            Self::Wrap(input) => input.start,
            Self::Context(point)
            | Self::At(point)
            | Self::Enclosing(point)
            | Self::Unwrap(point) => point,
        }
    }
}

/// One complete linear region and its admitted delimiter/count. The original
/// source is inspected once; generated delimiters share this request's budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wrapping {
    pub start: Scalar,
    pub end: Scalar,
    pub kind: Kind,
    pub count: usize,
}

/// One physical scalar and its already admitted positive count. Replacement
/// coordinates describe native Replace entry; generated mates never become input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Typed {
    pub point: Scalar,
    pub end: Scalar,
    pub character: char,
    pub count: usize,
    pub skip: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Typing {
    Open { kind: Kind, reused: usize },
    Skip(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair {
    pub opening: Span,
    pub closing: Span,
    pub kind: Kind,
    /// None is reserved for context whose complete nesting is not proved.
    pub depth: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Bytes,
    Depth,
    Retention,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    LiteralContext,
    AmbiguousQuote,
    AmbiguousSlash,
    NotDelimiter,
    NoEnclosingPair,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Wrap(Kind),
    Typing(Typing),
    Context(Context),
    Confirmed(Pair),
    Unmatched,
    Mismatched { expected: Kind, found: Kind },
    Uncertain(Reason),
    Limited(Limit),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub inspected_bytes: usize,
    pub syntax_nodes: usize,
    pub retained_bytes: usize,
    pub maximum_depth: usize,
    pub slices: usize,
    pub maximum_slice_tokens: usize,
    /// An unchanged, already qualified prefix; these are source scalar units.
    pub reused_chars: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub stamp: Stamp,
    pub query: Query,
    pub profile: Profile,
    pub policy_version: u32,
    pub context: Context,
    pub provenance: Provenance,
    pub outcome: Outcome,
    pub stats: Stats,
}

/// A revision and generation, never a retained pointer into source or syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub stamp: Stamp,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Ready,
    Retired,
}

/// Current, bounded syntax observation. No tree node escapes its native owner.
pub(crate) struct Hint {
    pub context: Context,
    pub nodes: usize,
    pub limited: bool,
}
