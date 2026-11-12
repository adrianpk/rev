use crate::config;
use crate::coordinates::{Error, Glyphs};

use super::Tree;

/// Desired columns survive temporary layout constraints. Only explicit width
/// actions replace ownership; automatic observations can grow, never shrink.
pub(super) struct Width {
    pub(super) desired: u16,
    automatic: bool,
}

impl Default for Width {
    fn default() -> Self {
        Self::configured(&config::Tree::default())
    }
}

impl Width {
    pub(super) fn configured(policy: &config::Tree) -> Self {
        Self {
            desired: u16::try_from(policy.width).expect("admitted width fits terminal cells"),
            automatic: policy.auto_width,
        }
    }

    pub(super) fn for_context(&self, policy: &config::Tree) -> Self {
        let mut next = Self::configured(policy);
        next.automatic = self.automatic;
        if !self.automatic {
            next.desired = self.desired;
        }
        next
    }
}

impl Tree {
    /// The desired width is independent from visibility and frame geometry.
    pub fn desired_width(&self) -> u16 {
        self.width.desired
    }

    pub fn auto_width(&self) -> bool {
        self.width.automatic
    }

    pub(crate) fn adjust_width(&mut self, widen: bool) {
        self.width.desired = if widen {
            self.width.desired.saturating_add(4)
        } else {
            self.width.desired.saturating_sub(4).max(1)
        };
        self.width.automatic = false;
    }

    pub(crate) fn reset_width(&mut self) {
        self.width = Width::configured(&self.policy);
        self.viewport.reset();
        self.observe_width();
    }

    pub(crate) fn toggle_auto_width(&mut self) {
        self.width.automatic = !self.width.automatic;
        if self.width.automatic {
            self.width.desired =
                u16::try_from(self.policy.width).expect("admitted width fits terminal cells");
            self.observe_width();
        }
    }

    pub(crate) fn inherit_width(&mut self, previous: &Self) {
        self.width = previous.width.for_context(&self.policy);
        self.viewport.reset();
        self.observe_width();
    }

    pub(super) fn new_width_context(&mut self) {
        self.width = self.width.for_context(&self.policy);
        self.viewport.reset();
        self.observe_width();
    }

    /// Called only when a retained listing is published or width ownership
    /// changes. Rendering, navigation and resizing do not measure or discover.
    pub(super) fn observe_width(&mut self) {
        let extent = self
            .content_width()
            .expect("retained tree labels have valid bounded display coordinates");
        self.viewport.observe(extent);
        if self.width.automatic {
            self.width.desired = self.width.desired.max(extent);
        }
    }

    fn content_width(&self) -> Result<u16, Error> {
        let mut extent = 0;
        for entry in &self.entries {
            let label = entry.label(self.policy.decorated, self.nerd_fonts)?;
            for glyph in Glyphs::for_text(&label)? {
                let end = u16::try_from(glyph?.columns.end).unwrap_or(u16::MAX);
                extent = extent.max(end);
            }
        }
        Ok(extent)
    }
}
