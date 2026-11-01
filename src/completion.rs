//! Disposable native proposals. Canonical buffers retain edit/history ownership.
pub(crate) mod context;
pub(crate) mod edits;
pub(crate) mod job;
mod kind;
pub(crate) mod literal;
pub(crate) mod matching;
pub use kind::Kind;
pub(crate) mod source;
mod target;
mod vocabulary;

use crate::buffer::{Buffer, Span, edit::Point};
use crate::window::ViewId;

pub(crate) const MAX_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_SYMBOLS: usize = 4096;
pub(crate) const MAX_LABEL: usize = 512;
pub(crate) const MAX_CANDIDATES: usize = 200;
pub(crate) const WORK_TIME: std::time::Duration = std::time::Duration::from_millis(50);

#[derive(Clone)]
pub(crate) struct Request {
    pub generation: u64,
    syntax_generation: u64,
    pub view: ViewId,
    pub point: Point,
    source: Span,
    prefix: String,
    original: String,
    byte: usize,
    pub manual: bool,
    pub due: std::time::Instant,
    pub policy: crate::config::Completion,
    pub observations: Vec<source::Observation>,
    pub limited: bool,
    pub context: context::Context,
    pub semantic: Option<crate::lsp::completion::Ticket>,
    pub admitted: bool,
    pub fallback_reason: CompletionFallback,
    pub native_only: bool,
}

impl Request {
    pub(crate) fn capture(
        buffer: &Buffer,
        view: ViewId,
        generation: u64,
        point: Point,
    ) -> Result<Self, String> {
        let (source, prefix) = target::at(buffer, point).map_err(|e| e.to_string())?;
        let original = buffer
            .slice(&source)
            .map_err(|e| e.to_string())?
            .to_string();
        let byte = crate::coordinates::Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(point.position)
            .map_err(|e| e.to_string())?
            .0;
        Ok(Self {
            generation,
            syntax_generation: buffer.syntax_generation(),
            view,
            point,
            source,
            prefix,
            original,
            byte,
            manual: true,
            due: std::time::Instant::now(),
            policy: crate::config::Completion::default(),
            observations: Vec::new(),
            limited: false,
            context: context::Context::Names,
            semantic: None,
            admitted: true,
            fallback_reason: CompletionFallback::Unavailable,
            native_only: false,
        })
    }
    pub(crate) fn complete(self, symbols: Vec<Symbol>) -> Completion {
        let symbols = symbols
            .into_iter()
            .filter(|symbol| self.context.accepts(symbol))
            .collect();
        let mut candidates = matching::candidates(symbols, &self.prefix, &self.original, self.byte);
        self.context.literals(&mut candidates);
        candidates.truncate(self.policy.max_candidates);
        Completion {
            generation: self.generation,
            syntax_generation: self.syntax_generation,
            view: self.view,
            point: self.point,
            source: self.source,
            candidates,
            selected: 0,
            popup_rows: self.policy.popup_rows,
            observations: self.observations,
            source_kind: CompletionSource::Native(self.fallback_reason),
            lsp_version: None,
        }
    }
    pub(crate) fn current(&self, view: ViewId, point: Point) -> bool {
        self.view == view && self.point == point
    }

    pub(crate) fn automatic(&self, buffer: &Buffer) -> bool {
        if self.prefix.trim_start_matches("r#").chars().count() >= self.policy.min_prefix {
            return true;
        }
        if !self.prefix.is_empty() {
            return false;
        }
        context::trigger(buffer, self.point)
    }

    pub(crate) fn prepare(&mut self, syntax: &crate::syntax::Syntax, text: ropey::RopeSlice<'_>) {
        self.context = syntax.completion_context(text, self.byte);
        if let Some(target) = syntax.format_target(text, self.byte) {
            self.source.range =
                text.byte_to_char(target.range.start)..text.byte_to_char(target.range.end);
            self.prefix = target.prefix;
            self.original = target.original;
            self.context = target.context;
        }
    }

    pub(crate) fn org(
        &mut self,
        scan: crate::org::completion::Scan,
        text: &ropey::Rope,
        cancel: &crate::org::Cancellation,
    ) -> Result<Vec<Symbol>, String> {
        let Some(result) = scan.run(text, self.byte, cancel)? else {
            return Ok(Vec::new());
        };
        self.source.range =
            text.byte_to_char(result.range.start)..text.byte_to_char(result.range.end);
        self.prefix = result.prefix;
        self.original = result.original;
        self.context = context::Context::Org(result.kind, result.closing);
        Ok(result.symbols)
    }

