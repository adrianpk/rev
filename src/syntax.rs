mod source;
mod style;

pub use style::Style;

use std::ops::Range;

use ropey::RopeSlice;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator, Tree};

use crate::buffer::{Buffer, Stamp};
use source::Source;

/// Derived byte coordinates are valid only for the exact canonical revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capture {
    pub bytes: Range<usize>,
    pub style: Style,
}

/// Owns one optional Rust parser/query/tree. Setup or parse failure is local to
/// styling; the canonical buffer and its edit/history admission are independent.
#[derive(Default)]
pub struct Syntax {
    parsed: Option<Parsed>,
    stamp: Option<Stamp>,
}

struct Parsed {
    parser: Parser,
    query: Query,
    tree: Option<Tree>,
}

impl Syntax {
    /// Compile a Rust highlight query. Missing, empty, invalid or unsupported
    /// captures produce the ordinary Text fallback, with no runtime installer.
    pub fn rust(query: Option<&str>) -> Self {
        Self {
            parsed: query.and_then(Parsed::new),
            stamp: None,
        }
    }

    pub fn mode(&self, stamp: Stamp) -> &'static str {
        if self.stamp == Some(stamp)
            && self
                .parsed
                .as_ref()
                .is_some_and(|parsed| parsed.tree.is_some())
        {
            "Rust"
        } else {
            "Text"
        }
    }

    /// Read borrowed rope chunks, never a flattened document copy. Refresh at
    /// most once for a revision; obsolete trees cannot supply current captures.
    pub fn update(&mut self, text: RopeSlice<'_>, stamp: Stamp) {
        if self.stamp == Some(stamp) {
            return;
        }
        if let Some(parsed) = &mut self.parsed {
            let mut read = |offset, _| source::chunk(text, offset);
            parsed.tree = parsed.parser.parse_with_options(&mut read, None, None);
        }
        self.stamp = Some(stamp);
    }

    /// Query only the requested visible byte interval. Resource exhaustion
    /// discards this request's styles instead of publishing partial captures.
    pub fn captures(&self, buffer: &Buffer, bytes: Range<usize>) -> Vec<Capture> {
        if self.mode(buffer.stamp()) != "Rust"
            || bytes.start >= bytes.end
            || bytes.end > buffer.len_bytes()
        {
            return Vec::new();
        }
        let parsed = self
            .parsed
            .as_ref()
            .expect("current Rust mode has a parser");
        let text = buffer
            .slice(
                &buffer
                    .span(0..buffer.len_chars())
                    .expect("whole buffer span"),
            )
            .expect("current whole buffer");
        parsed.captures(Source(text), bytes)
    }
}

impl Parsed {
    fn new(source: &str) -> Option<Self> {
        let language = tree_sitter_rust::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&language).ok()?;
        let query = Query::new(&language, source).ok()?;
        if !query
            .capture_names()
            .iter()
            .any(|name| Style::named(name).is_some())
        {
            return None;
        }
        Some(Self {
            parser,
            query,
            tree: None,
        })
    }

    fn captures(&self, source: Source<'_>, bytes: Range<usize>) -> Vec<Capture> {
        let Some(tree) = &self.tree else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        cursor.set_match_limit(4096);
        cursor.set_byte_range(bytes);
        let mut captures = cursor.captures(&self.query, tree.root_node(), source);
        let mut result = Vec::new();
        while let Some((matched, index)) = captures.next() {
            let capture = matched.captures()[*index];
            if let Some(style) = Style::named(self.query.capture_names()[capture.index as usize]) {
                result.push(Capture {
                    bytes: capture.node.byte_range(),
                    style,
                });
            }
            if result.len() > 65_536 {
                return Vec::new();
            }
        }
        if cursor.did_exceed_match_limit() {
            result.clear();
        }
        result
    }
}
