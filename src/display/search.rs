use crate::buffer::{Buffer, Span};
use crate::coordinates::Error as CoordinateError;
use crate::view::View;
use crate::window::Area;

use super::{Error, Frame, selection::Highlight};

impl Frame {
    /// Search marks only the selected pane, independently of region styling.
    pub(crate) fn search(
        &mut self,
        span: &Span,
        buffer: &Buffer,
        view: &View,
        area: Option<Area>,
    ) -> Result<(), Error> {
        let Some(area) = area else {
            return Ok(());
        };
        let highlight = Highlight::from_span(buffer, span)?;
        for local in 0..area.height().saturating_sub(1) {
            let row = area
                .row()
                .checked_add(local)
                .ok_or(CoordinateError::Overflow)?;
            let target = self
                .rows
                .get_mut(usize::from(row))
                .ok_or(CoordinateError::Overflow)?;
            target.search = highlight
                .row(
                    view.top.saturating_add(usize::from(local)),
                    view.left,
                    area.width(),
                )
                .map(|range| area.column() + range.start..area.column() + range.end);
        }
        Ok(())
    }
}
