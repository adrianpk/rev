mod group;
mod movement;
pub(in crate::org) use movement::pieces as movement_pieces;

use std::ops::Range;

use crate::buffer::{Buffer, Span, edit::Replacement};
use crate::command::OrgCommand;
use crate::coordinates::{Byte, Scalar};
use crate::org::Source;

use super::Outline;

pub(crate) enum SubtreeEdit {
    Levels(Vec<Replacement>),
    Reorder(Vec<Span>),
}

struct Group {
    bytes: Range<usize>,
    roots: Vec<usize>,
    headings: Range<usize>,
}

impl Outline {
    pub(crate) fn subtree_edit(
        &self,
        buffer: &Buffer,
        command: OrgCommand,
        point: Scalar,
    ) -> Result<SubtreeEdit, String> {
        self.complete().map_err(str::to_owned)?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        let group = self.subtree_group(buffer, point)?;
        self.admit_group(&group).map_err(str::to_owned)?;
        match command {
            OrgCommand::PromoteSubtree | OrgCommand::DemoteSubtree => {
                self.change_levels(buffer, &group, command)
            }
            _ => self
                .move_group(buffer, &group, command)
                .map(SubtreeEdit::Reorder),
        }
    }

    fn change_levels(
        &self,
        buffer: &Buffer,
        group: &Group,
        command: OrgCommand,
    ) -> Result<SubtreeEdit, String> {
        let demote = command == OrgCommand::DemoteSubtree;
        self.admit_level_change(group, demote)
            .map_err(str::to_owned)?;
        let mut edits = Vec::new();
        for h in &self.headings[group.headings.clone()] {
            let end = h.start + h.level;
            let start = end - usize::from(!demote);
            let source = Source::new(buffer)
                .range(Byte(start)..Byte(end))
                .map_err(|e| e.to_string())?;
            edits.push(Replacement {
                source: source.span().clone(),
                text: if demote { "*".into() } else { String::new() },
            });
        }
        Ok(SubtreeEdit::Levels(edits))
    }

    fn admit_level_change(&self, group: &Group, demote: bool) -> Result<(), &'static str> {
        if !demote && self.headings[group.roots[0]].level == 1 {
            return Err("Cannot promote a top-level Org subtree");
        }
        if demote {
            let parent = self
                .sibling(group.roots[0], false)
                .ok_or("No previous Org sibling to become the parent")?;
            self.admit_heading(parent)?;
            if self.headings[group.headings.clone()]
                .iter()
                .any(|h| h.level == 256)
            {
                return Err("Org subtree exceeds the native depth limit");
            }
        }
        Ok(())
    }

    fn sibling(&self, index: usize, forward: bool) -> Option<usize> {
        let h = &self.headings[index];
        let candidate = if forward {
            (index + 1..self.headings.len()).find(|i| self.headings[*i].level <= h.level)
        } else {
            (0..index)
                .rev()
                .find(|i| self.headings[*i].level <= h.level)
        }?;
        (self.headings[candidate].level == h.level && self.headings[candidate].parent == h.parent)
            .then_some(candidate)
    }
}
