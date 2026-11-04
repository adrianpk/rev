use super::{Key, Session, snapshot::Snapshot};
use crate::{
    command::TerminalCommand,
    coordinates::{Coordinates, Scalar},
    session::Action,
    term,
};

pub(super) struct Evidence {
    prompts: Vec<Scalar>,
    links: super::links::Links,
}
impl Evidence {
    pub fn capture(session: &term::Session, snapshot: &Snapshot) -> Self {
        let mut prompts: Vec<_> = session
            .prompts()
            .map(|point| snapshot.point(session, point))
            .collect();
        prompts.sort_unstable();
        prompts.dedup();
        Self {
            prompts,
            links: super::links::Links::capture(session, snapshot),
        }
    }
    fn prompt(&self, point: Scalar, forward: bool) -> Option<Scalar> {
        if forward {
            self.prompts.iter().copied().find(|p| *p > point)
        } else {
            self.prompts.iter().copied().rev().find(|p| *p < point)
        }
    }
}
impl Session {
    pub(in crate::session::terminals) fn terminal_evidence_command(
        &mut self,
        command: TerminalCommand,
    ) -> Option<Action> {
        Some(match command {
            TerminalCommand::NextPrompt => self.terminal_prompt(true),
            TerminalCommand::PreviousPrompt => self.terminal_prompt(false),
            TerminalCommand::OpenLink => self.terminal_selected_link(),
            TerminalCommand::NextError => self.terminal_next_error(),
            _ => return None,
        })
    }
    fn terminal_prompt(&mut self, forward: bool) -> Action {
        if !self.terminal_copy_active() {
            self.terminal_copy_mode();
        }
        let result = self.move_terminal_prompt(forward);
        self.copy_result(result)
    }
    fn move_terminal_prompt(&mut self, forward: bool) -> Result<(), String> {
        let key = self
            .copy_key()
            .ok_or("Prompt navigation requires terminal focus")?;
        let size = self
            .copy_size(key)
            .ok_or("Terminal copy presentation is suspended")?;
        let copy = self
            .terminals
            .copies
            .get_mut(&key)
            .ok_or("Terminal copy is unavailable")?;
        let point = copy
            .evidence
            .prompt(copy.view.point.position, forward)
            .ok_or("No retained OSC 133 prompt evidence in that direction")?;
        let coordinates = Coordinates::new(&copy.buffer).map_err(|e| e.to_string())?;
        let point = if coordinates.is_boundary(point).map_err(|e| e.to_string())? {
            point
        } else {
            coordinates.previous(point).map_err(|e| e.to_string())?
        };
        copy.view.point.position = point;
        copy.view.reset_navigation();
        copy.reveal(size)
    }
    fn terminal_selected_link(&mut self) -> Action {
        let Some(id) = self.current_terminal().filter(|_| self.terminals.focused) else {
            return self.error("Link activation requires terminal focus");
        };
        if let Some(key) = self
            .copy_key()
            .filter(|key| self.terminals.copies.contains_key(key))
        {
            let point = self.terminals.copies[&key].view.point.position;
            return self.terminal_copy_link(key, point);
        }
        let Some(session) = self.terminals.sessions.get(id) else {
            return self.error("Terminal is unavailable");
        };
        self.terminal_link_at(id, session.screen().grid().cursor.point)
    }
    pub(in crate::session::terminals) fn terminal_copy_link_cell(
        &mut self,
        key: Key,
        event: crossterm::event::MouseEvent,
    ) -> Action {
        let copy = self.terminals.copies.get(&key).expect("observed copy view");
        let cell = crate::coordinates::Cell {
            row: (copy.view.top + usize::from(event.row)).min(copy.buffer.len_lines() - 1),
            column: copy.view.left + usize::from(event.column),
        };
        let point = Coordinates::new(&copy.buffer)
            .map_err(|error| error.to_string())
            .and_then(|coordinates| {
                coordinates
                    .from_cell(cell)
                    .map_err(|error| error.to_string())
            });
        match point {
            Ok(point) => self.terminal_copy_link(key, point),
            Err(error) => self.error(&error.to_string()),
        }
    }
    pub(in crate::session::terminals) fn terminal_copy_link(
        &mut self,
        key: Key,
        point: Scalar,
    ) -> Action {
        let result = self
            .copy_link_target(key, point)
            .and_then(|(id, uri)| self.activate_terminal_link(id, &uri));
        match result {
            Ok(()) => self.message("Terminal link viewer running"),
            Err(error) => self.error(&error),
        }
    }
    fn copy_link_target(
        &self,
        key: Key,
        point: Scalar,
    ) -> Result<(term::SessionId, String), String> {
        let copy = self
            .terminals
            .copies
            .get(&key)
            .ok_or("Terminal copy is unavailable")?;
        let end = Coordinates::new(&copy.buffer)
            .map_err(|e| e.to_string())?
            .next_observed(point)
            .map_err(|e| e.to_string())?;
        let uri = copy
            .evidence
            .links
            .at(point.0..end.0)
            .ok_or("No retained hyperlink evidence at this copy location")?;
        Ok((copy.id, uri.into()))
    }
}
