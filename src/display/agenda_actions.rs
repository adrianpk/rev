use super::*;
use crate::org::agenda::ActionResult;
impl Frame {
    pub(crate) fn org_action_results(
        &mut self,
        results: &[ActionResult],
        top: usize,
        area: Area,
        active: bool,
    ) -> Result<(), Error> {
        for row in 0..area.height() {
            let line = Area::new(area.column(), area.row() + row, area.width(), 1)
                .ok_or(crate::coordinates::Error::Overflow)?;
            let text = if row == 0 {
                format!(
                    "Agenda actions: {} targets [buffer changes; q returns]",
                    results.len()
                )
            } else {
                results
                    .get(top + usize::from(row - 1))
                    .map(|r| {
                        format!(
                            "{} {}: {}: {}",
                            r.status.label(),
                            r.target
                                .source
                                .path
                                .as_ref()
                                .and_then(|p| p.file_name())
                                .map(|p| p.to_string_lossy())
                                .unwrap_or(std::borrow::Cow::Borrowed("buffer")),
                            r.target.title,
                            r.message
                        )
                    })
                    .unwrap_or_default()
            };
            self.projection_text(&text, line)?;
        }
        if active {
            self.cursor = None;
        }
        Ok(())
    }
}
