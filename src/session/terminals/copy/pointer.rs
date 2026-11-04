use super::{Key, Session, State};
use crate::{term::SessionId, view::Size};
use alacritty_terminal::index::Point;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

impl Session {
    pub(in crate::session::terminals) fn terminal_copy_pointer(
        &mut self,
        key: Key,
        id: SessionId,
        event: MouseEvent,
        cell: Option<Point>,
    ) -> Result<bool, String> {
        if !requested(event.kind) {
            return Ok(false);
        }
        let fresh = !self.terminals.copies.contains_key(&key);
        if fresh {
            if !starts_inspection(event.kind) {
                return Ok(false);
            }
            self.capture_pointer_copy(key, id, event, cell)?;
        }
        let size = self
            .copy_size(key)
            .ok_or("Terminal copy presentation is suspended")?;
        let copy = self
            .terminals
            .copies
            .get_mut(&key)
            .expect("copy pointer context");
        copy.inspect_pointer(
            event,
            size,
            self.buffers.snapshot().settings().mouse.scroll_lines,
            fresh,
        )?;
        Ok(true)
    }
    fn capture_pointer_copy(
        &mut self,
        key: Key,
        id: SessionId,
        event: MouseEvent,
        cell: Option<Point>,
    ) -> Result<(), String> {
        let mut viewport = self.copy_viewport(key);
        if event.kind == MouseEventKind::Down(MouseButton::Left) {
            viewport.select(
                cell.ok_or("Terminal pointer has no live cell observation")?,
                false,
            );
        }
        // Capture against the old VT coordinates before publishing a projection.
        self.admit_copy_at(key, id, viewport)
    }
}

impl State {
    fn inspect_pointer(
        &mut self,
        event: MouseEvent,
        size: Size,
        lines: usize,
        fresh: bool,
    ) -> Result<(), String> {
        if self.keep_pointer_observation(event, fresh) {
            return Ok(());
        }
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => self.pointer(event, false),
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
                self.pointer(event, true)
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => self
                .view
                .scroll_viewport(
                    &self.buffer,
                    event.kind == MouseEventKind::ScrollUp,
                    lines,
                    size,
                )
                .map_err(|e| e.to_string()),
            _ => Ok(()),
        }?;
        self.pointer_cell = Some((event.column, event.row));
        Ok(())
    }
    fn keep_pointer_observation(&mut self, event: MouseEvent, fresh: bool) -> bool {
        let cell = (event.column, event.row);
        if fresh && event.kind == MouseEventKind::Down(MouseButton::Left) {
            self.pointer_cell = Some(cell);
            return true;
        }
        event.kind == MouseEventKind::Up(MouseButton::Left) && self.pointer_cell == Some(cell)
    }
}

fn requested(kind: MouseEventKind) -> bool {
    matches!(
        kind,
        MouseEventKind::Down(MouseButton::Left)
            | MouseEventKind::Drag(MouseButton::Left)
            | MouseEventKind::Up(MouseButton::Left)
            | MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
    )
}
fn starts_inspection(kind: MouseEventKind) -> bool {
    matches!(
        kind,
        MouseEventKind::Down(MouseButton::Left) | MouseEventKind::ScrollUp
    )
}
