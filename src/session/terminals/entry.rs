use super::{Action, Session, State};
use crate::window::Split;
use std::path::PathBuf;

impl Session {
    pub(super) fn terminal_directory(&self, here: bool) -> PathBuf {
        if !here && let Some(directory) = self.tree.chosen_root() {
            return directory.to_owned();
        }
        if here
            && self.terminals.focused
            && let Some(directory) = self
                .current_terminal()
                .and_then(|id| self.terminals.contexts.get(&id))
        {
            return directory.clone();
        }
        self.buffer()
            .ok()
            .and_then(|b| b.file())
            .and_then(|f| f.path().parent())
            .unwrap_or(&self.terminals.launch.directory)
            .to_owned()
    }

    pub(super) fn terminal_other_pane(&mut self) -> Action {
        let id = self
            .current_terminal()
            .or(self.terminals.default)
            .filter(|id| {
                self.terminals
                    .sessions
                    .get(*id)
                    .is_some_and(|s| *s.state() == State::Running)
            });
        let visible = self.terminals.visible && self.terminals.selected != id;
        let mut candidate = self.windows.clone();
        let target = if candidate.iter().len() == 1 {
            let Some(composition) = self.terminal_composition(&candidate, visible) else {
                return self.error("Other terminal pane does not fit");
            };
            candidate.split(Split::Right, composition.editing_size())
        } else {
            candidate.next_view()
        };
        let target = match target {
            Ok(target) => target,
            Err(error) => return self.error(&error.to_string()),
        };
        candidate.select(target).expect("prepared pane");
        let Some(size) = self.buffer_geometry(&candidate, visible) else {
            return self.error("Other terminal pane does not fit");
        };
        if let Some(id) = id
            && let Err(error) = self.admit_terminal_geometry(id, size)
        {
            return self.error(&format!("Other terminal pane: {error}"));
        }
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let original = std::mem::replace(&mut self.windows, candidate);
        let action = match id {
            Some(id) => self.terminal_to_buffer(id),
            None => self.terminal_create(true),
        };
        if !self.terminals.focused || self.terminals.view != Some(target) {
            self.windows = original;
        }
        action
    }
}
