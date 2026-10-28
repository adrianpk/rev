mod model;
mod numbering;
mod operation;

use std::ops::Range;

use crate::buffer::{Buffer, Span, edit::Replacement};
use crate::coordinates::Byte;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Lists {
    complete: bool,
    items: Vec<Item>,
    lists: Vec<List>,
    editable: Vec<Range<usize>>,
}

#[derive(Debug, PartialEq, Eq)]
struct List {
    source: Range<usize>,
    parent: Option<usize>,
    items: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq)]
struct Item {
    source: Range<usize>,
    header: Range<usize>,
    bullet: Range<usize>,
    content: usize,
    indent: usize,
    column: usize,
    list: usize,
    number: Option<u64>,
    reset: Option<u64>,
    description: bool,
    checkbox: Option<(Range<usize>, u8)>,
}

pub(crate) struct ListEdit {
    pub(crate) edits: Vec<Replacement>,
    pub(crate) order: Vec<Span>,
    pub(crate) cursor: Option<crate::coordinates::Scalar>,
}

// Every plan requalifies the worker's offsets through canonical source. These
// temporary replacements are consumed by one ordinary checked transaction.
fn replacement(buffer: &Buffer, bytes: Range<usize>, text: String) -> Result<Replacement, String> {
    let source = super::Source::new(buffer)
        .range(Byte(bytes.start)..Byte(bytes.end))
        .map_err(|error| error.to_string())?;
    Ok(Replacement {
        source: source.span().clone(),
        text,
    })
}

fn source_text(buffer: &Buffer, range: Range<usize>) -> Result<String, String> {
    let span = buffer.byte_span(range).map_err(|error| error.to_string())?;
    Ok(buffer
        .slice(&span)
        .map_err(|error| error.to_string())?
        .to_string())
}
