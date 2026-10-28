use std::time::{Duration, Instant};

use crate::coordinates::{Cell, Coordinates};
use crate::display::Position;
use crate::panel::Panel;
use crate::view::Error;
use crate::window::Area;

use super::selection::Selection;
use super::{Action, Session};

const INTERVAL: Duration = Duration::from_millis(100);

/// One current edge and deadline; motion replaces its position, never queues work.
pub(super) struct Autoscroll {
    position: Position,
    backward: bool,
    due: Instant,
}

impl Autoscroll {
    fn blocked(&self, top: usize, last: usize) -> bool {
        if self.backward { top == 0 } else { top == last }
    }
}

impl Selection {
    pub(super) fn track(&mut self, position: Position, content: Area, now: Instant) {
        let Some(backward) = direction(position, content) else {
            self.autoscroll = None;
            return;
        };
        let due = self
            .autoscroll
            .as_ref()
            .filter(|pending| pending.backward == backward)
            .map_or(now + INTERVAL, |pending| pending.due);
        self.autoscroll = Some(Autoscroll {
            position,
            backward,
            due,
        });
    }
}

fn direction(position: Position, content: Area) -> Option<bool> {
    if content.height() == 0 {
        return None;
    }
    if position.row < content.row() {
        return Some(true);
    }
    (position.row >= content.row() + content.height()).then_some(false)
}

impl Session {
    pub(in crate::session) fn pointer_remaining(&self, now: Instant) -> Option<Duration> {
        let selection = self.selection.as_ref()?;
        self.capture_panel(selection.capture)?;
        Some(
            selection
                .autoscroll
                .as_ref()?
                .due
                .saturating_duration_since(now),
        )
    }

    pub(in crate::session) fn expire_pointer(&mut self, now: Instant) -> Action {
        let Some(mut selection) = self.selection.take() else {
            return Action::Wait;
        };
        let Some(panel) = self.capture_panel(selection.capture) else {
            return Action::Wait;
        };
        let action = self.pointer_tick(&mut selection, panel, now);
        self.selection = Some(selection);
        action
    }

    fn pointer_tick(&mut self, selection: &mut Selection, panel: Panel, now: Instant) -> Action {
        let Some(pending) = selection.autoscroll.as_ref() else {
            return Action::Wait;
        };
        if now < pending.due {
            return Action::Wait;
        }
        match self.scroll_selection(selection, panel) {
            Ok(action) => {
                if let Some(pending) = selection.autoscroll.as_mut() {
                    pending.due = now + INTERVAL;
                }
                action
            }
            Err(error) => {
                selection.autoscroll = None;
                self.error(&error.to_string())
            }
        }
    }

    fn scroll_selection(
        &mut self,
        selection: &mut Selection,
        panel: Panel,
    ) -> Result<Action, Error> {
        let Some(pending) = selection.autoscroll.as_ref() else {
            return Ok(Action::Wait);
        };
        let Some(candidate) = self.drag_view(panel, pending)? else {
            selection.autoscroll = None;
            return Ok(Action::Wait);
        };
        let before = self.view();
        let changed = candidate != *before;
        if pending.blocked(candidate.top, self.buffer()?.len_lines() - 1) {
            selection.autoscroll = None;
        }
        if changed {
            self.start_selection(selection)?;
        }
        *self.windows.selected_mut() = candidate;
        Ok(if changed {
            Action::Redraw
        } else {
            Action::Wait
        })
    }

    fn drag_view(
        &self,
        panel: Panel,
        pending: &Autoscroll,
    ) -> Result<Option<crate::view::View>, Error> {
        let buffer = self.buffer()?;
        let mut candidate = self.view().clone();
        let size = panel.outer().view_size();
        let distance = super::scroll::SCROLL_LINES.min(usize::from(panel.content().height()));
        candidate.scroll_viewport(buffer, pending.backward, distance, size)?;
        let text = candidate.gutter_geometry(buffer, panel.content())?.text();
        if text.width() == 0 {
            return Ok(None);
        }
        candidate.point.position =
            Coordinates::new(buffer)?.from_cell(edge_cell(&candidate, buffer, text, pending))?;
        candidate.reset_navigation();
        candidate.reveal(buffer, size)?;
        Ok(Some(candidate))
    }
}

fn edge_cell(
    view: &crate::view::View,
    buffer: &crate::buffer::Buffer,
    text: Area,
    pending: &Autoscroll,
) -> Cell {
    let offset = if pending.backward {
        0
    } else {
        usize::from(text.height()) - 1
    };
    let row = view.top.saturating_add(offset).min(buffer.len_lines() - 1);
    let local = pending
        .position
        .column
        .saturating_sub(text.column())
        .min(text.width() - 1);
    Cell {
        row,
        column: view.left.saturating_add(usize::from(local)),
    }
}
