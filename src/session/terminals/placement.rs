use super::{Action, Composition, Geometry, Session, SessionId};
use crate::{
    buffer::Buffer,
    view::View,
    window::{ViewId, Windows},
};

impl Session {
    pub(in crate::session) fn current_terminal(&self) -> Option<SessionId> {
        if let Some(view) = self.terminals.view {
            return self
                .windows
                .get(view)
                .ok()
                .and_then(|v| self.buffers.get(v.point.stamp.buffer).ok())
                .and_then(Buffer::terminal);
        }
        self.terminals.selected
    }

    pub(in crate::session) fn select_terminal(&mut self, id: SessionId) -> Action {
        if self.terminals.buffers.contains(&id) {
            self.terminal_to_buffer(id)
        } else {
            self.terminal_show(id)
        }
    }

    pub(super) fn terminal_move(&mut self, buffer: bool) -> Action {
        let Some(id) = self.current_terminal() else {
            return self.error("No terminal is selected");
        };
        if buffer {
            self.terminal_to_buffer(id)
        } else {
            self.terminal_to_panel(id)
        }
    }

    pub(super) fn buffer_geometry(&self, windows: &Windows, visible: bool) -> Option<Geometry> {
        let rows = if visible {
            self.terminal_geometry()?.rows + 2
        } else {
            0
        };
        let size = crate::view::Size {
            height: self.size.height.checked_sub(rows)?,
            ..self.size
        };
        let composition = Composition::workspace(windows, &self.tree, size, 0)?;
        let (_, panel) = composition
            .panels()
            .iter()
            .find(|(id, _)| *id == windows.selected_id())?;
        let area = panel.content();
        (area.width() >= 2 && area.height() >= 3).then_some(Geometry {
            columns: area.width(),
            rows: area.height(),
        })
    }

    pub(super) fn terminal_to_buffer(&mut self, id: SessionId) -> Action {
        let visible = self.terminals.visible && self.terminals.selected != Some(id);
        let Some(size) = self.buffer_geometry(&self.windows, visible) else {
            return self.error("Terminal buffer requires a usable pane");
        };
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if let Err(error) = self.terminals.sessions.resize(id, size) {
            return self.error(&format!("Move terminal: {error}"));
        }
        let Some(buffer) = self.buffers.iter().find(|b| b.terminal() == Some(id)) else {
            return self.error("Terminal buffer is no longer live");
        };
        let view = self.windows.selected_id();
        if self.buffer().is_ok_and(|b| b.terminal().is_none()) {
            self.terminals
                .back
                .insert(view, self.windows.selected().clone());
        }
        self.windows.selected_mut().rebind(buffer);
        self.terminals.visible = visible;
        self.terminals.buffers.insert(id);
        self.focus_terminal_view(view);
        self.message("Terminal buffer; C-g returns to the editor")
    }

    fn source_view(&self, id: ViewId) -> View {
        if let Some(view) = self.terminals.back.get(&id)
            && let Ok(buffer) = self.buffers.get(view.point.stamp.buffer)
        {
            if view.point.stamp == buffer.stamp() {
                return view.clone();
            }
            return View::with_settings(buffer, self.buffers.snapshot());
        }
        let buffer = self
            .buffers
            .iter()
            .find(|b| b.terminal().is_none())
            .expect("source replacement retained");
        View::with_settings(buffer, self.buffers.snapshot())
    }

    pub(super) fn restore_terminal_view(&mut self) {
        if let Some(id) = self.terminals.view.take() {
            let source = self.source_view(id);
            if let Ok(view) = self.windows.get_mut(id) {
                *view = source;
            }
            self.terminals.back.remove(&id);
        }
    }

    fn terminal_to_panel(&mut self, id: SessionId) -> Action {
        let Some(size) = self.terminal_geometry() else {
            return self.error("Terminal panel does not fit");
        };
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if let Err(error) = self.terminals.sessions.resize(id, size) {
            return self.error(&format!("Move terminal: {error}"));
        }
        let views: Vec<_> = self
            .windows
            .iter()
            .filter(|(_, v)| {
                self.buffers
                    .get(v.point.stamp.buffer)
                    .is_ok_and(|b| b.terminal() == Some(id))
            })
            .map(|(id, _)| id)
            .collect();
        for view in views {
            let source = self.source_view(view);
            *self.windows.get_mut(view).expect("live attachment") = source;
            self.terminals.back.remove(&view);
        }
        self.terminals.buffers.remove(&id);
        self.terminals.view = None;
        self.show_admitted_terminal(id)
    }

    pub(in crate::session) fn focus_terminal_view(&mut self, view: ViewId) {
        self.terminals.view = Some(view);
        self.terminals.focused = true;
        self.tree.edit();
        self.input.cancel();
        self.pending = None;
        self.sync_input_context();
    }

    pub(super) fn resize_terminal_view(&mut self, view: ViewId) -> Action {
        if self.windows.selected_id() != view {
            return Action::Wait;
        }
        let Some(id) = self.current_terminal() else {
            return Action::Wait;
        };
        let Some(size) = self.buffer_geometry(&self.windows, self.terminals.visible) else {
            return Action::Wait;
        };
        if self
            .terminals
            .sessions
            .get(id)
            .is_some_and(|s| s.geometry() == size)
        {
            return Action::Wait;
        }
        match self.terminals.sessions.resize(id, size) {
            Ok(()) => Action::Wait,
            Err(error) => self.error(&format!("Terminal retains its geometry: {error}")),
        }
    }
}
