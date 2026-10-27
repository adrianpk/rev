use super::{Error, Point, Prepared, Transaction, mapping};
use crate::coordinates::Coordinates;
use crate::window::ViewId;

impl Prepared {
    /// A viewport follows its source line start with left insertion affinity.
    /// Resolve the final logical row while text is still a private candidate.
    pub(crate) fn map_top(&self, top: Point) -> Result<usize, Error> {
        if top.stamp != self.record.before {
            return Err(Error::ChangedPositions);
        }
        let position =
            mapping::mapped(&self.record.changes, top.position, mapping::Affinity::Left)?;
        let coordinates =
            Coordinates::for_text(self.text.slice(..), self.record.after, self.tab_width);
        Ok(coordinates.to_line(position)?.row)
    }

    pub(crate) fn set_origin(&mut self, origin: ViewId) {
        self.record.origin = Some(origin);
        // A different editing view cannot inherit an adjacent typing group.
        if self
            .group
            .as_ref()
            .is_some_and(|group| group.origin != Some(origin))
        {
            self.group = None;
        }
    }
}

impl Transaction {
    pub(crate) fn origin(&self) -> Option<ViewId> {
        self.origin
    }
}
