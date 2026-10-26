mod source;
mod style;

pub use style::Style;

use std::ops::Range;

use ropey::RopeSlice;
use tree_sitter::{InputEdit, ParseOptions, Parser, Query, QueryCursor, StreamingIterator, Tree};

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
    source: Option<Stamp>,
    incremental: bool,
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
            source: None,
            incremental: false,
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

    /// Whether the last successful parse reused a tree already edited to the
    /// current source revision. Invalidation and failure clear this evidence.
    pub fn is_incremental(&self) -> bool {
        self.incremental
    }

    /// Edit the retained tree only after text admission succeeds. Its coordinate
    /// revision can advance through several hidden edits without reparsing, but
    /// no captures are current until the parser confirms that revised tree.
    pub(crate) fn edit(&mut self, before: Stamp, after: Stamp, edits: &[InputEdit]) {
        self.stamp = None;
        self.incremental = false;
        if let Some(parsed) = &mut self.parsed {
            parsed.edit(self.source == Some(before), edits);
        }
        self.source = Some(after);
    }

    /// Read borrowed rope chunks, at most once per revision. Only a retained
    /// tree whose coordinates match that revision can seed incremental parsing.
    pub fn update(&mut self, text: RopeSlice<'_>, stamp: Stamp) {
        self.refresh(text, stamp, None);
    }

    /// A caller can bound real parser work with Tree-sitter's progress callback.
    /// Cancellation discards the tree and parser continuation, preserving text.
    pub fn update_with(&mut self, text: RopeSlice<'_>, stamp: Stamp, options: ParseOptions<'_>) {
        self.refresh(text, stamp, Some(options));
    }

    fn refresh(&mut self, text: RopeSlice<'_>, stamp: Stamp, options: Option<ParseOptions<'_>>) {
        if self.stamp == Some(stamp) {
            return;
        }
        self.incremental = false;
        if let Some(parsed) = &mut self.parsed {
            self.incremental = parsed.update(text, self.source == Some(stamp), options);
        }
        self.source = Some(stamp);
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
    fn edit(&mut self, current: bool, edits: &[InputEdit]) {
        if !current {
            self.tree = None;
            return;
        }
        if let Some(tree) = &mut self.tree {
            for edit in edits {
                tree.edit(edit);
            }
        }
    }

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

    fn update(
        &mut self,
        text: RopeSlice<'_>,
        current: bool,
        options: Option<ParseOptions<'_>>,
    ) -> bool {
        let old = self.tree.as_ref().filter(|_| current);
        let reused = old.is_some();
        let mut read = |offset, _| source::chunk(text, offset);
        let tree = self.parser.parse_with_options(&mut read, old, options);
        if tree.is_none() {
            self.parser.reset();
        }
        self.tree = tree;
        reused && self.tree.is_some()
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
