//! Private numerical results over an immutable table projection. No evaluated
//! value or row/column number grants source identity or publication authority.
mod aggregate;
mod graph;
mod grid;
mod numeric;
mod plan;
#[cfg(test)]
mod tests;
mod value;
use super::{Component, Coordinate, Expression, Function, Operator, Program, Reference};
use crate::org::Cancellation;
use std::{fmt, ops::Range};

/// Data rows only; coordinates ignore hlines. `header_rows` is the number of
/// data rows above the first hline separating header and body. Use zero when
/// there is no separating hline. Ragged fields are observed as empty values.
#[derive(Debug, Clone)]
pub struct Grid {
    cells: Vec<String>,
    rows: usize,
    columns: usize,
    header_rows: usize,
}
/// The native finite numeric profile. Cells: 1..=1000000; steps:
/// 1..=10000000; significant digits: 1..=15; decimal places: 0..=15.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    pub cells: usize,
    pub steps: usize,
    pub significant_digits: usize,
    pub decimal_places: usize,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            cells: 100000,
            steps: 1000000,
            significant_digits: 12,
            decimal_places: 12,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// One-based data row, excluding separators.
    pub row: usize,
    /// One-based column.
    pub column: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ResultCell {
    pub cell: Cell,
    pub value: f64,
    pub text: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Reference,
    Numeric,
    DivisionByZero,
    NonFinite,
    Cycle,
    EmptyAggregate,
    Limit,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub cell: Option<Cell>,
    /// Half-open bytes in the selected original formula value.
    pub source: Range<usize>,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Org calculation {:?}", self.kind)?;
        if let Some(cell) = self.cell {
            write!(f, " at @{}${}", cell.row, cell.column)?;
        }
        write!(
            f,
            " bytes {}..{}: {}",
            self.source.start, self.source.end, self.message
        )
    }
}
impl std::error::Error for Error {}
impl Error {
    fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            cell: None,
            source: 0..0,
            message: message.into(),
        }
    }
    fn located(mut self, cell: Cell, source: Range<usize>) -> Self {
        self.cell = Some(cell);
        if self.source.is_empty() {
            self.source = source;
        }
        self
    }
    fn at_source(mut self, source: Range<usize>) -> Self {
        self.source = source;
        self
    }
}
struct Work<'a> {
    remaining: usize,
    cancel: &'a Cancellation,
}
impl Work<'_> {
    fn charge(&mut self, amount: usize) -> Result<(), Error> {
        if self.cancel.is_cancelled() {
            return Err(Error::new(ErrorKind::Cancelled, "cancelled or timed out"));
        }
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or_else(|| Error::new(ErrorKind::Limit, "formula work budget exhausted"))?;
        Ok(())
    }
}
#[derive(Clone, Copy)]
enum Input {
    Number(f64),
    Text,
}
#[derive(Clone, Copy)]
enum Value {
    Number(f64),
    Text,
    Range(grid::Rectangle),
}
struct Engine<'a> {
    grid: &'a Grid,
    program: &'a Program,
    policy: Policy,
    work: Work<'a>,
    owners: Vec<Option<usize>>,
    inputs: Vec<Option<Input>>,
    values: Vec<Option<f64>>,
}
impl Program {
    /// Evaluate the selected assignment set without mutating input. Field
    /// assignments override column assignments; repeated definitions use the
    /// last definition of the same field/column. Returned cells are row-major.
    /// Any failure drops the complete private result set. Formatting does not
    /// round dependency values; explicit round() does.
    pub fn evaluate(
        &self,
        grid: &Grid,
        policy: Policy,
        cancel: &Cancellation,
    ) -> Result<Vec<ResultCell>, Error> {
        policy.admit()?;
        if grid.cells.len() > policy.cells {
            return Err(Error::new(ErrorKind::Limit, "formula cell limit exceeded"));
        }
        let mut work = Work {
            remaining: policy.steps,
            cancel,
        };
        work.charge(1)?;
        let mut engine = Engine {
            grid,
            program: self,
            policy,
            work,
            owners: Vec::new(),
            inputs: vec![None; grid.cells.len()],
            values: vec![None; grid.cells.len()],
        };
        engine.run()
    }
}
impl Engine<'_> {
    fn run(&mut self) -> Result<Vec<ResultCell>, Error> {
        self.owners = self.owners()?;
        let order = self.order()?;
        for cell in order {
            let assignment =
                &self.program.assignments[self.owners[cell].expect("scheduled formula")];
            let number = self
                .expression(cell, assignment.expression)
                .map_err(|e| e.located(self.grid.location(cell), assignment.source.clone()))?;
            self.values[cell] = Some(number);
        }
        self.results()
    }
    fn results(&mut self) -> Result<Vec<ResultCell>, Error> {
        let mut results = Vec::new();
        for (cell, value) in self
            .values
            .iter()
            .enumerate()
            .filter_map(|(c, v)| v.map(|v| (c, v)))
        {
            self.work.charge(1)?;
            let assignment =
                &self.program.assignments[self.owners[cell].expect("evaluated formula")];
            let text = numeric::format(value, assignment.decimals, self.policy)
                .map_err(|e| e.located(self.grid.location(cell), assignment.source.clone()))?;
            self.work.charge(text.len())?;
            results.push(ResultCell {
                cell: self.grid.location(cell),
                value,
                text,
            });
        }
        Ok(results)
    }
}
impl Policy {
    fn admit(self) -> Result<(), Error> {
        let valid = (1..=1000000).contains(&self.cells)
            && (1..=10000000).contains(&self.steps)
            && (1..=15).contains(&self.significant_digits)
            && self.decimal_places <= 15;
        if !valid {
            return Err(Error::new(
                ErrorKind::Limit,
                "invalid formula numeric policy",
            ));
        }
        Ok(())
    }
}
