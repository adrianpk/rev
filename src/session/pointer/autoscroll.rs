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
    fn blocked(&self, top: usize, first: usize, last: usize) -> bool {
        if self.backward {
            top == first
        } else {
            top == last
        }
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
        match self.pointer_tick(&mut selection, panel, now) {
            Ok(action) => {
                self.selection = Some(selection);
                action
            }
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn pointer_tick(
        &mut self,
        selection: &mut Selection,
        panel: Panel,
        now: Instant,
    ) -> Result<Action, Error> {
        let Some(pending) = selection.autoscroll.as_ref() else {
            return Ok(Action::Wait);
        };
        if now < pending.due {
            return Ok(Action::Wait);
        }
        let action = self.scroll_selection(selection, panel)?;
        if let Some(pending) = selection.autoscroll.as_mut() {
            pending.due = now + INTERVAL;
        }
        Ok(action)
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
        let buffer = self.buffer()?;
        let first = candidate
            .source_at(buffer, 0)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let last = candidate
            .source_at(buffer, candidate.visible_lines(buffer) - 1)
            .ok_or(crate::coordinates::Error::Overflow)?;
        if pending.blocked(candidate.top, first, last) {
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
        let distance = self
            .mouse_scroll_lines()
            .min(usize::from(panel.content().height()));
        candidate.scroll_viewport(buffer, pending.backward, distance, size)?;
        let text = candidate.gutter_geometry(buffer, panel.content())?.text();
        if text.width() == 0 {
            return Ok(None);
        }
        candidate.point.position =
            Coordinates::new(buffer)?.from_cell(edge_cell(&candidate, buffer, text, pending)?)?;
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
) -> Result<Cell, Error> {
    let offset = if pending.backward {
        0
    } else {
        usize::from(text.height()) - 1
    };
    let projected = view
        .projected_top(buffer)
        .saturating_add(offset)
        .min(view.visible_lines(buffer) - 1);
    let row = view
        .source_at(buffer, projected)
        .ok_or(crate::coordinates::Error::Overflow)?;
    let local = pending
        .position
        .column
        .saturating_sub(text.column())
        .min(text.width() - 1);
    Ok(Cell {
        row,
        column: view.left.saturating_add(usize::from(local)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Command, Motion};
    use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

    fn mouse(kind: MouseEventKind, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 7,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    // Every due tick must independently reject a stale source, even if a host
    // changes ownership or the document stamp without the ordinary cancel hook.
    #[test]
    fn stale_capture() {
        for fault in ["view", "document", "point"] {
            let mut session = Session::empty(crate::view::Size {
                width: 80,
                height: 12,
            })
            .unwrap();
            session.handle(Event::Paste("row\n".repeat(50)));
            session.execute(Command::Motion(Motion::BufferStart));
            let now = Instant::now();
            session.handle_at(mouse(MouseEventKind::Down(MouseButton::Left), 0), now);
            session.handle_at(mouse(MouseEventKind::Drag(MouseButton::Left), 10), now);
            if fault == "document" {
                let id = session.view().point.stamp.buffer;
                let buffer = session.buffers.get_mut(id).unwrap();
                let point = session.windows.selected().point;
                let prepared = buffer
                    .prepare(
                        &[crate::buffer::edit::Replacement {
                            source: buffer.span(0..0).unwrap(),
                            text: "X".into(),
                        }],
                        &[point],
                        None,
                    )
                    .unwrap();
                buffer.publish(prepared, &mut [point]).unwrap();
            } else if fault == "view" {
                session.windows.selected_mut().point.stamp.revision += 1;
            } else {
                session.windows.selected_mut().point.position =
                    crate::coordinates::Scalar(usize::MAX);
            }
            let before = session.view().clone();
            let action = if fault == "point" {
                Action::Redraw
            } else {
                Action::Wait
            };
            assert_eq!(session.expire_input(now + INTERVAL), action, "{fault}");
            assert_eq!(session.remaining_input(now + INTERVAL), None);
            assert!(session.selection.is_none());
            assert_eq!(session.view(), &before);
        }
    }
}
