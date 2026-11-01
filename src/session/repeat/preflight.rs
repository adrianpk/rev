use super::{Intent, Payload, replay::override_count, selection::Geometry};
use crate::coordinates::Coordinates;
use crate::session::{Session, editing::Error};

impl Session {
    /// Reuse native deferred layout for the retained intent's NEW target. No
    /// input, history, source or repeat record is published during preparation.
    pub(in crate::session) fn repeat_preflight(&self, count: Option<u32>) -> Result<(), Error> {
        let Some(record) = &self.repeat.last else {
            return Ok(());
        };
        match &record.intent {
            Intent::Entry(entry) => self.repeat_entry_preflight(*entry)?,
            Intent::Edit(request, None) => {
                self.editing_preflight(override_count(*request, count))?;
            }
            Intent::Edit(_, Some(geometry)) => self.repeat_geometry_preflight(*geometry, count)?,
            Intent::Paste { payload, .. } => self.repeat_paste_preflight(payload)?,
        }
        Ok(())
    }

    fn repeat_entry_preflight(&self, entry: crate::command::ModalCommand) -> Result<(), Error> {
        let at = self.modal_target(entry)?;
        Coordinates::new(self.buffer()?)?.line_end(at)?;
        Ok(())
    }

    fn repeat_geometry_preflight(
        &self,
        geometry: Geometry,
        count: Option<u32>,
    ) -> Result<(), Error> {
        match geometry {
            Geometry::Block { .. } => {
                let selection = self.repeat_block_selection(geometry, count)?;
                selection.deletions(self.buffer()?)?;
            }
            Geometry::Linear { .. } => {
                self.repeat_selection(geometry, count)?;
            }
        }
        Ok(())
    }

    fn repeat_paste_preflight(&self, payload: &Payload) -> Result<(), Error> {
        if let Payload::Block(block) = payload {
            block.insertions(self.buffer()?, self.view().point)?;
        } else {
            let coordinates = Coordinates::new(self.buffer()?)?;
            let end = coordinates.line_end(self.view().point.position)?;
            coordinates.next(end)?;
        }
        Ok(())
    }
}
