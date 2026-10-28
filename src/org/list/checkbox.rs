mod selection;
mod structural;

use super::{ListEdit, Lists, progress::State, replacement};
use crate::buffer::{Buffer, edit::Replacement};
use crate::command::OrgCommand;
use crate::coordinates::{Coordinates, Scalar};

impl Lists {
    pub(super) fn checkbox_edit(
        &self,
        buffer: &Buffer,
        command: OrgCommand,
        point: Scalar,
    ) -> Result<ListEdit, String> {
        if !self.complete {
            return Err("Org list observation is incomplete".into());
        }
        let byte = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(point)
            .map_err(|e| e.to_string())?
            .0;
        if command == OrgCommand::InsertCheckboxItem {
            return self.insert_checkbox(buffer, point, byte);
        }
        let selected = self.checkbox_selection(buffer, command, point)?;
        let mut state = State::new(self, byte);
        let mut edits = self.change_checkboxes(buffer, &mut state, &selected, command)?;
        edits.extend(self.statistics(buffer, &mut state)?);
        self.admit_ordered(&state)?;
        edits.sort_by_key(|edit| edit.source.range.start);
        Ok(ListEdit {
            edits,
            order: Vec::new(),
            cursor: None,
        })
    }

    fn change_checkboxes(
        &self,
        buffer: &Buffer,
        state: &mut State,
        selected: &[usize],
        command: OrgCommand,
    ) -> Result<Vec<Replacement>, String> {
        if command == OrgCommand::UpdateStatisticsCookies {
            return Ok(Vec::new());
        }
        let first = *selected
            .first()
            .ok_or("No Org checkbox in this source context")?;
        let value = if state.values[first] == Some(b'X') {
            b' '
        } else {
            b'X'
        };
        let remove = state.values[first].is_some();
        for index in selected {
            state
                .affected
                .insert(self.heading_at(self.items[*index].source.start));
            if command == OrgCommand::ToggleCheckboxPresence {
                state.values[*index] = if remove { None } else { Some(b' ') };
            } else {
                self.toggle(state, *index, value, command)?;
            }
        }
        if command == OrgCommand::ToggleCheckboxPresence {
            return self.presence(buffer, selected);
        }
        if command == OrgCommand::CtrlCCtrlC {
            return self.numbering(buffer, &self.lists[self.items[first].list].items, None);
        }
        Ok(Vec::new())
    }

    fn toggle(
        &self,
        state: &mut State,
        index: usize,
        value: u8,
        command: OrgCommand,
    ) -> Result<(), String> {
        let list = &self.lists[self.items[index].list];
        if state.values[index].is_none() {
            return if command == OrgCommand::CtrlCCtrlC {
                Ok(())
            } else {
                Err("Org item has no checkbox; use org-toggle-checkbox-presence".into())
            };
        }
        if value == b'X' && (list.radio || command == OrgCommand::ToggleRadioButton) {
            for sibling in &list.items {
                if state.values[*sibling].is_some() {
                    state.values[*sibling] = Some(b' ');
                }
            }
        }
        state.values[index] = Some(value);
        Ok(())
    }

    fn presence(&self, buffer: &Buffer, selected: &[usize]) -> Result<Vec<Replacement>, String> {
        let remove = self.items[selected[0]].checkbox.is_some();
        let mut edits = Vec::new();
        for index in selected {
            let item = &self.items[*index];
            if item.description && !remove {
                return Err("Description lists do not admit factory checkboxes".into());
            }
            if let Some((range, _)) = &item.checkbox {
                if remove {
                    edits.push(replacement(
                        buffer,
                        range.start..item.content,
                        String::new(),
                    )?);
                }
            } else if !remove {
                edits.push(replacement(
                    buffer,
                    item.content..item.content,
                    "[ ] ".into(),
                )?);
            }
        }
        Ok(edits)
    }
}
