use super::{Candidate, Completion};
use crate::buffer::{Buffer, Span, edit::Replacement};
use crate::coordinates::Scalar;
use crate::lsp::completion::{Item, Proposal};
use std::sync::Arc;

pub(crate) type Edits = (Vec<Replacement>, Scalar);
impl Completion {
    pub(crate) fn proposal(&self) -> Option<Arc<Proposal>> {
        self.candidates.get(self.selected)?.proposal.clone()
    }
    pub(crate) fn resolved(&mut self, item: Item) {
        self.candidates[self.selected] = Candidate {
            label: item.label,
            kind: self.candidates[self.selected].kind,
            detail: self.candidates[self.selected].detail,
            text: item.text,
            range: Some(item.range),
            additional: item.additional,
            proposal: None,
        };
    }
    pub(crate) fn edits(&self, buffer: &Buffer) -> Result<Edits, String> {
        let primary = self.replacement(buffer)?;
        let selected = self
            .candidates
            .get(self.selected)
            .ok_or("No completion selected")?;
        let mut cursor = primary.source.range.start + primary.text.chars().count();
        let mut edits = Vec::new();
        for edit in &selected.additional {
            if edit.range.end <= primary.source.range.start {
                cursor = cursor
                    .checked_sub(edit.range.len())
                    .and_then(|at| at.checked_add(edit.text.chars().count()))
                    .ok_or("Completion cursor overflow")?;
            }
            edits.push(Replacement {
                source: Span {
                    stamp: primary.source.stamp,
                    range: edit.range.clone(),
                },
                text: edit.text.clone(),
            });
        }
        edits.push(primary);
        edits.sort_by_key(|edit| (edit.source.range.start, edit.source.range.end));
        Ok((edits, Scalar(cursor)))
    }
}
