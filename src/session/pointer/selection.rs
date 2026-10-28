use crate::buffer::edit::Point;
use crate::display::Position;
use crate::panel::Panel;
use crate::pointer::Capture;
use crate::view::Error;

use super::{Action, Session};

/// One live source and the distinction between an armed click and a text drag.
pub(crate) struct Selection {
    capture: Capture,
    dragging: bool,
}

impl Selection {
    pub(super) fn new(capture: Capture) -> Self {
        Self {
            capture,
            dragging: false,
        }
    }
}

impl Session {
    /// End native pointer capture on host mode disable or ownership loss.
    /// The canonical region remains available to ordinary editor commands.
    pub fn cancel_pointer_capture(&mut self) -> bool {
        self.selection.take().is_some()
    }

    pub(super) fn drag_pointer(&mut self, position: Position) -> Action {
        let Some(mut selection) = self.selection.take() else {
            return Action::Wait;
        };
        let Some(panel) = self.capture_panel(selection.capture) else {
            return Action::Wait;
        };
        match self.extend_selection(&mut selection, panel, position) {
            Ok(changed) => {
                self.selection = Some(selection);
                if changed {
                    Action::Redraw
                } else {
                    Action::Wait
                }
            }
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn release_pointer(&mut self, position: Position) -> Action {
        let action = if self
            .selection
            .as_ref()
            .is_some_and(|selection| selection.dragging)
        {
            self.drag_pointer(position)
        } else {
            Action::Wait
        };
        self.selection = None;
        action
    }

    fn capture_panel(&self, capture: Capture) -> Option<Panel> {
        if self.windows.selected_id() != capture.view || self.tree.focused() {
            return None;
        }
        let view = self.windows.get(capture.view).ok()?;
        let buffer = self.buffers.get(capture.anchor.stamp.buffer).ok()?;
        if view.point.stamp != capture.anchor.stamp || buffer.stamp() != capture.anchor.stamp {
            return None;
        }
        self.composition()?
            .panels()
            .iter()
            .find(|(view, _)| *view == capture.view)
            .map(|(_, panel)| *panel)
    }

    fn extend_selection(
        &mut self,
        selection: &mut Selection,
        panel: Panel,
        position: Position,
    ) -> Result<bool, Error> {
        let Some(point) = self.view().hit(self.buffer()?, panel.content(), position)? else {
            return Ok(false);
        };
        if point == self.view().point {
            return Ok(false);
        }
        self.place_point(point)?;
        self.start_selection(selection)?;
        Ok(true)
    }

    fn start_selection(&mut self, selection: &mut Selection) -> Result<(), Error> {
        if !selection.dragging {
            let Point { stamp, .. } = selection.capture.anchor;
            self.buffers
                .get_mut(stamp.buffer)?
                .activate_mark(selection.capture.anchor)?;
            selection.dragging = true;
        }
        Ok(())
    }
}
