use super::super::{Buffer, presentation::Mapping};
use crate::org::Outline;

/// Borrowed display evidence only; it cannot escape as an action outline.
pub(super) struct Display<'a> {
    outline: &'a Outline,
    mapping: Option<&'a Mapping>,
}

impl Display<'_> {
    pub(super) fn base_byte(&self, row: usize, current: usize) -> Option<usize> {
        self.mapping
            .map_or(Some(current), |map| map.base_byte(row, current))
    }

    pub(super) fn indentation(&self, row: usize, start: usize) -> Option<crate::org::Indentation> {
        self.outline.indentation_at(self.base_byte(row, start)?)
    }

    pub(super) fn heading(&self, row: usize, start: usize) -> Option<usize> {
        self.outline.heading_level_at(self.base_byte(row, start)?)
    }

    pub(super) fn same_geometry(&self, next: &Outline) -> bool {
        self.outline.same_presentation(next, |byte| {
            self.mapping
                .map_or(Some(byte), |map| map.current_position(byte))
        })
    }
}

impl Buffer {
    pub(super) fn org_display(&self) -> Option<Display<'_>> {
        if !self.is_org() {
            return None;
        }
        let outline = self.org_outline.as_ref()?;
        outline.indentation_at(0)?;
        outline.presentation_limit(self.len_bytes())?;
        let mapping = if outline.stamp == self.stamp() {
            None
        } else {
            Some(
                self.presentation
                    .as_ref()
                    .filter(|map| map.current == self.stamp() && map.base == outline.stamp)?,
            )
        };
        Some(Display { outline, mapping })
    }

    pub(crate) fn org_geometry(&self) -> Option<u64> {
        self.org_display().map(|_| self.org_geometry)
    }
}
