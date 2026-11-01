use crate::{
    buffer::Stamp,
    input::{EditingState, Focus},
    session::{Action, Session},
    window::ViewId,
};

pub(in crate::session) struct Continuation {
    view: ViewId,
    source: Stamp,
    started: bool,
}

impl Session {
    pub(in crate::session) fn begin_one_normal(&mut self) -> Action {
        if self.editing_state() != Some(EditingState::Insert) {
            return self.message("One Normal command requires the modal Insert state");
        }
        self.one_normal = Some(Continuation {
            view: self.windows.selected_id(),
            source: self.view().point.stamp,
            started: false,
        });
        self.input.set_editing_state(EditingState::Normal);
        self.message("One Normal command")
    }

    /// External source/view changes cannot inherit an insertion continuation.
    /// The completed native command may itself publish an admitted source edit.
    pub(in crate::session) fn qualify_one_normal(&mut self) {
        if self.one_normal.as_ref().is_some_and(|resume| {
            resume.view != self.windows.selected_id() || resume.source != self.view().point.stamp
        }) {
            self.one_normal = None;
            self.input.set_editing_state(EditingState::Normal);
        }
    }

    pub(in crate::session) fn start_one_normal(&mut self) {
        if let Some(resume) = &mut self.one_normal {
            resume.started = true;
        }
    }

    pub(in crate::session) fn complete_one_normal(&mut self) {
        if self.one_normal.is_none() || !self.one_normal_ready() {
            return;
        }
        let resume = self.one_normal.take().expect("checked continuation");
        if resume.view != self.windows.selected_id()
            || resume.source.buffer != self.view().point.stamp.buffer
            || self.tree.focused()
        {
            return;
        }
        if self
            .editing_state()
            .is_some_and(|state| state.visual().is_none())
            && self
                .view()
                .cell(self.buffer().expect("live insertion source"))
                .is_ok()
        {
            self.input.set_editing_state(EditingState::Insert);
        }
    }
    fn one_normal_ready(&self) -> bool {
        self.one_normal
            .as_ref()
            .is_some_and(|resume| resume.started)
            && self.input.focus() == Focus::Editing
            && !self.input.navigation_pending()
            && !self.layout_held()
            && self.bookmark_visit.is_none()
            && self.completion_request.is_none()
            && self.pending.is_none()
    }
}
