mod control;
mod copy;
mod entry;
mod input;
mod keyboard;
mod lifecycle;
mod link;
mod mouse;
mod placement;
mod pointer;
mod render;
mod selection;
mod shutdown;
mod source;
mod transfer;

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
    closing: std::collections::BTreeMap<SessionId, crate::buffer::Kill>,
    buffers: std::collections::BTreeSet<SessionId>,
    back: std::collections::BTreeMap<ViewId, crate::view::View>,
    pub(super) view: Option<ViewId>,
    contexts: std::collections::BTreeMap<SessionId, PathBuf>,
    viewport: crate::term::Viewport,
    literal: Option<SessionId>,
    capture: Option<pointer::Capture>,
    link: Option<link::Activation>,
    copies: std::collections::BTreeMap<copy::Key, copy::State>,
    copy_chord: Option<SessionId>,
    yank: Option<transfer::Yank>,
    shutdown: shutdown::Shutdown,
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
            closing: Default::default(),
            buffers: Default::default(),
            back: Default::default(),
            view: None,
            contexts: Default::default(),
            viewport: Default::default(),
            literal: None,
            capture: None,
            link: None,
            copies: Default::default(),
            copy_chord: None,
            yank: None,
            shutdown: Default::default(),
        }
    }
}

impl Session {
    fn terminal_closed(&mut self, id: SessionId) {
        if self.current_terminal() == Some(id) && self.terminals.focused {
            self.terminal_return();
        }
        self.terminals.copies.retain(|_, copy| copy.id != id);
        self.terminals.buffers.remove(&id);
        self.terminals.contexts.remove(&id);
        if self.terminals.default == Some(id) {
            self.terminals.default = None;
        }
        if self.terminals.selected == Some(id) {
            self.terminals.selected = None;
            self.terminals.visible = false;
            self.terminal_return();
        }
    }
    pub fn terminals(&self) -> &Sessions {
        &self.terminals.sessions
    }
    pub fn terminal_focused(&self) -> bool {
        self.terminals.focused
    }

    pub fn terminal_buffer(&self) -> Option<&crate::buffer::Buffer> {
        let id = self.current_terminal()?;
        self.buffers.iter().find(|b| b.terminal() == Some(id))
    }

    pub(super) fn terminal_command(&mut self, command: TerminalCommand) -> Action {
        if self.input.focus() != crate::input::Focus::Editing || self.pending.is_some() {
            return self.error("A host prompt owns terminal invocation");
        }
        self.terminal_dispatch(command)
    }

    fn terminal_dispatch(&mut self, command: TerminalCommand) -> Action {
        match command {
            TerminalCommand::CopyMode => return self.terminal_copy_mode(),
            TerminalCommand::CopyModeDone => return self.terminal_copy_done(),
            _ => {}
        }
        if let Some(action) = self.terminal_evidence_command(command) {
            return action;
        }
        if command == TerminalCommand::SendNextKey {
            return self.terminal_literal_next();
        }
        if let Some(action) = self.terminal_transfer_command(command) {
            return action;
        }
        self.terminal_placement_command(command)
    }

    fn terminal_placement_command(&mut self, command: TerminalCommand) -> Action {
        match command {
            TerminalCommand::Open => self.terminal_open(),
            TerminalCommand::Toggle => self.terminal_toggle(),
            TerminalCommand::NewBuffer => self.terminal_create(true),
            TerminalCommand::New => self.terminal_create(false),
            TerminalCommand::List => self.list_terminals(),
            TerminalCommand::Rename => self.rename_terminal(),
            TerminalCommand::MoveToBuffer => self.terminal_move(true),
            TerminalCommand::MoveToPanel => self.terminal_move(false),
            _ => self.terminal_entry_command(command),
        }
    }

    fn terminal_entry_command(&mut self, command: TerminalCommand) -> Action {
        match command {
            TerminalCommand::OpenHere => {
                self.terminal_create_at(false, self.terminal_directory(true))
            }
            TerminalCommand::OpenOtherPane => self.terminal_other_pane(),
            _ => unreachable!("directory entry commands only"),
        }
    }

