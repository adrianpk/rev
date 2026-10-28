mod blocks;
mod elements;
mod lists;
mod outline;
mod rows;
mod tokens;

use std::ops::Range;
use std::time::{Duration, Instant};

use crate::buffer::{Buffer, Error, Stamp};
use crate::coordinates::Byte;
use crate::org::{Cancellation, Coverage, InlineContext, Limit, Limits, Source, SourceRange};
use rows::Row;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Source,
    Example,
    Export,
    Comment,
    Verse,
    Greater,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureKind {
    Heading { level: usize },
    Section,
    Paragraph,
    List { ordered: bool },
    Item { indent: usize },
    Drawer,
    PropertyDrawer,
    Property { additive: bool },
    Planning,
    Keyword,
    AffiliatedKeyword,
    Table,
    TableRow { separator: bool },
    TableCell,
    TableFormula,
    Block(BlockKind),
    Malformed,
    Comment,
    FixedWidth,
}

/// Structural ownership is distinct from content used for inline recognition.
/// Heading source includes descendants; its content is the original title line.
/// Delimiters and affiliated metadata always retain their own exact ranges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureNode {
    pub kind: StructureKind,
    pub source: SourceRange,
    pub content: SourceRange,
    pub opening: Option<SourceRange>,
    pub closing: Option<SourceRange>,
    pub name: Option<SourceRange>,
    pub affiliated: Option<SourceRange>,
    pub parent: Option<usize>,
    pub context: Option<InlineContext>,
    pub complete: bool,
}

struct Raw {
    kind: StructureKind,
    source: Range<usize>,
    content: Range<usize>,
    opening: Option<Range<usize>>,
    closing: Option<Range<usize>>,
    name: Option<Range<usize>>,
    affiliated: Option<Range<usize>>,
    parent: Option<usize>,
    context: Option<InlineContext>,
    depth: usize,
}

impl Raw {
    fn new(kind: StructureKind, source: Range<usize>, parent: Option<usize>) -> Self {
        Self {
            kind,
            content: source.clone(),
            source,
            parent,
            opening: None,
            closing: None,
            name: None,
            affiliated: None,
            context: None,
            depth: 0,
        }
    }
}

/// Revision-bound offsets only. Incomplete observations cannot certify a whole
/// subtree even when some individual source ranges were already recognized.
pub struct Structure {
    stamp: Stamp,
    coverage: Coverage,
    covered: usize,
    nodes: Vec<Raw>,
}

impl Structure {
    pub fn stamp(&self) -> Stamp {
        self.stamp
    }
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    pub fn covered_bytes(&self) -> Byte {
        Byte(self.covered)
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, buffer: &Buffer, index: usize) -> Result<StructureNode, Error> {
        super::source::current(buffer, self.stamp)?;
        let raw = self.nodes.get(index).ok_or(Error::Bounds {
            start: index,
            end: index.saturating_add(1),
            len: self.nodes.len(),
        })?;
        qualify(
            Source::new(buffer),
            raw,
            self.coverage == Coverage::Complete,
        )
    }
}

fn qualify(source: Source<'_>, raw: &Raw, complete: bool) -> Result<StructureNode, Error> {
    let span = |range: &Range<usize>| source.range(Byte(range.start)..Byte(range.end));
    Ok(StructureNode {
        kind: raw.kind,
        source: span(&raw.source)?,
        content: span(&raw.content)?,
        opening: raw.opening.as_ref().map(span).transpose()?,
        closing: raw.closing.as_ref().map(span).transpose()?,
        name: raw.name.as_ref().map(span).transpose()?,
        affiliated: raw.affiliated.as_ref().map(span).transpose()?,
        parent: raw.parent,
        context: raw.context,
        complete: complete && raw.kind != StructureKind::Malformed,
    })
}

/// Explicit owned analysis, suitable for the shared Job owner. No buffer, file,
/// executable block or configuration authority is transferred to the worker.
pub struct StructureScan {
    text: ropey::Rope,
    stamp: Stamp,
    limits: Limits,
}

impl StructureScan {
    pub fn new(buffer: &Buffer, limits: Limits) -> Result<Self, Error> {
        Ok(Self {
            text: Source::new(buffer).snapshot()?,
            stamp: buffer.stamp(),
            limits,
        })
    }

    pub fn run(self, cancellation: &Cancellation) -> Structure {
        let mut work = Work {
            cancellation,
            yielded: Instant::now(),
        };
        let mut result = Structure {
            stamp: self.stamp,
            coverage: Coverage::Complete,
            covered: 0,
            nodes: Vec::new(),
        };
        let rows = rows::collect(&self.text, self.limits, &mut work, &mut result);
        let mut parser = Parser {
            rows,
            result,
            work,
            limits: self.limits,
        };
        if let Err(Stop(coverage)) = parser.outline() {
            parser.result.coverage = coverage;
        }
        parser.result
    }
}

struct Stop(Coverage);
type Parsed<T> = Result<T, Stop>;

struct Work<'a> {
    cancellation: &'a Cancellation,
    yielded: Instant,
}

impl Work<'_> {
    fn tick(&mut self) -> Parsed<()> {
        if self.cancellation.is_cancelled() {
            return Err(Stop(Coverage::Cancelled));
        }
        if self.yielded.elapsed() >= Duration::from_millis(10) {
            std::thread::yield_now();
            self.yielded = Instant::now();
        }
        Ok(())
    }
}

struct Parser<'a> {
    rows: Vec<Row>,
    result: Structure,
    work: Work<'a>,
    limits: Limits,
}

impl Parser<'_> {
    fn push(&mut self, mut raw: Raw) -> Parsed<usize> {
        self.work.tick()?;
        raw.depth = raw.parent.map_or(0, |id| self.result.nodes[id].depth + 1);
        if raw.depth >= self.limits.depth {
            return Err(Stop(Coverage::Incomplete(Limit::SourceDepth)));
        }
        if self.result.nodes.len() == self.limits.nodes {
            return Err(Stop(Coverage::Incomplete(Limit::SourceNodes)));
        }
        let id = self.result.nodes.len();
        self.result.nodes.push(raw);
        Ok(id)
    }

    fn range(&self, rows: Range<usize>) -> Range<usize> {
        self.rows[rows.start].source.start..self.rows[rows.end - 1].source.end
    }
}
