use crate::panel::Panel;
use crate::pointer::{Event, Kind, Target};
use crate::view::Error;
use crate::window::ViewId;

use super::{Action, Session};

// Native factory step; the recorded controls slice admits its startup setting.
pub(super) const SCROLL_LINES: usize = 3;

impl Session {
    pub(super) fn pointer_scroll(&mut self, event: Event) -> Action {
        if self.selection.is_some() {
            return Action::Wait;
        }
        let Some(target) = self
            .composition()
            .and_then(|composition| Target::resolve(&composition, event.position))
        else {
            return Action::Wait;
        };
        let backward = event.kind == Kind::ScrollUp;
        let result = match target {
            Target::Tree(panel) => {
                Ok(self
                    .tree
                    .scroll_viewport(backward, SCROLL_LINES, panel.content().height()))
            }
            Target::Editing { view, panel } => self.scroll_editing(view, panel, backward),
        };
        match result {
            Ok(true) => Action::Redraw,
            Ok(false) => Action::Wait,
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn scroll_editing(&mut self, id: ViewId, panel: Panel, backward: bool) -> Result<bool, Error> {
        let before = self.windows.get(id)?;
        let buffer = self.buffers.get(before.point.stamp.buffer)?;
        let mut candidate = before.clone();
        let size = panel.outer().view_size();
        candidate.reveal(buffer, size)?;
        candidate.scroll_viewport(buffer, backward, SCROLL_LINES, size)?;
        let changed = candidate != *before;
        *self.windows.get_mut(id)? = candidate;
        Ok(changed)
    }
}
