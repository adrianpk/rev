use super::super::{
    ListEdit, Lists, Target,
    progress::{Count, State},
};
use crate::buffer::Buffer;
use crate::command::OrgCommand;
use crate::coordinates::{Coordinates, Scalar};

impl Lists {
    pub(in crate::org::list) fn structural_progress(
        &self,
        buffer: &Buffer,
        target: Target,
        mut plan: ListEdit,
    ) -> Result<ListEdit, String> {
        let Target {
            index,
            command,
            point,
        } = target;
        if !plan.order.is_empty() {
            return Ok(plan);
        }
        let item = &self.items[index];
        let byte = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(point)
            .map_err(|e| e.to_string())?
            .0;
        let mut state = State::new(self, item.source.start);
        self.layout(&mut state, index, command, byte);
        let statistics = self.statistics(buffer, &mut state)?;
        self.admit_ordered(&state)?;
        if let Some(value) = state.values.get(self.items.len()).copied().flatten() {
            for edit in &mut plan.edits {
                edit.text = edit
                    .text
                    .replacen("[ ]", &format!("[{}]", char::from(value)), 1);
            }
        }
        shift_cursor(&mut plan, &statistics, point)?;
        plan.edits.extend(statistics);
        plan.edits.sort_by_key(|edit| edit.source.range.start);
        Ok(plan)
    }

    fn layout(&self, state: &mut State, index: usize, command: OrgCommand, byte: usize) {
        if command.list_insertion() {
            self.insert_layout(
                state,
                index,
                byte,
                command == OrgCommand::InsertCheckboxItem,
            );
        } else if command.list_indentation() {
            self.indent_layout(state, index, command);
        }
    }

    fn insert_layout(&self, state: &mut State, index: usize, byte: usize, checkbox: bool) {
        let item = &self.items[index];
        let new = state.values.len();
        state
            .values
            .push((checkbox || item.checkbox.is_some()).then_some(b' '));
        state.parents.push(state.parents[index]);
        state.positions.push(if byte <= item.content {
            item.source.start
        } else {
            byte
        });
        state.children.push(Count::default());
        if byte > item.content {
            for parent in &mut state.parents[..new] {
                if *parent == Some(index) {
                    *parent = Some(new);
                }
            }
        }
    }

    fn indent_layout(&self, state: &mut State, index: usize, command: OrgCommand) {
        let list = &self.lists[self.items[index].list];
        let position = list
            .items
            .iter()
            .position(|item| *item == index)
            .expect("list owns item");
        let right = matches!(
            command,
            OrgCommand::IndentItem | OrgCommand::IndentItemTree | OrgCommand::MetaRight
        );
        if right {
            if let Some(previous) = position.checked_sub(1) {
                state.parents[index] = Some(list.items[previous]);
            }
            if command != OrgCommand::IndentItemTree {
                state.reparent_children(index, state.parents[index]);
            }
        } else {
            state.parents[index] = list.parent.and_then(|parent| state.parents[parent]);
            self.outdent_following(state, index, position);
        }
    }

    fn outdent_following(&self, state: &mut State, index: usize, position: usize) {
        let list = &self.lists[self.items[index].list];
        if list.parent.is_some() {
            for sibling in &list.items[position + 1..] {
                state.parents[*sibling] = Some(index);
            }
        }
    }

    pub(super) fn insert_checkbox(
        &self,
        buffer: &Buffer,
        point: Scalar,
        byte: usize,
    ) -> Result<ListEdit, String> {
        if buffer.mark_active() {
            return Err("Org checkbox insertion requires an unselected item".into());
        }
        let index = self
            .at(byte, byte == buffer.len_bytes())
            .ok_or("No editable Org list item here")?;
        if self.items[index].description {
            return Err("Description lists do not admit factory checkboxes".into());
        }
        let plan = self.insert_with(buffer, index, point, true)?;
        self.structural_progress(
            buffer,
            Target {
                index,
                command: OrgCommand::InsertCheckboxItem,
                point,
            },
            plan,
        )
    }
}

fn shift_cursor(
    plan: &mut ListEdit,
    edits: &[crate::buffer::edit::Replacement],
    point: Scalar,
) -> Result<(), String> {
    if let Some(cursor) = &mut plan.cursor {
        for edit in edits
            .iter()
            .filter(|edit| edit.source.range.start < point.0)
        {
            cursor.0 = cursor
                .0
                .checked_add_signed(
                    edit.text.chars().count() as isize - edit.source.range.len() as isize,
                )
                .ok_or("Org insertion position is exhausted")?;
        }
    }
    Ok(())
}