    fn terminal_open(&mut self) -> Action {
        if let Some(id) = self.terminals.default
            && self
                .terminals
                .sessions
                .get(id)
                .is_some_and(|s| *s.state() == State::Running)
        {
            return self.select_terminal(id);
        }
        self.terminal_create(false)
    }

    fn terminal_toggle(&mut self) -> Action {
        if self.terminals.visible {
            return self.terminal_hide();
        }
        if let Some(id) = self.terminals.selected {
            return self.terminal_show(id);
        }
        self.terminal_open()
    }

    fn terminal_create(&mut self, buffer: bool) -> Action {
        self.terminal_create_at(buffer, self.terminal_directory(false))
    }

    fn terminal_create_at(&mut self, buffer: bool, directory: PathBuf) -> Action {
        let geometry = if buffer {
            self.buffer_geometry(&self.windows, self.terminals.visible)
        } else {
            self.terminal_geometry()
        };
        let size = match geometry {
            Some(size) => size,
            None => {
                return self.error("Terminal panel requires three rows and a usable source area");
            }
        };
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let mut launch = self.terminals.launch.clone();
        launch.directory = directory.clone();
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
        self.terminals.contexts.insert(id, directory);
        if !self
            .terminals
            .default
            .and_then(|id| self.terminals.sessions.get(id))
            .is_some_and(|s| *s.state() == State::Running)
        {
            self.terminals.default = Some(id);
        }
        if buffer {
            self.terminal_to_buffer(id)
        } else {
            self.terminal_show(id)
        }
    }

    pub(super) fn terminal_show(&mut self, id: SessionId) -> Action {
        let Some(size) = self.terminal_geometry() else {
            return self.error("Terminal panel requires three rows and a usable source area");
        };
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if let Err(error) = self.admit_terminal_geometry(id, size) {
            return self.error(&format!("Show terminal: {error}"));
        }
        self.show_admitted_terminal(id)
    }

    fn show_admitted_terminal(&mut self, id: SessionId) -> Action {
        if !self.terminals.focused || self.terminals.view.is_some() {
            self.terminals.home = Some((self.windows.selected_id(), self.tree.focused()));
        }
        if self.terminals.selected != Some(id) {
            self.terminals.viewport = Default::default();
        }
        self.terminals.selected = Some(id);
        self.terminals.view = None;
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
        if self.terminals.view.is_none() {
            self.terminal_return();
        }
        self.message("Terminal panel hidden")
    }

    pub(super) fn terminal_return(&mut self) {
        self.terminals.copy_chord = None;
        self.cancel_terminal_pointer();
        self.terminals.literal = None;
        self.restore_terminal_view();
        self.terminals.focused = false;
        if let Some((id, tree)) = self.terminals.home.take() {
            if self
                .windows
                .get(id)
                .ok()
                .and_then(|v| self.buffers.get(v.point.stamp.buffer).ok())
                .is_some_and(|b| b.terminal().is_some())
            {
                self.restore_source_at(id);
            }
            let _ = self.windows.select(id);
            if tree {
                let _ = self.tree.focus(self.size);
            }
        }
        self.sync_input_context();
    }

    pub(super) fn terminal_source_focus(&mut self) {
        self.terminals.copy_chord = None;
        self.cancel_terminal_pointer();
        self.terminals.literal = None;
        self.terminals.focused = false;
        self.terminals.view = None;
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
        if let Some(view) = self.terminals.view {
            return self.resize_terminal_view(view);
        }
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
        match self.admit_terminal_geometry(id, size) {
            Ok(()) => Action::Wait,
            Err(error) => self.error(&format!("Terminal retains its admitted geometry: {error}")),
        }
    }

    pub(super) fn terminal_remaining(&self) -> Option<Duration> {
        (self.terminals.sessions.is_ready()
            || self.terminal_link_ready()
            || self.terminal_shutdown_ready())
        .then_some(Duration::ZERO)
    }

    pub(super) fn expire_terminals(&mut self) -> Action {
        let link = self.expire_terminal_link();
        if !self.terminals.sessions.is_ready() {
            return link;
        }
        let work = self.terminals.sessions.service();
        let closure = self.finish_terminal_closures();
        if work.changed.is_empty() {
            if link != Action::Wait { link } else { closure }
        } else {
            Action::Redraw
        }
    }
}
