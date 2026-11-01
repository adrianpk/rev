use crate::buffer::Buffer;
use crate::command::navigation::Screen;
use crate::coordinates::{Cell, Coordinates, Scalar};

use super::{Error, Size, View};

impl View {
    /// Select an actual displayed row using the existing fold/wrap projection.
    pub(crate) fn screen_target(
        &self,
        buffer: &Buffer,
        size: Size,
        screen: Screen,
        count: usize,
    ) -> Result<Scalar, Error> {
        if !size.usable() {
            return Ok(self.point.position);
        }
        let width = self.text_width(buffer, size)?;
        let rows = size.text_rows();
        if buffer.soft_wrap() && width > 0 {
            let fragments = self.wrapped_rows(buffer, width, rows)?;
            let Some(fragment) = fragments.get(index(fragments.len(), screen, count)) else {
                return Ok(self.point.position);
            };
            return Self::fragment_point(buffer, fragment, 0);
        }
        let top = self.projected_top(buffer);
        let length = usize::from(rows).min(self.visible_lines(buffer).saturating_sub(top));
        let row = self
            .source_at(buffer, top + index(length, screen, count))
            .ok_or(crate::coordinates::Error::Overflow)?;
        let coordinates = Coordinates::new(buffer)?;
        let point = coordinates.from_cell(Cell { row, column: 0 })?;
        Ok(coordinates.navigation(point, crate::command::navigation::Target::FirstText, 1)?)
    }

    pub(crate) fn align_screen(
        &mut self,
        buffer: &Buffer,
        size: Size,
        screen: Screen,
    ) -> Result<(), Error> {
        self.alignment = match screen {
            Screen::Top => super::scroll::Alignment::Top,
            Screen::Middle => super::scroll::Alignment::Center,
            Screen::Bottom => super::scroll::Alignment::Bottom,
        };
        self.recenter(buffer, size)
    }
}

fn index(length: usize, screen: Screen, count: usize) -> usize {
    let last = length.saturating_sub(1);
    match screen {
        Screen::Top => count.saturating_sub(1).min(last),
        Screen::Middle => last / 2,
        Screen::Bottom => length.saturating_sub(count),
    }
}
