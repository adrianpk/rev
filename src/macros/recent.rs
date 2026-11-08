use super::*;
use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct Recent {
    pub entries: VecDeque<(Entry, Step)>,
    pub draft: Option<Recorder>,
    bytes: usize,
}
impl Recent {
    pub fn clear(&mut self) {
        self.entries.clear();
        self.draft = None;
        self.bytes = 0;
    }
    pub fn push(&mut self, entry: Entry, step: Step) {
        self.bytes += step.bytes() + std::mem::size_of::<(Entry, Step)>();
        self.entries.push_back((entry, step));
        self.drain();
    }
    pub fn drain(&mut self) {
        if let Some(recorder) = &mut self.draft {
            for step in std::mem::take(&mut recorder.steps) {
                self.bytes = self
                    .bytes
                    .saturating_add(step.bytes() + std::mem::size_of::<(Entry, Step)>());
                self.entries.push_back((recorder.entry, step));
            }
            recorder.bytes = 4096;
        }
        let pending = self.draft.as_ref().map_or(0, |r| {
            r.pending_bytes + r.command.as_ref().map_or(0, CommandStep::bytes)
        });
        while self.entries.len() > 300 || self.bytes.saturating_add(pending) > DEFINITION_BYTES {
            let Some((_, step)) = self.entries.pop_front() else {
                break;
            };
            self.bytes = self
                .bytes
                .saturating_sub(step.bytes() + std::mem::size_of::<(Entry, Step)>());
        }
    }
}
impl Recorder {
    pub fn fresh_pending(&self) -> bool {
        self.pending.is_empty() && self.command.is_none() && !self.selecting
    }
}
