mod model;
mod state;
pub(super) use model::cookies;
pub(super) use state::State;

use super::{Lists, replacement, source_text};
use crate::buffer::{Buffer, edit::Replacement};
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Cookie {
    pub(super) source: Range<usize>,
    percent: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Heading {
    pub(super) source: Range<usize>,
    pub(super) title: Range<usize>,
    pub(super) parent: Option<usize>,
    pub(super) level: usize,
    pub(super) task: Option<bool>,
    pub(super) ordered: bool,
    pub(super) data: Data,
    pub(super) cookies: Vec<Cookie>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) enum Data {
    #[default]
    Auto,
    Checkbox,
    Todo,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Count {
    pub(super) total: usize,
    pub(super) done: usize,
    pub(super) partial: usize,
}

impl Count {
    pub(super) fn add(&mut self, state: Option<u8>) {
        if let Some(state) = state {
            self.total += 1;
            self.done += usize::from(state == b'X');
            self.partial += usize::from(state == b'-');
        }
    }
    pub(super) fn state(self) -> u8 {
        if self.done == self.total {
            b'X'
        } else if self.done + self.partial == 0 {
            b' '
        } else {
            b'-'
        }
    }
}

impl Lists {
    pub(super) fn heading_at(&self, byte: usize) -> Option<usize> {
        self.headings
            .partition_point(|heading| heading.source.start <= byte)
            .checked_sub(1)
            .filter(|index| byte <= self.headings[*index].source.end)
    }

    pub(super) fn statistics(
        &self,
        buffer: &Buffer,
        state: &mut State,
    ) -> Result<Vec<Replacement>, String> {
        state.propagate();
        let mut edits = Vec::new();
        for (index, item) in self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| state.affected.contains(&self.heading_at(item.source.start)))
        {
            state.marker(buffer, item, index, &mut edits)?;
            update(buffer, &item.cookies, state.children[index], &mut edits)?;
        }
        let counts = state.heading_counts(self);
        for (index, heading) in self
            .headings
            .iter()
            .enumerate()
            .filter(|(index, _)| state.affected.contains(&Some(*index)))
        {
            let count = self.heading_count(index, heading, counts[index]);
            update(buffer, &heading.cookies, count, &mut edits)?;
        }
        Ok(edits)
    }

    fn heading_count(&self, index: usize, heading: &Heading, checkbox: Count) -> Count {
        if heading.data == Data::Checkbox || heading.data == Data::Auto && checkbox.total > 0 {
            return checkbox;
        }
        let mut count = Count::default();
        for child in self
            .headings
            .iter()
            .filter(|child| child.parent == Some(index))
        {
            count.add(child.task.map(|done| if done { b'X' } else { b' ' }));
        }
        count
    }
}

fn update(
    buffer: &Buffer,
    cookies: &[Cookie],
    count: Count,
    edits: &mut Vec<Replacement>,
) -> Result<(), String> {
    for cookie in cookies {
        let text = if cookie.percent {
            format!("[{}%]", count.done * 100 / count.total.max(1))
        } else {
            format!("[{}/{}]", count.done, count.total)
        };
        if source_text(buffer, cookie.source.clone())? != text {
            edits.push(replacement(buffer, cookie.source.clone(), text)?);
        }
    }
    Ok(())
}
