use std::sync::Arc;

use crate::buffer::Buffer;
use crate::command::OrgCommand;
use crate::coordinates::{Byte, Coordinates};
use crate::org::{Outline, Source, SourceRange, Visibility};

use super::{Error, View};

/// Transient presentation retained by the existing live-buffer position owner.
/// Offsets remain revision-bound, and each window keeps its own choices.
#[derive(Clone)]
pub(crate) struct Presentation {
    folds: Option<super::fold::Folds>,
    visibility: Option<Arc<Visibility>>,
    initialized: bool,
    suppressed: bool,
}

impl View {
    pub(crate) fn org_presentation(&self) -> Presentation {
        Presentation {
            folds: self.folds.clone(),
            visibility: self.visibility.clone(),
            initialized: self.visibility_initialized,
            suppressed: self.startup_suppressed,
        }
    }

    pub(crate) fn restore_org_presentation(&mut self, state: &Presentation) {
        self.folds = state.folds.clone();
        self.visibility = state.visibility.clone();
        self.visibility_initialized = state.initialized;
        self.startup_suppressed = state.suppressed;
    }

    pub(crate) fn pending_org_visibility(&self) -> bool {
        !self.visibility_initialized && !self.startup_suppressed
    }

    pub(crate) fn suppress_org_startup(&mut self) {
        self.startup_suppressed = true;
    }

    pub(crate) fn initial_org_visibility(
        &mut self,
        buffer: &Buffer,
        model: Arc<Outline>,
        configured: crate::config::StartupVisibility,
    ) -> Result<(), String> {
        if !self.pending_org_visibility() || !model.startup_ready() {
            return Ok(());
        }
        let next = Visibility::new(model)
            .startup(configured)
            .map_err(str::to_owned)?;
        self.install_visibility(buffer, next)
            .map_err(|e| e.to_string())?;
        self.visibility_initialized = true;
        Ok(())
    }

    pub(crate) fn initial_org_drawer(
        &mut self,
        buffer: &Buffer,
        model: Option<Arc<Outline>>,
        point: crate::coordinates::Scalar,
        body: std::ops::Range<usize>,
    ) -> Result<(), String> {
        if self.visibility_initialized || self.visibility.is_some() {
            return Ok(());
        }
        let Some(model) = model else {
            // The editor knows its own footer boundaries. Project those lines
            // before the first frame while the normal bounded Org worker runs.
            if self.folds.is_none() {
                let range = Source::new(buffer)
                    .range(Byte(body.start)..Byte(body.end))
                    .map_err(|e| e.to_string())?;
                self.set_folds(buffer, &[range])
                    .map_err(|e| e.to_string())?;
            }
            return Ok(());
        };
        let byte = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(point)
            .map_err(|e| e.to_string())?
            .0;
        let next = Visibility::new(model)
            .change(OrgCommand::HideDrawer, byte)
            .map_err(str::to_owned)?;
        self.install_visibility(buffer, next)
            .map_err(|e| e.to_string())?;
        self.visibility_initialized = true;
        Ok(())
    }

    pub(crate) fn org_visibility(
        &mut self,
        buffer: &Buffer,
        model: Arc<Outline>,
        command: OrgCommand,
        configured: crate::config::StartupVisibility,
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
        let next = if command == OrgCommand::CycleSetStartupVisibility {
            current.startup(configured)
        } else {
            current.change(command, byte)
        }
        .map_err(str::to_owned)?;
        self.install_visibility(buffer, next)
            .map_err(|e| e.to_string())?;
        self.visibility_initialized = true;
        Ok(())
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
