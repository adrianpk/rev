pub(crate) mod assets;
mod catalog;
mod completion;
mod formats;
mod scope;
mod source;
mod style;
mod web;

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

/// Owns one optional language parser/query/tree. Setup or parse failure is local to
/// styling; the canonical buffer and its edit/history admission are independent.
#[derive(Default)]
pub struct Syntax {
    parsed: Option<Parsed>,
    stamp: Option<Stamp>,
    source: Option<Stamp>,
    incremental: bool,
    working: bool,
}

struct Parsed {
    grammar: crate::config::Grammar,
    language: tree_sitter::Language,
    parser: Parser,
    query: std::sync::Arc<Query>,
    completion: Option<std::sync::Arc<Query>>,
    external: bool,
    tree: Option<Tree>,
}

impl Syntax {
    /// Compile a Rust highlight query. Missing, empty, invalid or unsupported
    /// captures produce the ordinary Text fallback, with no runtime installer.
    pub fn rust(query: Option<&str>) -> Self {
        Self {
            parsed: query.and_then(|source| Parsed::new(crate::config::Grammar::Rust, source)),
            stamp: None,
            source: None,
            incremental: false,
            working: false,
        }
    }

    pub(crate) fn bundled(grammar: crate::config::Grammar) -> Self {
        Self {
            parsed: catalog::highlights(grammar).and_then(|query| Parsed::new(grammar, &query)),
            ..Self::default()
        }
    }

    pub(crate) fn external(assets: &assets::Assets) -> Self {
        let mut parser = Parser::new();
        if parser.set_language(&assets.language).is_err() {
            return Self::default();
        }
        Self {
            parsed: Some(Parsed {
                grammar: crate::config::Grammar::Plain,
                language: assets.language.clone(),
                parser,
                query: std::sync::Arc::clone(&assets.highlights),
                completion: Some(std::sync::Arc::clone(&assets.completions)),
                external: true,
                tree: None,
            }),
            ..Self::default()
        }
    }

    pub fn mode(&self, stamp: Stamp) -> &'static str {
        if self.stamp == Some(stamp)
            && self
                .parsed
                .as_ref()
                .is_some_and(|parsed| parsed.tree.is_some())
        {
            let parsed = self.parsed.as_ref().expect("current parser");
            if parsed.external {
                "External"
            } else {
                catalog::labels(parsed.grammar).mode
            }
        } else {
            "Text"
        }
    }

    pub(crate) fn source_mode(&self, stamp: Stamp) -> &'static str {
        if self.mode(stamp) == "Text" {
            return "Org Src";
        }
        let parsed = self.parsed.as_ref().expect("current parser");
        if parsed.external {
            "Org Src External"
        } else {
            catalog::labels(parsed.grammar).source_mode
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
        if self.working {
            if let Some(parsed) = &mut self.parsed {
                parsed.activate_parser();
            }
            // The old worker remains pinned to its old rope/tree. New text
            // gets a current parser, and its retired result cannot replace it.
            self.working = false;
        }
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
        if self.working || self.stamp == Some(stamp) {
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
        if self.mode(buffer.stamp()) == "Text"
            || bytes.start >= bytes.end
            || bytes.end > buffer.len_bytes()
        {
            return Vec::new();
        }
        let parsed = self.parsed.as_ref().expect("current syntax has a parser");
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

impl Syntax {
    pub(crate) fn take_work(&mut self) -> Option<Self> {
        if self.working {
            return None;
        }
        let parsed = self.parsed.as_mut()?;
        let work = Self {
            parsed: Some(parsed.take_parser()),
            stamp: self.stamp,
            source: self.source,
            incremental: self.incremental,
            working: false,
        };
        // Painted revisions need only their immutable tree/query projection.
        // If work starts before the first repaint, let that frame parse current
        // text independently. No old-revision captures survive a source edit.
        self.working = self.stamp.is_some();
        if !self.working {
            parsed.activate_parser();
        }
        Some(work)
    }
    pub(crate) fn working(&self) -> bool {
        self.working
    }
}

impl Parsed {
    fn activate_parser(&mut self) {
        self.parser
            .set_language(&self.language)
            .expect("retained syntax language remains admitted");
    }

    fn take_parser(&mut self) -> Self {
        Self {
            grammar: self.grammar,
            language: self.language.clone(),
            parser: std::mem::replace(&mut self.parser, Parser::new()),
            query: std::sync::Arc::clone(&self.query),
            completion: self.completion.clone(),
            external: self.external,
            tree: self.tree.clone(),
        }
    }

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

    fn new(grammar: crate::config::Grammar, source: &str) -> Option<Self> {
        let language = catalog::language(grammar)?;
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
            grammar,
            parser,
            query: std::sync::Arc::new(query),
            completion: completion::query(grammar, &language).map(std::sync::Arc::new),
            external: false,
            tree: None,
            language,
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
