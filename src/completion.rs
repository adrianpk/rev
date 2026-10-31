//! Disposable native proposals. Canonical buffers retain edit/history ownership.
mod matching;
mod target;

use crate::buffer::{Buffer, Span, edit::Point};
use crate::window::ViewId;

pub(crate) const MAX_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_SYMBOLS: usize = 4096;
pub(crate) const MAX_LABEL: usize = 512;
pub(crate) const MAX_CANDIDATES: usize = 200;
pub(crate) const WORK_TIME: std::time::Duration = std::time::Duration::from_millis(50);

#[derive(Debug)]
pub(crate) struct Symbol {
    pub name: String,
    pub kind: &'static str,
    pub byte: usize,
}

#[cfg(test)]
mod tests;

/// A display label and literal insertion are separate identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    label: String,
    text: String,
}

impl Candidate {
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Read-only observation of one view/revision-qualified manual invocation.
#[derive(Clone, Debug)]
pub struct Completion {
    pub(crate) generation: u64,
    view: ViewId,
    point: Point,
    source: Span,
    candidates: Vec<Candidate>,
    selected: usize,
}

impl Completion {
    pub(crate) fn new(
        buffer: &Buffer,
        view: ViewId,
        generation: u64,
        context: (Point, Vec<Symbol>),
    ) -> Result<Self, String> {
        let (point, symbols) = context;
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
        let candidates = matching::candidates(symbols, &prefix, &original, byte);
        Ok(Self {
            generation,
            view,
            point,
            source,
            candidates,
            selected: 0,
        })
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
        self.view == view && self.point == point && self.point.stamp == buffer.stamp()
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
            source: self.source.clone(),
            text: candidate.text.clone(),
        })
    }
}
