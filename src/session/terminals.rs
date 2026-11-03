mod input;
mod source;

use std::{path::PathBuf, sync::Arc, time::Duration};

use crate::{
    command::TerminalCommand,
    file::watch::Wake,
    panel::Composition,
    term::{Geometry, Launch, Limits, SessionId, Sessions, State},
    view::Size,
    window::ViewId,
};

use super::{Action, Session};

pub(super) struct Work {
    sessions: Sessions,
    launch: Launch,
    default: Option<SessionId>,
    selected: Option<SessionId>,
    pub(super) visible: bool,
    pub(super) focused: bool,
    pub(super) host_cancel: bool,
    height: u16,
    home: Option<(ViewId, bool)>,
}

impl Work {
    pub(super) fn new(
        prefs: &crate::config::Terminal,
        directory: PathBuf,
        wake: Arc<Wake>,
    ) -> Self {
        let program = if prefs.program.is_empty() {
            std::env::var("SHELL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "/bin/sh".into())
        } else {
            prefs.program.clone()
        };
        let limits = Limits {
            max_sessions: prefs.max_sessions,
            scrollback_lines: prefs.scrollback_lines,
            max_pending_input_bytes: prefs.max_pending_input_bytes,
            max_pending_output_bytes: prefs.max_pending_output_bytes,
        };
        Self {
            sessions: Sessions::new(limits, Arc::new(move || wake.signal()))
                .expect("admitted terminal preferences"),
            launch: Launch {
                program,
                args: prefs.args.clone(),
                directory,
            },
            default: None,
            selected: None,
            visible: false,
            focused: false,
            host_cancel: false,
            height: prefs.panel_height as u16,
            home: None,
        }
    }
}

impl Session {
    pub(super) fn close_terminal(&mut self, id: SessionId) -> std::io::Result<()> {
        self.terminals.sessions.close(id)?;
        if self.terminals.default == Some(id) {
            self.terminals.default = None;
        }
        if self.terminals.selected == Some(id) {
            self.terminals.selected = None;
            self.terminals.visible = false;
            self.terminal_return();
        }
        Ok(())
    }
    pub fn terminals(&self) -> &Sessions {
        &self.terminals.sessions
    }
    pub fn terminal_focused(&self) -> bool {
        self.terminals.focused
    }

    pub fn terminal_buffer(&self) -> Option<&crate::buffer::Buffer> {
        let id = self.terminals.selected?;
        self.buffers.iter().find(|b| b.terminal() == Some(id))
    }

    pub(super) fn terminal_command(&mut self, command: TerminalCommand) -> Action {
        if self.input.focus() != crate::input::Focus::Editing || self.pending.is_some() {
            return self.error("A host prompt owns terminal invocation");
        }
        if command == TerminalCommand::Toggle {
            if self.terminals.visible {
                return self.terminal_hide();
            }
            if let Some(id) = self.terminals.selected {
                return self.terminal_show(id);
            }
        }
        if let Some(id) = self.terminals.default
            && self
                .terminals
                .sessions
                .get(id)
                .is_some_and(|s| *s.state() == State::Running)
        {
            return self.terminal_show(id);
        }
        self.terminal_create()
    }

    fn terminal_create(&mut self) -> Action {
        let size = match self.terminal_geometry() {
            Some(size) => size,
            None => {
                return self.error("Terminal panel requires three rows and a usable source area");
            }
        };
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let mut launch = self.terminals.launch.clone();
        launch.directory = self
            .tree
            .chosen_root()
            .or_else(|| self.buffer().ok()?.file()?.path().parent())
            .unwrap_or(&launch.directory)
            .to_owned();
        let id = match self.terminals.sessions.open(launch, size) {
            Ok(id) => id,
            Err(error) => return self.error(&format!("Open terminal: {error}")),
        };
        if let Err(error) = self.buffers.terminal(id) {
            let cleanup = self.terminals.sessions.close(id);
            return self.error(&format!(
                "Create terminal buffer: {error}; cleanup: {cleanup:?}"
            ));
        }
        self.terminals.default = Some(id);
        self.terminal_show(id)
    }

    pub(super) fn terminal_show(&mut self, id: SessionId) -> Action {
        let Some(size) = self.terminal_geometry() else {
            return self.error("Terminal panel requires three rows and a usable source area");
        };
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if let Err(error) = self.terminals.sessions.resize(id, size) {
            return self.error(&format!("Show terminal: {error}"));
        }
        if !self.terminals.focused {
            self.terminals.home = Some((self.windows.selected_id(), self.tree.focused()));
        }
        self.terminals.selected = Some(id);
        self.dismiss_completion();
        self.terminals.visible = true;
        self.terminals.focused = true;
        self.tree.edit();
        self.input.cancel();
        self.pending = None;
        self.sync_input_context();
        self.message("Terminal; C-g returns to the editor, M-x opens host commands")
    }

    fn terminal_hide(&mut self) -> Action {
        self.terminals.visible = false;
        self.terminal_return();
        self.message("Terminal panel hidden")
    }

    pub(super) fn terminal_return(&mut self) {
        self.terminals.focused = false;
        if let Some((id, tree)) = self.terminals.home.take() {
            let _ = self.windows.select(id);
            if tree {
                let _ = self.tree.focus(self.size);
            }
        }
        self.sync_input_context();
    }

    pub(super) fn terminal_source_focus(&mut self) {
        self.terminals.focused = false;
        self.terminals.home = None;
    }

    fn terminal_geometry(&self) -> Option<Geometry> {
        let minimum = self.windows.layout().panel_minimum()?;
        let rows = self
            .terminals
            .height
            .min(self.size.height.checked_sub(minimum.height + 3)?);
        if rows < 3 || self.size.width < 2 {
            return None;
        }
        let size = Size {
            height: self.size.height - rows - 2,
            ..self.size
        };
        let composition = Composition::workspace(&self.windows, &self.tree, size, 0)?;
        if composition.panels().is_empty() {
            return None;
        }
        Some(Geometry {
            columns: self.size.width,
            rows,
        })
    }

    pub(super) fn terminal_rows(&self) -> Option<u16> {
        self.terminals
            .visible
            .then(|| self.terminal_geometry())
            .flatten()
            .map(|s| s.rows)
    }

    pub(super) fn resize_terminal(&mut self) -> Action {
        let Some(id) = self.terminals.selected.filter(|_| self.terminals.visible) else {
            return Action::Wait;
        };
        let Some(size) = self.terminal_geometry() else {
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
            Err(error) => self.error(&format!("Terminal retains its admitted geometry: {error}")),
        }
    }

    pub(super) fn terminal_remaining(&self) -> Option<Duration> {
        self.terminals.sessions.is_ready().then_some(Duration::ZERO)
    }

    pub(super) fn expire_terminals(&mut self) -> Action {
        if !self.terminals.sessions.is_ready() {
            return Action::Wait;
        }
        let work = self.terminals.sessions.service();
        if work.changed.is_empty() {
            Action::Wait
        } else {
            Action::Redraw
        }
    }

    pub(super) fn terminal_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&Composition>,
    ) {
        let Some(panel) = composition.and_then(Composition::terminal) else {
            return;
        };
        let Some(id) = self.terminals.selected else {
            return;
        };
        let Some(session) = self.terminals.sessions.get(id) else {
            return;
        };
        let name = self.terminal_buffer().map_or("Terminal", |b| b.name());
        let label = format!(" {name}  {:?}  C-g: editor", session.state());
        frame.terminal(session, panel, &label);
    }
}
