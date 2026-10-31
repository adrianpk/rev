use super::*;
use crate::input::Focus;

impl Session {
    pub(in crate::session) fn toggle_bookmark_markers(&mut self) -> Action {
        if self.input.focus() != Focus::Editing || self.tree.focused() {
            return self.message("Bookmark markers require an editing view");
        }
        let view = self.windows.selected_mut();
        view.bookmark_markers = !view.bookmark_markers;
        self.message("")
    }

    /// First marker demand admits storage at a session transition, before any
    /// frame work. No target is resolved merely because its file is displayed.
    pub(in crate::session) fn prepare_bookmark_markers(&mut self) {
        if self.bookmark_storage.attempted {
            return;
        }
        let demand = self.windows.iter().any(|(_, view)| {
            view.bookmark_markers
                && self
                    .buffers
                    .get(view.point.stamp.buffer)
                    .is_ok_and(|buffer| buffer.file().is_some())
        });
        if demand && let Err(error) = self.ensure_bookmarks() {
            self.notice(&error);
        }
    }
}
