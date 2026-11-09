use super::{Buffer, Change, Error, Operation, Point, Prepared, State};
use crate::coordinates::{Coordinates, Scalar};
use ropey::Rope;

impl Buffer {
    /// Disk refresh alone may change a read-only document. Ordinary prepare,
    /// history and publication retain their normal writable admission.
    pub(crate) fn prepare_reload_text(
        &self,
        text: &Rope,
        bom: bool,
        points: &[Point],
    ) -> Result<Prepared, Error> {
        let next_state = self.next_state.checked_add(1).ok_or(Error::Capacity)?;
        let state = State {
            buffer: self.id,
            sequence: self.next_state,
        };
        let changes = self.reload_changes(text)?;
        let mut prepared = self.prepare_changes(changes, points, state, None)?;
        prepared.operation = Operation::Reload;
        prepared.record.after_bom = bom;
        prepared.next_state = next_state;
        Ok(prepared)
    }

    fn reload_changes(&self, text: &Rope) -> Result<Vec<Change>, Error> {
        if self.text == *text {
            return Ok(Vec::new());
        }
        // Keep unchanged prefix/suffix anchors through the native edit mapper.
        // Both seams must be grapheme boundaries in both document images.
        let before = Coordinates::new(self)?;
        let after = Coordinates::for_text(text.slice(..), self.stamp(), self.policy.tab_width);
        let start = self.reload_prefix(text, &before, &after)?;
        let suffix = self
            .text
            .chars_at(self.len_chars())
            .reversed()
            .zip(text.chars_at(text.len_chars()).reversed())
            .take(self.len_chars().min(text.len_chars()) - start)
            .take_while(|(a, b)| a == b)
            .count();
        let mut old_end = self.len_chars() - suffix;
        let mut new_end = text.len_chars() - suffix;
        while !before.is_boundary(Scalar(old_end))? || !after.is_boundary(Scalar(new_end))? {
            old_end += 1;
            new_end += 1;
        }
        Ok(vec![Change {
            before: start..old_end,
            after: start..new_end,
            removed: self.text.slice(start..old_end).into(),
            inserted: text.slice(start..new_end).into(),
        }])
    }
    fn reload_prefix(
        &self,
        text: &Rope,
        before: &Coordinates<'_>,
        after: &Coordinates<'_>,
    ) -> Result<usize, Error> {
        let mut start = self
            .text
            .chars()
            .zip(text.chars())
            .take_while(|(a, b)| a == b)
            .count();
        while !before.is_boundary(Scalar(start))? || !after.is_boundary(Scalar(start))? {
            start -= 1;
        }
        Ok(start)
    }
}
