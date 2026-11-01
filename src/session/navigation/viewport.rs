use crate::command::navigation::Target;
use crate::command::{Navigation, Scroll};
use crate::search::Direction;
use crate::view::{Error, View};

use super::super::Session;

impl Session {
    pub(super) fn navigation_viewport(
        &self,
        candidate: &mut View,
        request: Navigation,
    ) -> Result<bool, Error> {
        let buffer = self.buffer()?;
        let size = self.pane_size();
        match request.target {
            Target::Line(direction) => {
                candidate.scroll(buffer, line(direction), request.amount())?
            }
            Target::Page { backward, half } => {
                self.navigation_page(candidate, (backward, half), request.amount())?
            }
            Target::Screen(screen) => {
                candidate.point.position =
                    candidate.screen_target(buffer, size, screen, request.amount())?;
                candidate.reset_navigation();
            }
            Target::Align(screen) => candidate.align_screen(buffer, size, screen)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn navigation_page(
        &self,
        candidate: &mut View,
        policy: (bool, bool),
        count: usize,
    ) -> Result<(), Error> {
        let size = self.pane_size();
        if !size.usable() {
            return Ok(());
        }
        let buffer = self.buffer()?;
        let width = candidate.text_width(buffer, size)?;
        let (backward, half) = policy;
        let rows = usize::from(size.text_rows());
        let overlap = self.buffers.snapshot().settings().scroll.page_overlap;
        let page = if half {
            (rows / 2).max(1)
        } else {
            rows.saturating_sub(overlap).max(1)
        };
        let distance = page
            .checked_mul(count)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let scroll = if backward {
            Scroll::PageBackward
        } else {
            Scroll::PageForward
        };
        if buffer.soft_wrap() && width > 0 {
            candidate.visual_scroll(buffer, width, scroll, distance)
        } else {
            candidate.scroll(buffer, scroll, distance)
        }
    }
}

fn line(direction: Direction) -> Scroll {
    if direction == Direction::Forward {
        Scroll::NextLine
    } else {
        Scroll::PreviousLine
    }
}
