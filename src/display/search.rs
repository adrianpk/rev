use crate::buffer::{Buffer, Span};
use crate::coordinates::Error as CoordinateError;
use crate::panel::Panel;
use crate::view::View;

use super::{Error, Frame, selection::Highlight};

impl Frame {
    /// Search marks only selected panel content, independently of region styling.
    pub(crate) fn search(
        &mut self,
        span: &Span,
        buffer: &Buffer,
        view: &View,
        panel: Option<Panel>,
    ) -> Result<(), Error> {
        let Some(panel) = panel else {
            return Ok(());
        };
        let highlight = Highlight::from_span(buffer, span)?;
        for local in 0..panel.content().height() {
            let columns = highlight.row(
                view.top.saturating_add(usize::from(local)),
                view.left,
                panel.content().width(),
            );
            if let Some((row, columns)) = columns.and_then(|columns| panel.clip(local, columns)) {
                let target = self
                    .rows
                    .get_mut(usize::from(row))
                    .ok_or(CoordinateError::Overflow)?;
                target.search = Some(columns);
            }
        }
        Ok(())
    }
}
