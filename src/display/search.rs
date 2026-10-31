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
        let geometry = view.gutter_geometry(buffer, panel.content())?;
        let panel = Panel::new(geometry.text(), false, panel.active());
        let highlight = Highlight::from_span(buffer, span)?;
        let text = panel.content();
        let fragments = if buffer.soft_wrap() {
            view.wrapped_rows(buffer, text.width(), text.height())?
        } else {
            Vec::new()
        };
        for local in 0..panel.content().height() {
            let columns = if buffer.soft_wrap() {
                fragments.get(usize::from(local)).and_then(|f| {
                    highlight.row(f.line, f.columns.start, f.highlight_width(text.width()))
                })
            } else {
                view.source_row(buffer, usize::from(local))
                    .and_then(|source| highlight.row(source, view.left, text.width()))
            };
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
