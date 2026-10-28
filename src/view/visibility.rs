use std::sync::Arc;

use crate::buffer::Buffer;
use crate::command::OrgCommand;
use crate::coordinates::{Byte, Coordinates};
use crate::org::{Outline, Source, SourceRange, Visibility};

use super::{Error, View};

impl View {
    pub(crate) fn org_visibility(
        &mut self,
        buffer: &Buffer,
        model: Arc<Outline>,
        command: OrgCommand,
    ) -> Result<(), String> {
        let byte = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(self.point.position)
            .map_err(|e| e.to_string())?
            .0;
        let current = self
            .visibility
            .as_ref()
            .filter(|v| v.model.stamp == buffer.stamp())
            .map(|v| v.as_ref().clone())
            .unwrap_or_else(|| Visibility::new(model));
        let next = current.change(command, byte).map_err(str::to_owned)?;
        self.install_visibility(buffer, next)
            .map_err(|e| e.to_string())
    }
    fn install_visibility(&mut self, buffer: &Buffer, next: Visibility) -> Result<(), Error> {
        let spans = next.spans();
        let ranges: Vec<_> = spans
            .iter()
            .map(|span| Source::new(buffer).range(Byte(span.bytes.start)..Byte(span.bytes.end)))
            .collect::<Result<_, _>>()?;
        let mut candidate = self.clone();
        let row = self.cell(buffer)?.row;
        for (span, range) in spans.iter().zip(&ranges) {
            if hidden(buffer, range, row)? {
                candidate.point.position =
                    Coordinates::new(buffer)?.from_byte(Byte(span.anchor))?;
                break;
            }
        }
        candidate.set_folds(buffer, &ranges)?;
        candidate.visibility = Some(Arc::new(next));
        *self = candidate;
        Ok(())
    }
    pub(super) fn reveal_visibility(&mut self, buffer: &Buffer, row: usize) -> Result<bool, Error> {
        let Some(state) = self
            .visibility
            .as_ref()
            .filter(|state| state.model.stamp == buffer.stamp())
        else {
            return Ok(false);
        };
        if self.project_row(buffer, row).is_some() {
            return Ok(false);
        }
        let byte = Coordinates::new(buffer)?.to_byte(self.point.position)?.0;
        let next = state.reveal(byte);
        self.install_visibility(buffer, next)?;
        Ok(true)
    }
}

fn hidden(buffer: &Buffer, range: &SourceRange, row: usize) -> Result<bool, Error> {
    let coordinates = Coordinates::new(buffer)?;
    let start = coordinates.to_line(crate::coordinates::Scalar(range.span().range.start))?;
    let end = coordinates.to_line(crate::coordinates::Scalar(range.span().range.end))?;
    Ok(start.row <= row && row < end.row + usize::from(end.column != 0))
}
