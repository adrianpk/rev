use alacritty_terminal::{
    grid::Dimensions,
    index::{Column, Line, Point},
    term::{TermMode, cell::Flags},
};
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::{Action, Session, SessionId};
use crate::{
    panel::Panel,
    term::Viewport,
    window::{Area, ViewId},
};

#[derive(Clone, Copy)]
struct Surface {
    id: SessionId,
    view: Option<ViewId>,
    panel: Panel,
}

#[derive(Clone, Copy)]
pub(super) struct Capture {
    id: SessionId,
    view: Option<ViewId>,
    local: bool,
}

impl Surface {
    fn copy_key(self) -> super::copy::Key {
        self.view
            .map_or(super::copy::Key::Panel, super::copy::Key::View)
    }
    fn matches(self, capture: Option<Capture>, event: MouseEvent) -> bool {
        match capture {
            Some(capture) => self.id == capture.id && self.view == capture.view,
            None => contains(self.panel.outer(), event.column, event.row),
        }
    }
}

impl Session {
    pub(super) fn terminal_pointer(&mut self, event: MouseEvent) -> Option<Action> {
        if self.selection.is_some()
            && matches!(event.kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_))
        {
            return None;
        }
        let capture = self.terminal_capture(event.kind);
        let surface = self.find_terminal_surface(|surface| surface.matches(capture, event));
        let Some(surface) = surface else {
            self.cancel_terminal_pointer();
            if matches!(event.kind, MouseEventKind::Down(_)) {
                self.terminal_source_focus();
            }
            return None;
        };
        if matches!(event.kind, MouseEventKind::Down(_)) {
            self.cancel_pointer_capture();
            if let Err(error) = self.focus_terminal_surface(surface) {
                return Some(self.error(&error));
            }
        }
        if capture.is_none() && !contains(surface.panel.content(), event.column, event.row) {
            return Some(Action::Redraw);
        }
        Some(self.route_terminal_pointer(surface, event, capture))
    }

    fn find_terminal_surface(&self, matches: impl Fn(&Surface) -> bool) -> Option<Surface> {
        let composition = self.composition()?;
        for (view, panel) in composition.panels() {
            let id = self
                .windows
                .get(*view)
                .ok()
                .and_then(|v| self.buffers.get(v.point.stamp.buffer).ok())
                .and_then(|b| b.terminal());
            if let Some(id) = id {
                let surface = Surface {
                    id,
                    view: Some(*view),
                    panel: *panel,
                };
                if matches(&surface) {
                    return Some(surface);
                }
            }
        }
        let surface = Surface {
            id: self.terminals.selected?,
            view: None,
            panel: composition.terminal()?,
        };
        matches(&surface).then_some(surface)
    }

    fn focus_terminal_surface(&mut self, surface: Surface) -> Result<(), String> {
        if let Some(view) = surface.view {
            self.select_view(view).map_err(|e| e.to_string())?;
        } else {
            let geometry = self
                .terminal_geometry()
                .ok_or("Terminal presentation is suspended at this screen size")?;
            self.stop_typing().map_err(|e| e.to_string())?;
            self.admit_terminal_geometry(surface.id, geometry)
                .map_err(|e| e.to_string())?;
            self.show_admitted_terminal(surface.id);
        }
        Ok(())
    }

    fn route_terminal_pointer(
        &mut self,
        surface: Surface,
        mut event: MouseEvent,
        capture: Option<Capture>,
    ) -> Action {
        let area = surface.panel.content();
        let Some(session) = self.terminals.sessions.get(surface.id) else {
            return Action::Wait;
        };
        let size = session.geometry();
        let mode = *session.screen().mode();
        event.column = event
            .column
            .saturating_sub(area.column())
            .min(area.width().min(size.columns).saturating_sub(1));
        event.row = event
            .row
            .saturating_sub(area.row())
            .min(area.height().min(size.rows).saturating_sub(1));
        if event.kind == MouseEventKind::Down(MouseButton::Left)
            && event.modifiers.contains(KeyModifiers::CONTROL)
        {
            if self.terminals.copies.contains_key(&surface.copy_key()) {
                return self.terminal_copy_link_cell(surface.copy_key(), event);
            }
            return self.terminal_link_at(surface.id, self.terminal_cell(surface, event));
        }
        let local = capture.map_or_else(
            || self.local_terminal_pointer(surface, event, mode),
            |c| c.local,
        );
        if matches!(event.kind, MouseEventKind::Up(_)) {
            self.cancel_terminal_pointer();
        }
        if local {
            return self.terminal_local_pointer(surface, event);
        }
        let Some(bytes) = super::mouse::encode(event, mode) else {
            return Action::Wait;
        };
        match self.terminals.sessions.queue(surface.id, &bytes) {
            Ok(()) => {
                self.discard_passive_copy(surface.copy_key());
                self.capture_terminal_pointer(surface, event.kind, false);
                Action::Redraw
            }
            Err(error) => self.error(&format!("Terminal pointer input: {error}")),
        }
    }

    fn local_terminal_pointer(&self, surface: Surface, event: MouseEvent, mode: TermMode) -> bool {
        event.modifiers.contains(KeyModifiers::SHIFT)
            || !mode.intersects(TermMode::MOUSE_MODE)
            || self.terminal_viewport(surface).offset > 0
            || self
                .terminals
                .copies
                .get(&surface.copy_key())
                .is_some_and(|copy| copy.mode)
    }

    fn terminal_cell(&self, surface: Surface, event: MouseEvent) -> Point {
        let session = self
            .terminals
            .sessions
            .get(surface.id)
            .expect("live surface");
        let grid = session.screen().grid();
        let offset = self
            .terminal_viewport(surface)
            .offset
            .min(grid.history_size());
        let mut point = Point::new(
            Line(i32::from(event.row) - offset as i32),
            Column(usize::from(event.column)),
        );
        if grid[point].flags.contains(Flags::WIDE_CHAR_SPACER) {
            point.column.0 = point.column.0.saturating_sub(1);
        }
        point
    }

    fn terminal_local_pointer(&mut self, surface: Surface, event: MouseEvent) -> Action {
        let key = surface.copy_key();
        let cell =
            (!self.terminals.copies.contains_key(&key)).then(|| self.terminal_cell(surface, event));
        match self.terminal_copy_pointer(key, surface.id, event, cell) {
            Ok(true) => {}
            Ok(false) => return Action::Wait,
            Err(error) => return self.error(&error),
        }
        self.track_local_cells(surface, event);
        self.capture_terminal_pointer(surface, event.kind, true);
        if matches!(
            event.kind,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
        ) {
            self.scroll_terminal_surface(surface, event.kind);
            if event.kind == MouseEventKind::ScrollDown {
                self.resume_copy_tail(surface.copy_key());
            }
        }
        self.message("")
    }

    fn track_local_cells(&mut self, surface: Surface, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let point = self.terminal_cell(surface, event);
                self.terminal_viewport_mut(surface).select(point, false);
            }
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
                let point = self.terminal_cell(surface, event);
                self.terminal_viewport_mut(surface).select(point, true);
            }
            _ => {}
        }
    }

    fn scroll_terminal_surface(&mut self, surface: Surface, kind: MouseEventKind) {
        let history = self
            .terminals
            .sessions
            .get(surface.id)
            .expect("live surface")
            .screen()
            .grid()
            .history_size();
        let lines = self.buffers.snapshot().settings().mouse.scroll_lines;
        let viewport = self.terminal_viewport_mut(surface);
        viewport.offset = if kind == MouseEventKind::ScrollUp {
            viewport.offset.saturating_add(lines).min(history)
        } else {
            viewport.offset.saturating_sub(lines)
        };
    }

    fn terminal_viewport(&self, surface: Surface) -> Viewport {
        surface.view.map_or(self.terminals.viewport, |view| {
            self.windows.get(view).expect("live surface").terminal
        })
    }

    fn terminal_viewport_mut(&mut self, surface: Surface) -> &mut Viewport {
        match surface.view {
            Some(view) => &mut self.windows.get_mut(view).expect("live surface").terminal,
            None => &mut self.terminals.viewport,
        }
    }

    fn capture_terminal_pointer(&mut self, surface: Surface, kind: MouseEventKind, local: bool) {
        if matches!(kind, MouseEventKind::Down(_)) {
            self.terminals.capture = Some(Capture {
                id: surface.id,
                view: surface.view,
                local,
            });
        }
    }

    fn terminal_capture(&self, kind: MouseEventKind) -> Option<Capture> {
        matches!(kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_))
            .then_some(self.terminals.capture)
            .flatten()
    }

    pub(in crate::session) fn cancel_terminal_pointer(&mut self) {
        self.terminals.capture = None;
    }

    pub(in crate::session) fn terminal_pointer_reporting(&self) -> (bool, bool) {
        let visible = self.find_terminal_surface(|_| true).is_some();
        let motion = self.terminals.focused
            && !self.terminal_host_transient()
            && self
                .current_terminal()
                .and_then(|id| self.terminals.sessions.get(id))
                .is_some_and(|s| s.screen().mode().contains(TermMode::MOUSE_MOTION));
        (self.mouse_enabled() || visible, motion)
    }
}

fn contains(area: Area, column: u16, row: u16) -> bool {
    (area.column()..area.column() + area.width()).contains(&column)
        && (area.row()..area.row() + area.height()).contains(&row)
}
