use crate::buffer::{BufferId, Buffers};
use crate::completion::{Completion, CompletionSource, source::MAX_BUFFERS};
use crate::input::EditingState;

/// One invocation owner; cancellation retains worker ownership until polling reaps it.
pub(in crate::session) struct State {
    pub(super) popup: Option<Completion>,
    generation: u64,
    request: Option<u64>,
    editing: Option<EditingState>,
    pub(super) source: Option<CompletionSource>,
    pub(super) worker: crate::completion::job::Work,
    pub(super) resolving: Option<super::resolve::Resolving>,
    recent: Vec<BufferId>,
    pub(super) policy: crate::config::Completion,
}

impl State {
    pub(in crate::session) fn new(policy: crate::config::Completion) -> Self {
        Self {
            popup: None,
            generation: 0,
            request: None,
            editing: None,
            source: None,
            worker: crate::completion::job::Work::default(),
            resolving: None,
            recent: Vec::new(),
            policy,
        }
    }

    pub(super) fn admit(&mut self, editing: Option<EditingState>) -> Result<u64, String> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or("Completion identity exhausted")?;
        self.request = Some(self.generation);
        self.editing = editing;
        Ok(self.generation)
    }

    pub(super) fn dismiss(&mut self) {
        if let Some(resolve) = self.resolving.take() {
            resolve.cancel();
        }
        self.source = None;
        self.popup = None;
        self.request = None;
        self.editing = None;
        self.worker.cancel();
    }

    /// Terminal outcomes retire authority while retaining the last source observation.
    pub(super) fn retire_request(&mut self) {
        self.request = None;
    }

    pub(in crate::session) fn has_request(&self) -> bool {
        self.request.is_some()
    }

    pub(in crate::session) fn has_popup(&self) -> bool {
        self.popup.is_some()
    }

    pub(super) fn matches_generation(&self, generation: u64) -> bool {
        self.request == Some(generation)
    }

    pub(super) fn editing_current(&self, editing: Option<EditingState>) -> bool {
        self.editing == editing
    }

    pub(in crate::session) fn owns_source(&self, id: BufferId) -> bool {
        self.worker.owns(id)
    }

    pub(super) fn recency(&self, id: BufferId) -> usize {
        self.recent
            .iter()
            .position(|recent| *recent == id)
            .unwrap_or(MAX_BUFFERS)
    }

    pub(super) fn touch(&mut self, id: BufferId, buffers: &Buffers) {
        self.recent
            .retain(|prior| *prior != id && buffers.get(*prior).is_ok());
        self.recent.insert(0, id);
        self.recent.truncate(MAX_BUFFERS);
    }
}
