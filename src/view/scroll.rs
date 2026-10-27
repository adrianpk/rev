use crate::buffer::Buffer;
use crate::command::Scroll;
use crate::coordinates::{Cell, Coordinates};

use super::{Error, Size, View};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Alignment {
    Center,
    Top,
    Bottom,
}

impl View {
    /// Nonvertical commands and direct insertion start a new navigation sequence.
    pub(crate) fn reset_navigation(&mut self) {
        self.column = None;
        self.alignment = Alignment::Center;
    }

    pub(crate) fn scroll(
        &mut self,
        buffer: &Buffer,
        scroll: Scroll,
        distance: usize,
    ) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        self.sync_policy(buffer);
        self.alignment = Alignment::Center;
        if distance == 0 {
            return Ok(());
        }
        let column = *self.column.get_or_insert(point.column);
        let row = destination(point.row, distance, scroll.backward()).min(buffer.len_lines() - 1);
        self.point.position = Coordinates::new(buffer)?.from_cell(Cell { row, column })?;
        if scroll.page() {
            self.top = destination(self.top, point.row.abs_diff(row), scroll.backward())
                .min(buffer.len_lines() - 1);
        }
        Ok(())
    }

    pub(crate) fn recenter(&mut self, buffer: &Buffer, size: Size) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        self.sync_policy(buffer);
        self.align(point, size);
        Ok(())
    }

    fn align(&mut self, point: Cell, size: Size) {
        if !size.usable() {
            return;
        }
        let rows = usize::from(size.text_rows());
        let (offset, next) = match self.alignment {
            Alignment::Center => (rows / 2, Alignment::Top),
            Alignment::Top => (0, Alignment::Bottom),
            Alignment::Bottom => (rows - 1, Alignment::Center),
        };
        self.top = point.row.saturating_sub(offset);
        self.alignment = next;
    }
}

fn destination(origin: usize, distance: usize, backward: bool) -> usize {
    if backward {
        origin.saturating_sub(distance)
    } else {
        origin.saturating_add(distance)
    }
}
