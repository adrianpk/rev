use super::*;
use crate::{
    buffer::{Buffer, BufferId, Stamp, State},
    file::{Outcome, Publication},
};
use std::path::PathBuf;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Unchanged,
    ChangedInBuffer,
    Saved,
    Rejected,
    CompletionUnresolved,
    ChangedAfter,
    Closed,
}
#[derive(Clone, Debug)]
pub struct ResultEntry {
    pub buffer: BufferId,
    pub stamp: Stamp,
    pub state: State,
    pub label: String,
    pub role: &'static str,
    pub path: Option<PathBuf>,
    pub status: Status,
    pub error: Option<String>,
}
impl ResultEntry {
    fn new(buffer: &Buffer, role: &'static str, changed: bool) -> Self {
        Self {
            buffer: buffer.id(),
            stamp: buffer.stamp(),
            state: buffer.state(),
            role,
            label: buffer.name().to_owned(),
            path: buffer.file().map(|f| f.path().to_owned()),
            status: if changed {
                Status::ChangedInBuffer
            } else {
                Status::Unchanged
            },
            error: None,
        }
    }
    fn current(&mut self, buffers: &crate::buffer::Buffers) {
        self.status = match buffers.get(self.buffer) {
            Err(_) => Status::Closed,
            Ok(buffer) if buffer.state() != self.state => Status::ChangedAfter,
            _ => self.status,
        }
    }
}
impl Session {
    /// Last operation's actual participants. Later edits or closure invalidate
    /// their pinned state rather than claiming that old save evidence still fits.
    pub fn refile_results(&self) -> Vec<ResultEntry> {
        let mut results = self.refile_result.clone();
        for entry in &mut results {
            entry.current(&self.buffers);
        }
        results
    }
    pub(super) fn record_refile(&mut self, source: BufferId, target: BufferId, copy: bool) {
        self.refile_result.clear();
        let from = self.buffers.get(source).expect("published refile source");
        self.refile_result.push(ResultEntry::new(
            from,
            if source == target {
                "source/destination"
            } else {
                "source"
            },
            !copy || source == target,
        ));
        if source != target {
            let to = self
                .buffers
                .get(target)
                .expect("published refile destination");
            self.refile_result
                .push(ResultEntry::new(to, "destination", true));
        }
    }
    pub(in crate::session) fn show_refile_results(&mut self) -> Action {
        let entries = self.refile_results();
        if entries.is_empty() {
            return self.message("No refile result is retained");
        }
        let text = entries
            .iter()
            .map(|e| format!("{}: {} [{}]", e.role, e.status.message(), e.label))
            .collect::<Vec<_>>()
            .join("; ");
        let errors = entries
            .iter()
            .filter_map(|e| e.error.as_ref().map(|error| format!("{}: {error}", e.role)))
            .collect::<Vec<_>>()
            .join("; ");
        let mut message = format!("{} result: {text}", self.refile_label);
        if !errors.is_empty() {
            message.push_str(&format!("; {errors}"));
        }
        self.message(&message)
    }
    pub(in crate::session) fn note_refile_save(&mut self, saved_state: State, outcome: &Outcome) {
        for entry in &mut self.refile_result {
            let Ok(buffer) = self.buffers.get(entry.buffer) else {
                continue;
            };
            if saved_state != entry.state
                || buffer.state() != entry.state
                || buffer.file().map(|f| f.path()) != Some(outcome.target())
            {
                continue;
            }
            entry.path = Some(outcome.target().to_owned());
            entry.status = match outcome.publication() {
                Publication::Complete => Status::Saved,
                Publication::NotPublished => Status::Rejected,
                Publication::Uncertain => Status::CompletionUnresolved,
            };
            entry.error = outcome.error().map(ToString::to_string);
        }
    }
    pub(in crate::session) fn note_refile_rejection(&mut self, id: BufferId, message: &str) {
        for entry in self.refile_result.iter_mut().filter(|e| e.buffer == id) {
            if self.buffers.get(id).is_ok_and(|b| b.state() == entry.state) {
                entry.status = Status::Rejected;
                entry.error = Some(message.to_owned());
            }
        }
    }
}

impl Status {
    fn message(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::ChangedInBuffer => "unsaved in buffer",
            Self::Saved => "saved",
            Self::Rejected => "save rejected; canonical buffer retained",
            Self::CompletionUnresolved => "published; save completion unresolved",
            Self::ChangedAfter => "changed since move",
            Self::Closed => "buffer closed",
        }
    }
}
