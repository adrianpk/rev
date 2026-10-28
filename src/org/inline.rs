mod bracket;
mod entity;
mod link;
mod markup;
mod read;

use std::ops::Range;

use super::{Cancellation, Coverage, Limit, Limits, Source, SourceRange};
use crate::buffer::{Buffer, Error, Stamp};
use crate::coordinates::Byte;
use read::{Read, Stop};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkKind {
    Bracket,
    Angle,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineKind {
    Bold,
    Italic,
    Underline,
    Strike,
    Code,
    Verbatim,
    Entity,
    Link(LinkKind),
    Target,
    RadioTarget,
    FootnoteReference,
    FootnoteInline,
}

/// Structural recognition supplies the enclosing context explicitly. Literal
/// regions never recognize inline objects; minimal contexts exclude links/notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineContext {
    Standard,
    Minimal,
    Literal,
}

/// Every range retains exact source spelling, including delimiter escapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineNode {
    pub kind: InlineKind,
    pub source: SourceRange,
    pub content: SourceRange,
    pub detail: Option<SourceRange>,
    pub parent: Option<usize>,
}

struct Raw {
    kind: InlineKind,
    source: Range<usize>,
    content: Range<usize>,
    detail: Option<Range<usize>>,
    parent: Option<usize>,
}

impl Raw {
    fn new(kind: InlineKind, source: Range<usize>, content: Range<usize>) -> Self {
        Self {
            kind,
            source,
            content,
            detail: None,
            parent: None,
        }
    }
}

/// Derived spans only; reading a result revalidates its canonical revision.
pub struct InlineObservation {
    stamp: Stamp,
    coverage: Coverage,
    nodes: Vec<Raw>,
}

impl InlineObservation {
    pub fn stamp(&self) -> Stamp {
        self.stamp
    }
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, buffer: &Buffer, index: usize) -> Result<InlineNode, Error> {
        super::source::current(buffer, self.stamp)?;
        let raw = self.nodes.get(index).ok_or(Error::Bounds {
            start: index,
            end: index.saturating_add(1),
            len: self.nodes.len(),
        })?;
        qualify(Source::new(buffer), raw)
    }
}

fn qualify(source: Source<'_>, raw: &Raw) -> Result<InlineNode, Error> {
    let span = |range: &Range<usize>| source.range(Byte(range.start)..Byte(range.end));
    Ok(InlineNode {
        kind: raw.kind,
        source: span(&raw.source)?,
        content: span(&raw.content)?,
        detail: raw.detail.as_ref().map(span).transpose()?,
        parent: raw.parent,
    })
}

/// An owned immutable snapshot for explicit native analysis, usable by Job.
pub struct InlineScan {
    text: ropey::Rope,
    stamp: Stamp,
    range: Range<usize>,
    context: InlineContext,
    limits: Limits,
}

impl InlineScan {
    pub fn new(
        buffer: &Buffer,
        range: &SourceRange,
        context: InlineContext,
        limits: Limits,
    ) -> Result<Self, Error> {
        let source = Source::new(buffer);
        source.text(range)?;
        let text = source.snapshot()?;
        let start = text.char_to_byte(range.span().range.start);
        let end = text.char_to_byte(range.span().range.end);
        Ok(Self {
            text,
            stamp: buffer.stamp(),
            range: start..end,
            context,
            limits,
        })
    }

    pub fn run(self, cancellation: &Cancellation) -> InlineObservation {
        let mut result = InlineObservation {
            stamp: self.stamp,
            coverage: Coverage::Complete,
            nodes: Vec::new(),
        };
        let cap = self.range.start + (self.range.end - self.range.start).min(self.limits.bytes);
        let end = self.text.char_to_byte(self.text.byte_to_char(cap));
        let mut read = Read::new(&self.text, self.range.start..end, cancellation);
        if let Err(Stop(coverage)) = self.scan(&mut read, &mut result) {
            result.coverage = coverage;
        }
        if result.coverage == Coverage::Complete && end < self.range.end {
            result.coverage = Coverage::Incomplete(Limit::AnalysisBytes);
        }
        if end < self.range.end {
            let retained = result
                .nodes
                .iter()
                .position(|node| node.source.end >= end)
                .unwrap_or(result.nodes.len());
            result.nodes.truncate(retained);
        }
        result
    }

    fn scan(&self, read: &mut Read<'_>, result: &mut InlineObservation) -> Result<(), Stop> {
        let mut position = read.start;
        if self.context == InlineContext::Literal {
            return read.tick();
        }
        while position < read.end {
            match candidate(read, position, self.context)? {
                Some(node) => {
                    if result.nodes.len() == self.limits.nodes {
                        return Err(Stop(Coverage::Incomplete(Limit::SourceNodes)));
                    }
                    position = node.source.end;
                    result.nodes.push(node);
                }
                None => position += 1,
            }
        }
        Ok(())
    }
}

fn candidate(
    read: &mut Read<'_>,
    position: usize,
    context: InlineContext,
) -> Result<Option<Raw>, Stop> {
    let byte = read.byte(position)?;
    if b"*/_+=~".contains(&byte.unwrap_or(0)) {
        return markup::recognize(read, position);
    }
    match byte {
        Some(b'\\') => entity::recognize(read, position),
        Some(b'[') if context == InlineContext::Standard => bracket::recognize(read, position),
        Some(b'<') => link::angle(read, position, context),
        _ if context == InlineContext::Standard => link::plain(read, position),
        _ => Ok(None),
    }
}
