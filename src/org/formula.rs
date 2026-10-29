//! Bounded, data-only Org table formula syntax. All byte ranges are relative to
//! the supplied expression text; they never grant buffer or publication authority.
/// Native numerical evaluation over immutable data projections.
pub mod eval;
mod lex;
mod parse;
mod reference;
use crate::org::Cancellation;
use std::{fmt, ops::Range};

/// A finite syntax arena; child indices precede their parent. Parsing has no
/// evaluation, file, process, configuration or source mutation side effects.
#[derive(Debug, Clone)]
pub struct Program {
    nodes: Vec<Node>,
    assignments: Vec<Assignment>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub target: Reference,
    pub expression: usize,
    pub decimals: Option<usize>,
    pub source: Range<usize>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub expression: Expression,
    pub source: Range<usize>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    Field(Reference),
    Range(Reference, Reference),
    Unary(Operator, usize),
    Binary(Operator, usize, usize),
    Group(usize),
    Call(Function, Vec<usize>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    Sum,
    Mean,
    Minimum,
    Maximum,
    Round,
    Abs,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub row: Option<Component>,
    pub column: Option<Component>,
    pub source: Range<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub coordinate: Coordinate,
    pub source: Range<usize>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coordinate {
    Current,
    Absolute(usize),
    Relative(i64),
}
/// Resource settings for syntax only. Table dimensions and dependency/evaluation
/// work remain the responsibility of their operation's qualified table snapshot.
/// Bytes: 1..=33554432; steps: 1..=10000000; recursion depth: 1..=256;
/// decimal places: 0..=15. Defaults use the frozen native profile. Syntax also
/// has fixed ceilings of 100000 tokens and 100000 arena nodes.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub bytes: usize,
    pub steps: usize,
    pub depth: usize,
    pub decimal_places: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: 8388608,
            steps: 1000000,
            depth: 64,
            decimal_places: 12,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Limit,
    Cancelled,
}
/// Exact half-open UTF-8 byte position in the supplied formula value, including
/// an empty EOF range when an operand or closing delimiter is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub source: Range<usize>,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Org formula {:?} at bytes {}..{}: {}",
            self.kind, self.source.start, self.source.end, self.message
        )
    }
}
impl std::error::Error for Error {}
impl Error {
    fn new(kind: ErrorKind, source: Range<usize>, message: impl Into<String>) -> Self {
        Self {
            kind,
            source,
            message: message.into(),
        }
    }
}
impl Program {
    pub fn parse(text: &str, limits: Limits, cancel: &Cancellation) -> Result<Self, Error> {
        limits.admit()?;
        parse::Parser::new(text, limits, cancel)?.program()
    }
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    pub fn assignments(&self) -> &[Assignment] {
        &self.assignments
    }
}
impl Function {
    fn named(name: &str) -> Option<Self> {
        match name {
            "round" => Some(Self::Round),
            "abs" => Some(Self::Abs),
            other => Self::aggregate(other.strip_prefix('v').unwrap_or(other)),
        }
    }
    fn aggregate(name: &str) -> Option<Self> {
        match name {
            "sum" => Some(Self::Sum),
            "mean" => Some(Self::Mean),
            "min" => Some(Self::Minimum),
            "max" => Some(Self::Maximum),
            _ => None,
        }
    }
    fn arity(self, count: usize) -> bool {
        match self {
            Self::Abs => count == 1,
            Self::Round => (1..=2).contains(&count),
            _ => count > 0,
        }
    }
}

impl Limits {
    fn admit(self) -> Result<(), Error> {
        let valid = (1..=33554432).contains(&self.bytes)
            && (1..=10000000).contains(&self.steps)
            && (1..=256).contains(&self.depth)
            && self.decimal_places <= 15;
        if !valid {
            return Err(Error::new(
                ErrorKind::Limit,
                0..0,
                "invalid formula resource limits",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // The worker's private deadline token reaches formula parsing unchanged;
    // expiration cannot yield a syntax observation that later authorizes work.
    #[test]
    fn formula_deadline() {
        let expired = Cancellation::default().bounded(std::time::Instant::now());
        let error = Program::parse("$2=$1+1", Limits::default(), &expired).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Cancelled);
    }
}
