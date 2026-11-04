mod control;
mod navigation;
mod operator;
mod pointer;
mod query;
mod search;
mod snapshot;
mod state;

use super::{Action, Session};
use crate::{
    command::{Command, LinearCommand, ModalCommand},
    input::{Focus, SearchAction},
    search::Direction,
    term::{SessionId, Viewport},
    view::{Size, View},
    window::ViewId,
};
pub(super) use state::State;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Key {
    Panel,
    View(ViewId),
}

impl Session {
    fn copy_key(&self) -> Option<Key> {
        self.terminals
            .focused
            .then(|| self.terminals.view.map_or(Key::Panel, Key::View))
    }
    pub(in crate::session) fn terminal_copy_active(&self) -> bool {
        self.copy_key()
            .and_then(|key| self.terminals.copies.get(&key))
            .is_some_and(|copy| copy.mode)
    }
    /// The selected frozen view is independent of the live terminal resource.
    pub fn terminal_copy_view(&self) -> Option<&View> {
        self.terminals
            .copies
            .get(&self.copy_key()?)
            .map(|copy| &copy.view)
    }
    pub(in crate::session) fn sync_terminal_copy_input(&mut self) -> bool {
        if !self.terminal_copy_active() {
            return false;
        }
        let key = self.copy_key().expect("copy context");
        let copy = self.terminals.copies.get(&key).expect("copy context");
        self.input
            .editing_owner(self.windows.selected_id(), copy.buffer.id());
        self.input
            .observe_navigation(self.windows.selected_id(), copy.view.point);
        if let Some(shape) = copy.buffer.visual() {
            let state = crate::input::EditingState::from_visual(shape);
            if self.input.editing_state() != Some(state) {
                self.input.set_editing_state(state);
            }
        }
        true
    }
    pub(super) fn terminal_copy_mode(&mut self) -> Action {
        if self.terminal_copy_active() {
            return self.message("Terminal copy mode is already active");
        }
        match self.admit_terminal_copy() {
            Ok(()) => {
                let key = self.copy_key().expect("admitted copy");
                self.terminals
                    .copies
                    .get_mut(&key)
                    .expect("admitted copy")
                    .mode = true;
                self.input.cancel();
                self.input
                    .set_editing_state(crate::input::EditingState::Normal);
                self.sync_input_context();
                self.message("Terminal copy mode; Return resumes live input")
            }
            Err(error) => self.error(&error),
        }
    }
    fn admit_terminal_copy(&mut self) -> Result<(), String> {
        let key = self
            .copy_key()
            .ok_or("Terminal copy mode requires terminal focus")?;
        let id = self.current_terminal().ok_or("No terminal is selected")?;
        if self.terminals.copies.contains_key(&key) {
            return Ok(());
        }
        self.admit_copy_at(key, id, self.copy_viewport(key))
    }
    fn admit_copy_at(&mut self, key: Key, id: SessionId, viewport: Viewport) -> Result<(), String> {
        let session = self
            .terminals
            .sessions
            .get(id)
            .ok_or("Terminal is unavailable")?;
        let candidate = State::capture(id, session, viewport, self.buffers.snapshot())?;
        let retained: usize = self.terminals.copies.values().map(|copy| copy.charge).sum();
        if self.terminals.copies.len() >= snapshot::STATES
            || retained + candidate.charge > snapshot::EDITOR_BYTES
        {
            return Err(
                "Terminal copy storage is full; existing views and live state retained".into(),
            );
        }
        self.terminals.copies.insert(key, candidate);
        Ok(())
    }
    pub(super) fn resume_copy_tail(&mut self, key: Key) {
        if self.copy_viewport(key).offset == 0
            && self
                .terminals
                .copies
                .get(&key)
                .is_some_and(|copy| !copy.mode)
        {
            self.terminals.copies.remove(&key);
            self.copy_viewport_mut(key).selection = None;
        }
    }
    pub(super) fn discard_passive_copy(&mut self, key: Key) {
        if self
            .terminals
            .copies
            .get(&key)
            .is_some_and(|copy| !copy.mode)
        {
            self.terminals.copies.remove(&key);
            self.copy_viewport_mut(key).offset = 0;
            self.copy_viewport_mut(key).selection = None;
        }
    }
    pub(super) fn resume_terminal_input_tail(&mut self) {
        if let Some(key) = self.copy_key() {
            self.discard_passive_copy(key);
        }
    }
    pub(super) fn terminal_copy_done(&mut self) -> Action {
        let Some(key) = self.copy_key() else {
            return self.error("No terminal copy view is selected");
        };
        if self.terminals.copies.remove(&key).is_none() {
            return self.error("Terminal copy mode is not active");
        }
        self.copy_viewport_mut(key).offset = 0;
        self.copy_viewport_mut(key).selection = None;
        self.input.cancel();
        self.input
            .set_editing_state(crate::input::EditingState::Normal);
        self.message("Terminal live input; following the tail")
    }
    pub(super) fn terminal_copy_quit(&mut self) -> Option<Action> {
        let key = self.copy_key()?;
        let copy = self.terminals.copies.get_mut(&key)?;
        if !copy.mode {
            return None;
        }
        if let Some(search) = copy.search.take() {
            copy.view = search.origin;
            self.input.cancel();
            return Some(self.message("Terminal copy search cancelled"));
        }
        if self.terminals.host_cancel {
            return None;
        }
        if copy.clear_selection() {
            self.input
                .set_editing_state(crate::input::EditingState::Normal);
            return Some(self.message("Terminal copy selection cleared"));
        }
        Some(self.terminal_copy_done())
    }
    pub(in crate::session) fn terminal_copy_command(&mut self, command: Command) -> Option<Action> {
        if !self.terminal_copy_active() || self.input.focus() != Focus::Editing {
            return None;
        }
        let key = self.copy_key().expect("active copy view");
        if let Some(action) = self.copy_text_command(key, command) {
            return Some(action);
        }
        let size = match self.copy_size(key) {
            Some(size) => size,
            None => return Some(self.error("Terminal copy presentation is suspended")),
        };
        match command {
            Command::Navigate(request) => self.copy_navigation(key, request, size),
            Command::Motion(_) | Command::Scroll(_) | Command::Region(_) => {
                Some(self.move_terminal_copy(key, command, size))
            }
            _ => None,
        }
    }
    fn copy_text_command(&mut self, key: Key, command: Command) -> Option<Action> {
        match command {
            Command::Edit(request) => Some(self.edit_terminal_copy(key, request)),
            Command::Linear(LinearCommand::Copy) => Some(self.copy_terminal_region(key)),
            Command::SearchForward => Some(self.start_terminal_copy_search(Direction::Forward)),
            Command::SearchBackward => Some(self.start_terminal_copy_search(Direction::Backward)),
            Command::Modal(mode) => Some(self.terminal_copy_modal(key, mode)),
            Command::Rectangle(command) => Some(self.terminal_copy_rectangle(key, command)),
            _ => None,
        }
    }
    fn move_terminal_copy(&mut self, key: Key, command: Command, size: Size) -> Action {
        let copy = self.terminals.copies.get_mut(&key).expect("selected copy");
        let result = match command {
            Command::Motion(motion) => copy.motion(motion),
            Command::Scroll(scroll) => copy.scroll(
                scroll,
                if scroll.page() {
                    usize::from(size.text_rows())
                        .saturating_sub(copy.page_overlap)
                        .max(1)
                } else {
                    1
                },
                size,
            ),
            Command::Region(region) => copy.region(region),
            _ => unreachable!("copy navigation commands"),
        }
        .and_then(|_| copy.reveal(size));
        self.copy_result(result)
    }
    fn copy_navigation(
        &mut self,
        key: Key,
        request: crate::command::Navigation,
        size: Size,
    ) -> Option<Action> {
        use crate::command::navigation::Target;
        if matches!(request.target, Target::Find(_)) && request.character.is_none() {
            let outcome = self.input.expect_navigation(request);
            return Some(self.outcome(outcome, std::time::Instant::now()));
        }
        if let Target::Search(search) = request.target {
            return Some(self.navigate_copy_search(key, search, request.amount()));
        }
        let copy = self.terminals.copies.get_mut(&key).expect("selected copy");
        let result = copy.navigate(request, size).and_then(|_| copy.reveal(size));
        Some(self.copy_result(result))
    }
    fn terminal_copy_modal(&mut self, key: Key, mode: ModalCommand) -> Action {
        let copy = self.terminals.copies.get_mut(&key).expect("selected copy");
        if mode == ModalCommand::Normal {
            copy.clear_selection();
            self.input
                .set_editing_state(crate::input::EditingState::Normal);
            return self.message("");
        }
        let Some(shape) = mode.visual() else {
            return self.error("Terminal copy text is read-only");
        };
        let result = copy
            .buffer
            .activate_visual(copy.view.point, shape)
            .map_err(|e| e.to_string());
        if result.is_ok() {
            let editing = match shape {
                crate::buffer::Visual::Character => crate::input::EditingState::VisualCharacter,
                crate::buffer::Visual::Line => crate::input::EditingState::VisualLine,
                crate::buffer::Visual::Block => crate::input::EditingState::VisualBlock,
            };
            self.input.set_editing_state(editing);
        }
        self.copy_result(result)
    }
    fn copy_terminal_region(&mut self, key: Key) -> Action {
        match self.prepare_copy_transfer(key) {
            Ok(entry) => {
                self.kills.publish_transfer(entry);
                self.message("Terminal region copied; copy viewport retained")
            }
            Err(error) => self.error(&error),
        }
    }
    fn prepare_copy_transfer(&self, key: Key) -> Result<crate::kill::Entry, String> {
        let copy = self.terminals.copies.get(&key).expect("selected copy");
        if let Some(selection) = crate::rectangle::Selection::new(&copy.buffer, copy.view.point)
            .map_err(|e| e.to_string())?
        {
            let rectangle = crate::rectangle::Rectangle::capture(
                &copy.buffer,
                &selection,
                crate::rectangle::Capture::Copy,
            )
            .map_err(|e| e.to_string())?;
            return Ok(self.kills.prepare_block(std::sync::Arc::new(rectangle)));
        }
        let shape = if copy.buffer.visual() == Some(crate::buffer::Visual::Line) {
            crate::kill::Shape::Line
        } else {
            crate::kill::Shape::Linear
        };
        self.kills
            .prepare_transfer(
                &copy.buffer,
                &copy.selected()?,
                crate::kill::Transfer {
                    view: self.windows.selected_id(),
                    backward: false,
                    shape,
                },
            )
            .map_err(|e| e.to_string())
    }
    fn copy_result(&mut self, result: Result<(), String>) -> Action {
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn copy_size(&self, key: Key) -> Option<Size> {
        let composition = self.composition()?;
        let panel = match key {
            Key::Panel => composition.terminal()?,
            Key::View(view) => composition.panels().iter().find(|(id, _)| *id == view)?.1,
        };
        Some(Size {
            width: panel.content().width(),
            height: panel.content().height().saturating_add(2),
        })
    }
    fn copy_viewport(&self, key: Key) -> Viewport {
        match key {
            Key::Panel => self.terminals.viewport,
            Key::View(view) => self.windows.get(view).expect("copy attachment").terminal,
        }
    }
    fn copy_viewport_mut(&mut self, key: Key) -> &mut Viewport {
        match key {
            Key::Panel => &mut self.terminals.viewport,
            Key::View(view) => {
                &mut self
                    .windows
                    .get_mut(view)
                    .expect("copy attachment")
                    .terminal
            }
        }
    }
    pub(in crate::session) fn sync_terminal_copies(&mut self) {
        self.cancel_obsolete_copy_searches();
        self.terminals.copies.retain(|key, copy| {
            if self.terminals.sessions.get(copy.id).is_none() {
                return false;
            }
            match key {
                Key::Panel => self.terminals.selected == Some(copy.id),
                Key::View(view) => self
                    .windows
                    .get(*view)
                    .ok()
                    .and_then(|v| self.buffers.get(v.point.stamp.buffer).ok())
                    .is_some_and(|b| b.terminal() == Some(copy.id)),
            }
        });
    }
    fn cancel_obsolete_copy_searches(&mut self) {
        let owner = self.copy_key();
        let searching = self.input.focus() == Focus::Search;
        for (key, copy) in &mut self.terminals.copies {
            if (Some(*key) != owner || !searching)
                && let Some(search) = copy.search.take()
            {
                copy.view = search.origin;
            }
        }
    }
    pub(super) fn start_terminal_copy_search(&mut self, direction: Direction) -> Action {
        let key = self.copy_key().expect("selected terminal copy");
        if let Err(error) = self.input.search(direction) {
            return self.error(&error.to_string());
        }
        let copy = self.terminals.copies.get_mut(&key).expect("selected copy");
        copy.search = Some(search::Search::new(&copy.view, direction));
        self.message("Search frozen terminal output")
    }
    pub(in crate::session) fn terminal_copy_search_step(&mut self) -> Option<Action> {
        if self.input.focus() != Focus::Search {
            return None;
        }
        let key = self.copy_key()?;
        let query = self.input.prompt()?.text().to_owned();
        let size = self.copy_size(key)?;
        let copy = self.terminals.copies.get_mut(&key)?;
        let search = copy.search.as_mut()?;
        let result = search.update(&copy.buffer, &query).and_then(|point| {
            if let Some(point) = point {
                copy.view.point.position = point;
                copy.reveal(size)?;
                return Ok(());
            }
            Err("No match in terminal copy output".into())
        });
        Some(self.copy_result(result))
    }
    pub(in crate::session) fn terminal_copy_search_action(
        &mut self,
        action: SearchAction,
    ) -> Option<Action> {
        let key = self.copy_key()?;
        let copy = self.terminals.copies.get_mut(&key)?;
        let search = copy.search.as_mut()?;
        match action {
            SearchAction::Accept => Some(self.finish_copy_search(key)),
            SearchAction::Repeat(direction) => {
                search.repeat(direction);
                self.input.search_label(direction);
                self.terminal_copy_search_step()
            }
        }
    }
}
