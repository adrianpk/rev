use super::Tree;

/// A single display-column interval for every admitted row. Extent is observed
/// only at listing publication; geometry and pan operations use cached values.
#[derive(Default)]
pub(super) struct Viewport {
    extent: u16,
    offset: u16,
    width: u16,
}

impl Viewport {
    pub(super) fn reset(&mut self) {
        self.offset = 0;
    }

    pub(super) fn observe(&mut self, extent: u16) {
        self.extent = extent;
        self.clamp(self.width);
    }

    fn clamp(&mut self, width: u16) {
        // A hidden/unusable panel retains its position until usable space returns.
        if width == 0 {
            return;
        }
        self.width = width;
        self.offset = self.offset.min(self.extent.saturating_sub(width));
    }
}

impl Tree {
    /// One retained display-column displacement shared by every tree row.
    pub fn horizontal_offset(&self) -> u16 {
        self.viewport.offset
    }

    pub(crate) fn clamp_horizontal(&mut self, width: u16) {
        self.viewport.clamp(width);
    }

    pub(crate) fn pan(&mut self, right: bool, width: u16) {
        self.clamp_horizontal(width);
        if width == 0 {
            return;
        }
        self.viewport.offset = if right {
            self.viewport.offset.saturating_add(1)
        } else {
            self.viewport.offset.saturating_sub(1)
        };
        self.clamp_horizontal(width);
    }
}