    pub(crate) fn semantic_allowed(
        &self,
        source: &source::Source,
        deadline: std::time::Instant,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<bool, String> {
        if self.manual || self.context.structured() {
            return Ok(true);
        }
        if let Some(kind) =
            source
                .syntax
                .external_literal(source.text.slice(..), self.byte, deadline, cancel)?
        {
            return Ok(if kind == "comment" {
                self.policy.in_comments
            } else {
                self.policy.in_strings
            });
        }
        Ok(source.syntax.completion_allowed(self.byte, &self.policy))
    }

    pub(crate) fn allowed(
        &self,
        source: &source::Source,
        deadline: std::time::Instant,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<bool, String> {
        let syntax = &source.syntax;
        if !self.manual
            && let Some(kind) =
                syntax.external_literal(source.text.slice(..), self.byte, deadline, cancel)?
        {
            return Ok(if kind == "comment" {
                self.policy.in_comments
            } else {
                self.policy.in_strings
            });
        }
        Ok(self.context != context::Context::Unsupported
            && (self.context.structured()
                || self.manual
                || syntax.completion_allowed(self.byte, &self.policy)))
    }
}

#[derive(Debug)]
pub(crate) struct Symbol {
    pub name: String,
    pub kind: &'static str,
    pub byte: usize,
    pub origin: usize,
    pub scope: Option<std::ops::Range<usize>>,
}

#[cfg(test)]
mod tests;

/// A display label and literal insertion are separate identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    label: String,
    pub(crate) kind: Kind,
    pub(crate) detail: &'static str,
    pub(crate) text: String,
    range: Option<std::ops::Range<usize>>,
    additional: Vec<crate::lsp::completion::Edit>,
    proposal: Option<std::sync::Arc<crate::lsp::completion::Proposal>>,
}

impl Candidate {
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn detail(&self) -> &str {
        self.detail
    }
}

/// Read-only observation of one view/revision-qualified manual invocation.
#[derive(Clone, Debug)]
pub struct Completion {
    pub(crate) generation: u64,
    syntax_generation: u64,
    view: ViewId,
    point: Point,
    source: Span,
    candidates: Vec<Candidate>,
    selected: usize,
    pub(crate) popup_rows: usize,
    observations: Vec<source::Observation>,
    source_kind: CompletionSource,
    lsp_version: Option<crate::lsp::DocumentVersion>,
}

impl Completion {
    pub(crate) fn sources_current(&self, buffers: &crate::buffer::Buffers) -> bool {
        self.observations
            .iter()
            .all(|source| source.current(buffers))
    }
    #[cfg(test)]
    pub(crate) fn new(
        buffer: &Buffer,
        view: ViewId,
        generation: u64,
        context: (Point, Vec<Symbol>),
    ) -> Result<Self, String> {
        let (point, symbols) = context;
        Ok(Request::capture(buffer, view, generation, point)?.complete(symbols))
    }

    pub fn source(&self) -> CompletionSource {
        self.source_kind
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }
    pub fn selected(&self) -> Option<usize> {
        (!self.candidates.is_empty()).then_some(self.selected)
    }

    pub(crate) fn navigate(&mut self, previous: bool) {
        if self.candidates.is_empty() {
            return;
        }
        self.selected = if previous {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(self.candidates.len() - 1)
        };
    }

    pub(crate) fn current(&self, buffer: &Buffer, view: ViewId, point: Point) -> bool {
        self.view == view
            && self.point == point
            && self.point.stamp == buffer.stamp()
            && self.syntax_generation == buffer.syntax_generation()
    }

    pub(crate) fn replacement(
        &self,
        buffer: &Buffer,
    ) -> Result<crate::buffer::edit::Replacement, String> {
        buffer.slice(&self.source).map_err(|e| e.to_string())?;
        let candidate = self
            .candidates
            .get(self.selected)
            .ok_or("No completion selected")?;
        if candidate.text.len() > 64 * 1024 {
            return Err("Completion insertion exceeds 64 KiB".into());
        }
        Ok(crate::buffer::edit::Replacement {
            source: Span {
                stamp: self.source.stamp,
                range: candidate
                    .range
                    .clone()
                    .unwrap_or_else(|| self.source.range.clone()),
            },
            text: candidate.text.clone(),
        })
    }
}

pub use crate::lsp::completion::{Fallback as CompletionFallback, Source as CompletionSource};

impl Request {
    pub(crate) fn semantic_target(&self) -> (usize, std::ops::Range<usize>, String) {
        (
            self.point.position.0,
            self.source.range.clone(),
            self.prefix.clone(),
        )
    }
    pub(crate) fn semantic_completion(self, reply: crate::lsp::completion::Reply) -> Completion {
        let limit = self.policy.max_candidates;
        let mut completion = self.complete(Vec::new());
        completion.lsp_version = reply.version;
        completion.source_kind = reply.source;
        completion.candidates = reply
            .items
            .into_iter()
            .map(|item| {
                let (kind, detail) = Kind::semantic(item.kind);
                Candidate {
                    label: item.label,
                    kind,
                    detail,
                    text: item.text,
                    range: Some(item.range),
                    additional: item.additional,
                    proposal: item.proposal,
                }
            })
            .take(limit)
            .collect();
        completion
    }
}
impl Completion {
    pub(crate) fn fallback(&mut self, source: CompletionSource) {
        self.source_kind = source;
    }
    pub(crate) fn lsp_current(&self, client: &crate::lsp::Client) -> bool {
        self.lsp_version
            .is_none_or(|version| client.document_version(version.stamp.buffer) == Some(version))
    }
}
