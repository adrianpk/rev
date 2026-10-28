use super::{Count, Lists};
use crate::buffer::{Buffer, edit::Replacement};
use crate::org::list::{Item, replacement};

pub(in crate::org::list) struct State {
    pub(in crate::org::list) values: Vec<Option<u8>>,
    pub(in crate::org::list) parents: Vec<Option<usize>>,
    pub(in crate::org::list) positions: Vec<usize>,
    pub(in crate::org::list) affected: std::collections::BTreeSet<Option<usize>>,
    pub(in crate::org::list) children: Vec<Count>,
}

impl State {
    pub(in crate::org::list) fn new(model: &Lists, byte: usize) -> Self {
        Self {
            affected: [model.heading_at(byte)].into_iter().collect(),
            values: model
                .items
                .iter()
                .map(|item| item.checkbox.as_ref().map(|entry| entry.1))
                .collect(),
            parents: model
                .items
                .iter()
                .map(|item| model.lists[item.list].parent)
                .collect(),
            positions: model.items.iter().map(|item| item.source.start).collect(),
            children: vec![Count::default(); model.items.len()],
        }
    }

    pub(in crate::org::list) fn reparent_children(&mut self, index: usize, parent: Option<usize>) {
        for child in self.parents.iter_mut().skip(index + 1) {
            if *child == Some(index) {
                *child = parent;
            }
        }
    }

    pub(super) fn propagate(&mut self) {
        let mut order: Vec<_> = (0..self.values.len()).collect();
        order.sort_by_key(|index| self.positions[*index]);
        self.children.fill(Count::default());
        for index in order.into_iter().rev() {
            if self.values[index].is_some() && self.children[index].total > 0 {
                self.values[index] = Some(self.children[index].state());
            }
            if let Some(parent) = self.parents[index] {
                self.children[parent].add(self.values[index]);
            }
        }
    }

    pub(super) fn marker(
        &self,
        buffer: &Buffer,
        item: &Item,
        index: usize,
        edits: &mut Vec<Replacement>,
    ) -> Result<(), String> {
        if let Some((range, old)) = &item.checkbox {
            if let Some(state) = self.values[index].filter(|state| state != old) {
                edits.push(replacement(
                    buffer,
                    range.start + 1..range.end - 1,
                    char::from(state).to_string(),
                )?);
            }
        }
        Ok(())
    }

    pub(super) fn heading_counts(&self, model: &Lists) -> Vec<Count> {
        let mut counts = vec![Count::default(); model.headings.len()];
        for index in 0..self.values.len() {
            if self.parents[index].is_none() {
                if let Some(heading) = model.heading_at(self.positions[index]) {
                    counts[heading].add(self.values[index]);
                }
            }
        }
        counts
    }
}
